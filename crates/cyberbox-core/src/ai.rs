use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum AiEvent {
    Token(String),
    Done,
    Error(String),
}

/// One turn of prior conversation, sent back to Ollama's `/api/chat` on
/// every request — the endpoint is stateless, so full history travels with
/// each call, same as any OpenAI-style chat API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
}

const SYSTEM_PROMPT: &str = "You are Cyber Bro, an AI assistant embedded in cyber-box, a sandboxed \
penetration-testing terminal application. The user is an authorized security professional working \
inside this tool against systems they own or are authorized to test — the application itself already \
displays that warning permanently, so do not add legal disclaimers, ethics caveats, or reminders to \
only test authorized systems. Answer directly and concisely. Format responses in Markdown (tables, \
fenced code blocks, headings) where it helps readability. If terminal output is attached below, treat \
it as what the user is currently looking at and reference it directly when it's relevant to their \
question — don't ignore it.";

#[derive(Deserialize)]
struct ChatChunk {
    message: Option<ChatChunkMessage>,
    #[serde(default)]
    done: bool,
}

#[derive(Deserialize)]
struct ChatChunkMessage {
    content: String,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagsModel>,
}

#[derive(Deserialize)]
struct TagsModel {
    name: String,
}

/// Lists the models this Ollama instance actually has pulled, so the caller
/// can offer a real choice instead of assuming a specific model exists.
pub async fn list_models(ollama_url: &str) -> anyhow::Result<Vec<String>> {
    let url = format!("{}/api/tags", ollama_url.trim_end_matches('/'));
    let resp = reqwest::get(&url).await?.error_for_status()?;
    let parsed: TagsResponse = resp.json().await?;
    Ok(parsed.models.into_iter().map(|m| m.name).collect())
}

/// Sends `history` (prior turns) plus a new `question` (with optional
/// `context`, e.g. terminal output) to Ollama's `/api/chat` and streams the
/// response back through `tx`. No-op unless called — the caller is
/// responsible for gating this behind the AI-enabled toggle.
pub fn ask(
    ollama_url: String,
    model: String,
    history: Vec<ChatMessage>,
    question: String,
    context: String,
    tx: UnboundedSender<AiEvent>,
) {
    tokio::spawn(async move {
        let user_content = if context.trim().is_empty() {
            question
        } else {
            format!("Attached terminal output:\n{context}\n\nUser question: {question}")
        };

        let mut messages = Vec::with_capacity(history.len() + 2);
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: SYSTEM_PROMPT.to_string(),
        });
        messages.extend(history);
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_content,
        });

        let client = reqwest::Client::new();
        let url = format!("{}/api/chat", ollama_url.trim_end_matches('/'));
        let body = ChatRequest {
            model: &model,
            messages: &messages,
            stream: true,
        };

        let resp = match client.post(&url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(AiEvent::Error(format!(
                    "failed to reach Ollama at {url}: {e}"
                )));
                return;
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let detail = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(str::to_string))
                .unwrap_or(text);
            let _ = tx.send(AiEvent::Error(format!(
                "Ollama returned {status}: {detail}"
            )));
            return;
        }

        let mut stream = resp.bytes_stream();
        let mut buf = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(AiEvent::Error(format!("stream error: {e}")));
                    return;
                }
            };
            buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].to_string();
                buf.drain(..=pos);
                if line.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<ChatChunk>(&line) {
                    Ok(parsed) => {
                        if let Some(msg) = parsed.message {
                            if !msg.content.is_empty() {
                                let _ = tx.send(AiEvent::Token(msg.content));
                            }
                        }
                        if parsed.done {
                            let _ = tx.send(AiEvent::Done);
                            return;
                        }
                    }
                    Err(e) => {
                        let _ =
                            tx.send(AiEvent::Error(format!("failed to parse Ollama chunk: {e}")));
                        return;
                    }
                }
            }
        }
        let _ = tx.send(AiEvent::Done);
    });
}
