<script lang="ts">
  import { onMount, tick } from "svelte";
  import { api, onEvent } from "./api";

  let { open, enabled, getContext }: { open: boolean; enabled: boolean; getContext: () => string } = $props();

  interface Message {
    id: string;
    role: "user" | "assistant";
    text: string;
  }

  interface Segment {
    kind: "text" | "code";
    content: string;
    lang?: string;
  }

  const MODEL_STORAGE_KEY = "cyberbox-ai-model";

  let messages = $state<Message[]>([]);
  let input = $state("");
  let busy = $state(false);
  let copiedId = $state("");

  let models = $state<string[]>([]);
  let modelsLoaded = $state(false);
  let modelsError = $state("");
  let selectedModel = $state("");

  let messagesEl = $state<HTMLDivElement>();
  let inputEl = $state<HTMLTextAreaElement>();

  async function loadModels() {
    try {
      models = await api.listOllamaModels();
      modelsError = models.length === 0 ? "No Ollama models found. Pull one with `ollama pull <model>`." : "";
      const remembered = localStorage.getItem(MODEL_STORAGE_KEY);
      if (remembered && models.includes(remembered)) {
        selectedModel = remembered;
      }
    } catch (e) {
      models = [];
      modelsError = `Couldn't reach Ollama: ${e}`;
    } finally {
      modelsLoaded = true;
    }
  }

  function onModelChange() {
    if (selectedModel) localStorage.setItem(MODEL_STORAGE_KEY, selectedModel);
  }

  function newChat() {
    if (busy) return;
    messages = [];
    tick().then(() => inputEl?.focus());
  }

  /** Splits AI output on ```fenced``` code blocks so commands render in monospace. */
  function segments(text: string): Segment[] {
    const parts: Segment[] = [];
    const re = /```(\w+)?\n?([\s\S]*?)```/g;
    let last = 0;
    let match: RegExpExecArray | null;
    while ((match = re.exec(text))) {
      if (match.index > last) parts.push({ kind: "text", content: text.slice(last, match.index) });
      parts.push({ kind: "code", content: match[2].replace(/\n$/, ""), lang: match[1] });
      last = match.index + match[0].length;
    }
    if (last < text.length) parts.push({ kind: "text", content: text.slice(last) });
    return parts;
  }

  async function copyMessage(m: Message) {
    await navigator.clipboard.writeText(m.text).catch(() => {});
    copiedId = m.id;
    setTimeout(() => {
      if (copiedId === m.id) copiedId = "";
    }, 1200);
  }

  async function scrollToBottom() {
    await tick();
    if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
  }

  function autosize() {
    if (!inputEl) return;
    inputEl.style.height = "auto";
    inputEl.style.height = `${Math.min(inputEl.scrollHeight, 120)}px`;
  }

  onMount(() => {
    loadModels();
  });

  $effect(() => {
    if (open) {
      tick().then(() => inputEl?.focus());
    }
  });

  async function ask() {
    if (!enabled || busy || !selectedModel || !input.trim()) return;
    const question = input.trim();
    input = "";
    await tick();
    autosize();
    messages = [
      ...messages,
      { id: crypto.randomUUID(), role: "user", text: question },
      { id: crypto.randomUUID(), role: "assistant", text: "" },
    ];
    busy = true;
    scrollToBottom();

    const requestId = crypto.randomUUID();
    const context = getContext();

    const unToken = await onEvent<string>(`ai:${requestId}:token`, (t) => {
      const last = messages[messages.length - 1];
      messages = [...messages.slice(0, -1), { ...last, text: last.text + t }];
      scrollToBottom();
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

    api.askAi(requestId, selectedModel, question, context).catch(() => {
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
      <span
        >AI Agent {#if !enabled}<span class="off">(disabled)</span>{/if}</span
      >
      <div class="header-actions">
        {#if enabled && models.length > 0}
          <select
            class="model-select"
            bind:value={selectedModel}
            onchange={onModelChange}
            title="Ollama model"
          >
            <option value="" disabled>choose a model…</option>
            {#each models as m (m)}
              <option value={m}>{m}</option>
            {/each}
          </select>
        {:else if enabled && modelsLoaded}
          <button class="icon-btn" onclick={loadModels} title="Retry loading models">&#8635;</button>
        {/if}
        {#if enabled}
          <button class="icon-btn" onclick={newChat} disabled={busy || messages.length === 0} title="New chat"
            >&#9998; New</button
          >
        {/if}
      </div>
    </div>
    <div class="messages" bind:this={messagesEl}>
      {#if !enabled}
        <div class="hint">AI suggestions are off. Toggle them from the status bar.</div>
      {:else if modelsError}
        <div class="hint error">{modelsError}</div>
      {:else if !selectedModel}
        <div class="hint">
          Pick a model above, then ask about the active terminal's output or anything else.
        </div>
      {:else if messages.length === 0}
        <div class="hint">Ask about the active terminal's output, a tool, or anything else.</div>
      {/if}
      {#each messages as m, i (m.id)}
        <div class="msg {m.role}">
          <div class="msg-head">
            <span class="role">{m.role === "user" ? "you" : "ai"}</span>
            {#if m.role === "assistant" && m.text}
              <button class="copy-btn" onclick={() => copyMessage(m)} title="Copy response">
                {copiedId === m.id ? "copied" : "copy"}
              </button>
            {/if}
          </div>
          {#if m.role === "assistant" && !m.text && busy && i === messages.length - 1}
            <div class="thinking"><span></span><span></span><span></span></div>
          {:else}
            <div class="text">
              {#each segments(m.text) as seg, si (si)}
                {#if seg.kind === "code"}
                  <pre class="code">{seg.content}</pre>
                {:else}
                  {seg.content}
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
    <div class="input-row">
      <textarea
        bind:this={inputEl}
        bind:value={input}
        onkeydown={onKeydown}
        oninput={autosize}
        disabled={!enabled || !selectedModel}
        placeholder={!enabled
          ? "AI is disabled"
          : !selectedModel
            ? "Choose a model above first"
            : "Ask the AI agent… (Shift+Enter for a new line)"}
        rows="1"></textarea>
      <button onclick={ask} disabled={!enabled || !selectedModel || busy}>Send</button>
    </div>
  </div>
{/if}

<style>
  .panel {
    position: absolute;
    right: 12px;
    bottom: 44px;
    width: 380px;
    height: 520px;
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
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .off {
    color: var(--text-faint);
    font-weight: 400;
  }
  .model-select {
    max-width: 160px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-size: 11px;
    font-weight: 400;
    padding: 3px 4px;
  }
  .icon-btn {
    background: transparent;
    border: none;
    color: var(--text-faint);
    font-size: 11px;
    font-weight: 400;
    letter-spacing: normal;
    white-space: nowrap;
    padding: 3px 7px;
    border-radius: 6px;
  }
  .icon-btn:hover:not(:disabled) {
    background: var(--border-soft);
    color: var(--text);
  }
  .icon-btn:disabled {
    opacity: 0.4;
  }
  .hint.error {
    color: var(--danger);
  }
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .hint {
    color: var(--text-faint);
    font-size: 12px;
    line-height: 1.5;
  }
  .msg-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2px;
  }
  .msg .role {
    font-size: 10px;
    text-transform: uppercase;
    color: var(--text-faint);
  }
  .copy-btn {
    background: transparent;
    border: none;
    color: var(--text-faint);
    font-size: 10px;
    padding: 0 4px;
    border-radius: 4px;
    opacity: 0;
    transition: opacity 0.1s;
  }
  .msg:hover .copy-btn {
    opacity: 1;
  }
  .copy-btn:hover {
    color: var(--text);
    background: var(--border-soft);
  }
  .msg.user .text {
    color: var(--text);
  }
  .msg.assistant .text {
    color: var(--accent);
  }
  .text {
    font-size: 12.5px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .code {
    display: block;
    margin: 4px 0;
    padding: 8px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--text);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-x: auto;
  }
  .thinking {
    display: flex;
    gap: 4px;
    padding: 4px 0;
  }
  .thinking span {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
    opacity: 0.4;
    animation: pulse 1.1s infinite ease-in-out;
  }
  .thinking span:nth-child(2) {
    animation-delay: 0.15s;
  }
  .thinking span:nth-child(3) {
    animation-delay: 0.3s;
  }
  @keyframes pulse {
    0%,
    80%,
    100% {
      opacity: 0.25;
      transform: scale(0.85);
    }
    40% {
      opacity: 1;
      transform: scale(1.1);
    }
  }
  .input-row {
    display: flex;
    align-items: flex-end;
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
    max-height: 120px;
    overflow-y: auto;
  }
  button {
    background: var(--accent-dim);
    color: #06170a;
    border: none;
    border-radius: 8px;
    padding: 0 14px;
    height: 30px;
    font-weight: 600;
    font-size: 12px;
    flex-shrink: 0;
  }
  button:disabled {
    background: var(--border);
    color: var(--text-faint);
  }
</style>
