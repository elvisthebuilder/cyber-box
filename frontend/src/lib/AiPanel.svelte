<script lang="ts">
  import { onMount, tick } from "svelte";
  import { api, onEvent } from "./api";
  import Icon from "./Icon.svelte";

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
  let modelMenuOpen = $state(false);

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

  function persistModel() {
    if (selectedModel) localStorage.setItem(MODEL_STORAGE_KEY, selectedModel);
  }

  function toggleModelMenu(e: MouseEvent) {
    e.stopPropagation();
    modelMenuOpen = !modelMenuOpen;
  }

  function chooseModel(m: string) {
    selectedModel = m;
    persistModel();
    modelMenuOpen = false;
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

<svelte:window onclick={() => (modelMenuOpen = false)} />

{#if open}
  <div class="panel">
    <div class="header">
      <span
        >Cyber Bro {#if !enabled}<span class="off">(disabled)</span>{/if}</span
      >
      <div class="header-actions">
        {#if enabled && models.length > 0}
          <div class="model-picker">
            <button class="model-btn" onclick={toggleModelMenu} title="Choose Ollama model">
              <span class="model-btn-label">{selectedModel || "choose a model…"}</span>
              <span class="chevron" class:open={modelMenuOpen}><Icon name="chevron-down" size={11} /></span>
            </button>
            {#if modelMenuOpen}
              <div class="model-menu">
                {#each models as m (m)}
                  <button
                    class="model-option"
                    class:selected={m === selectedModel}
                    onclick={() => chooseModel(m)}
                  >
                    {m}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {:else if enabled && modelsLoaded}
          <button class="icon-btn" onclick={loadModels} title="Retry loading models"
            ><Icon name="rotate-ccw" /></button
          >
        {/if}
        {#if enabled}
          <button class="icon-btn" onclick={newChat} disabled={busy || messages.length === 0} title="New chat"
            ><Icon name="plus" /></button
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
                <Icon name={copiedId === m.id ? "check" : "copy"} size={12} />
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
            : "Ask Cyber Bro… (Shift+Enter for a new line)"}
        rows="1"></textarea>
      <button
        class="send-btn"
        onclick={ask}
        disabled={!enabled || !selectedModel || busy || !input.trim()}
        title="Send"
      >
        {#if busy}
          <span class="spinner"></span>
        {:else}
          <Icon name="send" size={14} />
        {/if}
      </button>
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
  .model-picker {
    position: relative;
  }
  .model-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: 170px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-size: 11px;
    font-weight: 400;
    letter-spacing: normal;
    padding: 4px 6px;
  }
  .model-btn:hover {
    border-color: var(--accent-dim);
  }
  .model-btn-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chevron {
    display: flex;
    color: var(--text-faint);
    transition: transform 0.15s;
    flex-shrink: 0;
  }
  .chevron.open {
    transform: rotate(180deg);
  }
  .model-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    min-width: 100%;
    max-width: 260px;
    max-height: 220px;
    overflow-y: auto;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    padding: 4px;
    z-index: 30;
  }
  .model-option {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text);
    font-size: 11.5px;
    font-weight: 400;
    letter-spacing: normal;
    padding: 6px 8px;
    border-radius: 5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .model-option:hover {
    background: var(--border-soft);
  }
  .model-option.selected {
    color: var(--accent);
    background: rgba(57, 211, 83, 0.1);
  }
  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: none;
    color: var(--text-faint);
    font-size: 12px;
    font-weight: 400;
    letter-spacing: normal;
    padding: 0;
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
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: transparent;
    border: none;
    color: var(--text-faint);
    padding: 0;
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
    align-items: center;
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
    padding: 7px 8px;
    font-family: var(--font-ui);
    font-size: 12px;
    line-height: 1.4;
    max-height: 120px;
    overflow-y: auto;
    box-sizing: border-box;
  }
  .send-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    flex-shrink: 0;
    background: var(--accent-dim);
    color: #06170a;
    border: none;
    border-radius: 8px;
  }
  .send-btn:hover:not(:disabled) {
    background: var(--accent);
  }
  .send-btn:disabled {
    background: var(--border);
    color: var(--text-faint);
  }
  .spinner {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 2px solid rgba(6, 23, 10, 0.35);
    border-top-color: #06170a;
    animation: spin 0.7s linear infinite;
  }
  .send-btn:disabled .spinner {
    border-color: rgba(255, 255, 255, 0.15);
    border-top-color: var(--text-faint);
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
