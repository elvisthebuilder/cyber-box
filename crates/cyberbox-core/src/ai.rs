use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum AiEvent {
    Token(String),
    Done,
    Error(String),
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateChunk {
    response: Option<String>,
    #[serde(default)]
    done: bool,
}

/// Sends `question` plus recent tool-output `context` to Ollama and streams
/// the response back through `tx`. No-op unless called — the caller is
/// responsible for gating this behind the AI-enabled toggle.
pub fn ask(
    ollama_url: String,
    model: String,
    question: String,
    context: String,
    tx: UnboundedSender<AiEvent>,
) {
    tokio::spawn(async move {
        let prompt = if context.trim().is_empty() {
            question
        } else {
            format!("Recent tool output:\n{context}\n\nUser question: {question}")
        };

        let client = reqwest::Client::new();
        let url = format!("{}/api/generate", ollama_url.trim_end_matches('/'));
        let body = GenerateRequest {
            model: &model,
            prompt: &prompt,
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
                match serde_json::from_str::<GenerateChunk>(&line) {
                    Ok(parsed) => {
                        if let Some(text) = parsed.response {
                            let _ = tx.send(AiEvent::Token(text));
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
