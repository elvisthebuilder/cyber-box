<script lang="ts">
  import { api, onEvent } from "./api";

  let { open, enabled, getContext }: { open: boolean; enabled: boolean; getContext: () => string } = $props();

  interface Message {
    id: string;
    role: "user" | "assistant";
    text: string;
  }

  let messages = $state<Message[]>([]);
  let input = $state("");
  let busy = $state(false);

  async function ask() {
    if (!enabled || busy || !input.trim()) return;
    const question = input.trim();
    input = "";
    messages = [
      ...messages,
      { id: crypto.randomUUID(), role: "user", text: question },
      { id: crypto.randomUUID(), role: "assistant", text: "" },
    ];
    busy = true;

    const requestId = crypto.randomUUID();
    const context = getContext();

    const unToken = await onEvent<string>(`ai:${requestId}:token`, (t) => {
      const last = messages[messages.length - 1];
      messages = [...messages.slice(0, -1), { ...last, text: last.text + t }];
    });
    const unDone = await onEvent<void>(`ai:${requestId}:done`, () => {
      busy = false;
      unToken();
      unDone();
      unErr();
    });
    const unErr = await onEvent<string>(`ai:${requestId}:error`, (e) => {
      const last = messages[messages.length - 1];
      messages = [...messages.slice(0, -1), { ...last, text: `[error] ${e}` }];
      busy = false;
      unToken();
      unDone();
      unErr();
    });

    api.askAi(requestId, question, context).catch(() => {
      busy = false;
    });
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      ask();
    }
  }
</script>

{#if open}
  <div class="panel">
    <div class="header">
      AI Agent {#if !enabled}<span class="off">(disabled)</span>{/if}
    </div>
    <div class="messages">
      {#if !enabled}
        <div class="hint">AI suggestions are off. Toggle them from the status bar.</div>
      {:else if messages.length === 0}
        <div class="hint">Ask about the active terminal's output, a tool, or anything else.</div>
      {/if}
      {#each messages as m (m.id)}
        <div class="msg {m.role}">
          <div class="role">{m.role === "user" ? "you" : "ai"}</div>
          <div class="text">{m.text || "…"}</div>
        </div>
      {/each}
    </div>
    <div class="input-row">
      <textarea
        bind:value={input}
        onkeydown={onKeydown}
        disabled={!enabled}
        placeholder={enabled ? "Ask the AI agent…" : "AI is disabled"}
        rows="2"></textarea>
      <button onclick={ask} disabled={!enabled || busy}>Send</button>
    </div>
  </div>
{/if}

<style>
  .panel {
    position: absolute;
    right: 12px;
    bottom: 44px;
    width: 340px;
    height: 440px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    z-index: 20;
  }
  .header {
    padding: 10px 14px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    border-bottom: 1px solid var(--border);
    color: var(--text-dim);
  }
  .off {
    color: var(--text-faint);
    font-weight: 400;
  }
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .hint {
    color: var(--text-faint);
    font-size: 12px;
    line-height: 1.5;
  }
  .msg .role {
    font-size: 10px;
    text-transform: uppercase;
    color: var(--text-faint);
    margin-bottom: 2px;
  }
  .msg.user .text {
    color: var(--text);
  }
  .msg.assistant .text {
    color: var(--accent);
    white-space: pre-wrap;
  }
  .text {
    font-size: 12.5px;
    line-height: 1.5;
    white-space: pre-wrap;
  }
  .input-row {
    display: flex;
    gap: 6px;
    padding: 10px;
    border-top: 1px solid var(--border);
  }
  textarea {
    flex: 1;
    resize: none;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text);
    padding: 6px 8px;
    font-family: var(--font-ui);
    font-size: 12px;
  }
  button {
    background: var(--accent-dim);
    color: #06170a;
    border: none;
    border-radius: 8px;
    padding: 0 14px;
    font-weight: 600;
    font-size: 12px;
  }
  button:disabled {
    background: var(--border);
    color: var(--text-faint);
  }
</style>
