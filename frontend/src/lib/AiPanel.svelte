<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
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

  // Context attachment — mirrors Zed's "add context" chip: shows what will
  // be sent alongside the question, toggleable per-message.
  let contextEnabled = $state(true);
  let contextLines = $state(0);
  let contextPollTimer: ReturnType<typeof setInterval> | undefined;

  function refreshContextPreview() {
    const ctx = getContext();
    contextLines = ctx.trim() ? ctx.split("\n").length : 0;
  }

  // "New chat from summary" — mirrors Zed's New From Summary: summarize the
  // current thread with the model, then start fresh with that summary
  // carried forward as context instead of losing it outright.
  let newChatMenuOpen = $state(false);
  let summarizing = $state(false);
  let summaryError = $state("");
  let carriedSummary = $state("");

  async function loadModels() {
    try {
      models = await api.listOllamaModels();
      modelsError = models.length === 0 ? "No models pulled — click to retry" : "";
      const remembered = localStorage.getItem(MODEL_STORAGE_KEY);
      if (remembered && models.includes(remembered)) {
        selectedModel = remembered;
      }
    } catch {
      models = [];
      modelsError = "Ollama unreachable — click to retry";
    } finally {
      modelsLoaded = true;
    }
  }

  function persistModel() {
    if (selectedModel) localStorage.setItem(MODEL_STORAGE_KEY, selectedModel);
  }

  function toggleModelMenu(e: MouseEvent) {
    e.stopPropagation();
    if (modelsError) {
      loadModels();
      return;
    }
    modelMenuOpen = !modelMenuOpen;
  }

  function chooseModel(m: string) {
    selectedModel = m;
    persistModel();
    modelMenuOpen = false;
  }

  function toggleNewChatMenu(e: MouseEvent) {
    e.stopPropagation();
    if (messages.length === 0) return;
    newChatMenuOpen = !newChatMenuOpen;
  }

  function newChat() {
    if (busy || summarizing) return;
    newChatMenuOpen = false;
    messages = [];
    carriedSummary = "";
    summaryError = "";
    tick().then(() => inputEl?.focus());
  }

  /** Streams one request to completion and resolves with the full text, without touching `messages`. */
  function runOnce(model: string, question: string, context: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const requestId = crypto.randomUUID();
      let text = "";
      let unToken: () => void = () => {};
      let unDone: () => void = () => {};
      let unErr: () => void = () => {};

      Promise.all([
        onEvent<string>(`ai:${requestId}:token`, (t) => {
          text += t;
        }),
        onEvent<void>(`ai:${requestId}:done`, () => {
          unToken();
          unDone();
          unErr();
          resolve(text);
        }),
        onEvent<string>(`ai:${requestId}:error`, (e) => {
          unToken();
          unDone();
          unErr();
          reject(new Error(e));
        }),
      ]).then(([t, d, e]) => {
        unToken = t;
        unDone = d;
        unErr = e;
        api.askAi(requestId, model, question, context).catch(reject);
      });
    });
  }

  async function newChatFromSummary() {
    if (busy || summarizing || !selectedModel || messages.length === 0) return;
    newChatMenuOpen = false;
    summarizing = true;
    summaryError = "";
    const transcript = messages
      .filter((m) => m.text.trim())
      .map((m) => `${m.role === "user" ? "User" : "Assistant"}: ${m.text}`)
      .join("\n\n");
    const prompt =
      "Summarize the key points, conclusions, and any commands or findings from this conversation " +
      "in 2-4 concise sentences, so it can be carried into a new chat as context. Reply with only " +
      `the summary, no preamble.\n\n${transcript}`;
    try {
      const summary = await runOnce(selectedModel, prompt, "");
      carriedSummary = summary.trim();
      messages = [];
      scrollToBottom();
      tick().then(() => inputEl?.focus());
    } catch (e) {
      summaryError = `Couldn't summarize: ${e}`;
    } finally {
      summarizing = false;
    }
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
    contextPollTimer = setInterval(() => {
      if (open) refreshContextPreview();
    }, 2000);
  });

  onDestroy(() => {
    if (contextPollTimer) clearInterval(contextPollTimer);
  });

  $effect(() => {
    if (open) {
      tick().then(() => inputEl?.focus());
      refreshContextPreview();
    }
  });

  async function ask() {
    if (!enabled || busy || summarizing || !selectedModel || !input.trim()) return;
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
    const liveContext = contextEnabled ? getContext() : "";
    const context = carriedSummary
      ? `Summary of earlier conversation:\n${carriedSummary}\n\n${liveContext}`
      : liveContext;

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

<svelte:window
  onclick={() => {
    modelMenuOpen = false;
    newChatMenuOpen = false;
  }}
/>

{#if open}
  <div class="panel">
    <div class="header">
      <span class="title"
        >Cyber Bro {#if !enabled}<span class="off">(disabled)</span>{/if}</span
      >
      {#if enabled}
        <div class="new-chat-picker">
          <button
            class="icon-btn"
            onclick={messages.length > 0 ? toggleNewChatMenu : newChat}
            disabled={busy || summarizing || messages.length === 0}
            title="New chat"
          >
            {#if summarizing}
              <span class="spinner muted"></span>
            {:else}
              <Icon name="plus" />
            {/if}
          </button>
          {#if newChatMenuOpen}
            <div class="new-chat-menu">
              <button class="menu-option" onclick={newChat}>New chat</button>
              <button class="menu-option" onclick={newChatFromSummary} disabled={!selectedModel}>
                New chat from summary
              </button>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="messages" bind:this={messagesEl}>
      {#if !enabled}
        <div class="hint">AI suggestions are off. Toggle them from the status bar.</div>
      {:else if !selectedModel}
        <div class="hint">
          Pick a model below, then ask about the active terminal's output or anything else.
        </div>
      {:else if messages.length === 0 && !carriedSummary}
        <div class="hint">Ask about the active terminal's output, a tool, or anything else.</div>
      {/if}
      {#if summaryError}
        <div class="hint error">{summaryError}</div>
      {/if}
      {#if carriedSummary}
        <div class="summary-banner">
          <Icon name="rotate-ccw" size={11} />
          <span>{carriedSummary}</span>
        </div>
      {/if}
      {#each messages as m, i (m.id)}
        {#if m.role === "user"}
          <div class="msg user">
            <div class="text">
              {#each segments(m.text) as seg, si (si)}
                {#if seg.kind === "code"}
                  <pre class="code">{seg.content}</pre>
                {:else}
                  {seg.content}
                {/if}
              {/each}
            </div>
          </div>
        {:else}
          <div class="msg assistant">
            {#if !m.text && busy && i === messages.length - 1}
              <div class="thinking"><span></span><span></span><span></span></div>
            {:else}
              {#if m.text}
                <button class="copy-btn" onclick={() => copyMessage(m)} title="Copy response">
                  <Icon name={copiedId === m.id ? "check" : "copy"} size={12} />
                </button>
              {/if}
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
        {/if}
      {/each}
    </div>

    <div class="composer">
      <textarea
        bind:this={inputEl}
        bind:value={input}
        onkeydown={onKeydown}
        oninput={autosize}
        disabled={!enabled || !selectedModel || summarizing}
        placeholder={!enabled
          ? "AI is disabled"
          : !selectedModel
            ? "Choose a model below first"
            : "Ask Cyber Bro… (Shift+Enter for a new line)"}
        rows="1"></textarea>
      <div class="composer-toolbar">
        <div class="toolbar-left">
          {#if enabled && selectedModel}
            <button
              class="context-chip"
              class:off={!contextEnabled}
              onclick={() => (contextEnabled = !contextEnabled)}
              title={contextEnabled
                ? "Terminal output is attached as context — click to exclude it from the next message"
                : "Terminal context excluded — click to include it again"}
            >
              <span class="context-dot"></span>
              {contextEnabled && contextLines > 0 ? `Terminal · ${contextLines}` : "Terminal"}
            </button>
          {/if}
        </div>
        <div class="toolbar-right">
          {#if enabled && modelsLoaded}
            <div class="model-picker">
              <button
                class="model-btn"
                class:error={!!modelsError}
                onclick={toggleModelMenu}
                title={modelsError || "Choose Ollama model"}
              >
                <span class="model-btn-label">{modelsError || selectedModel || "choose a model…"}</span>
                {#if !modelsError}
                  <span class="chevron" class:open={modelMenuOpen}
                    ><Icon name="chevron-down" size={11} /></span
                  >
                {/if}
              </button>
              {#if modelMenuOpen && !modelsError}
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
          {/if}
          <button
            class="send-btn"
            onclick={ask}
            disabled={!enabled || !selectedModel || busy || summarizing || !input.trim()}
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

  /* header: title + new chat only, mirroring Zed's minimal toolbar */
  .header {
    height: 34px;
    flex-shrink: 0;
    padding: 0 8px 0 12px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
  }
  .title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-dim);
  }
  .off {
    color: var(--text-faint);
    font-weight: 400;
  }
  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    background: transparent;
    border: none;
    color: var(--text-faint);
    padding: 0;
    border-radius: 4px;
  }
  .icon-btn:hover:not(:disabled) {
    background: var(--border-soft);
    color: var(--text);
  }
  .icon-btn:disabled {
    opacity: 0.4;
  }

  .new-chat-picker {
    position: relative;
  }
  .new-chat-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    min-width: 170px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    padding: 4px;
    z-index: 30;
  }
  .menu-option {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text);
    font-size: 11.5px;
    font-weight: 400;
    padding: 6px 8px;
    border-radius: 5px;
    white-space: nowrap;
  }
  .menu-option:hover:not(:disabled) {
    background: var(--border-soft);
  }
  .menu-option:disabled {
    color: var(--text-faint);
    opacity: 0.5;
  }

  .summary-banner {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    margin: 4px 12px 8px;
    padding: 8px 10px;
    background: var(--border-soft);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-dim);
    font-size: 11.5px;
    line-height: 1.5;
  }
  .summary-banner :global(svg) {
    flex-shrink: 0;
    margin-top: 2px;
    color: var(--text-faint);
  }

  .context-chip {
    display: flex;
    align-items: center;
    gap: 5px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 5px;
    color: var(--text-faint);
    font-size: 10.5px;
    padding: 3px 6px;
  }
  .context-chip:hover {
    background: var(--border-soft);
    color: var(--text-dim);
  }
  .context-chip.off {
    text-decoration: line-through;
    opacity: 0.6;
  }
  .context-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
    flex-shrink: 0;
  }
  .context-chip.off .context-dot {
    background: var(--text-faint);
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .hint {
    padding: 4px 20px;
    color: var(--text-faint);
    font-size: 12px;
    line-height: 1.5;
  }

  /* user message: boxed card, like Zed's editor_background card */
  .msg.user {
    margin: 4px 8px;
    padding: 12px 8px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .msg.user .text {
    color: var(--text);
  }

  /* assistant message: bare flowing text, no box, wider gutter */
  .msg.assistant {
    position: relative;
    padding: 6px 20px;
  }
  .msg.assistant .text {
    color: var(--text);
  }
  .msg.assistant .copy-btn {
    position: absolute;
    top: 4px;
    right: 6px;
    opacity: 0;
    transition: opacity 0.1s;
  }
  .msg.assistant:hover .copy-btn {
    opacity: 1;
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
  }
  .copy-btn:hover {
    color: var(--text);
    background: var(--border-soft);
  }

  .text {
    font-size: 12.5px;
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .code {
    display: block;
    margin: 4px 0;
    padding: 8px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--text);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-x: auto;
  }
  .msg.user .code {
    background: var(--bg-elevated);
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

  /* composer: single bordered container holding the editor + its own
     toolbar row (model picker + send), mirroring Zed's message editor. */
  .composer {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 8px;
    padding: 8px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  textarea {
    resize: none;
    background: transparent;
    border: none;
    color: var(--text);
    padding: 2px 2px 0;
    font-family: var(--font-ui);
    font-size: 12.5px;
    line-height: 1.4;
    max-height: 120px;
    overflow-y: auto;
    box-sizing: border-box;
  }
  textarea:focus {
    outline: none;
  }
  .composer-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .toolbar-left {
    flex: 1;
  }
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .model-picker {
    position: relative;
  }
  .model-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    max-width: 190px;
    background: var(--border-soft);
    border: 1px solid transparent;
    border-radius: 5px;
    color: var(--text-dim);
    font-size: 11px;
    font-weight: 400;
    letter-spacing: normal;
    padding: 3px 6px;
  }
  .model-btn:hover {
    color: var(--text);
    background: var(--border);
  }
  .model-btn.error {
    color: var(--danger);
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
    bottom: calc(100% + 6px);
    right: 0;
    min-width: 170px;
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

  .send-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    background: var(--accent-dim);
    color: #06170a;
    border: none;
    border-radius: 5px;
  }
  .send-btn:hover:not(:disabled) {
    background: var(--accent);
  }
  .send-btn:disabled {
    background: var(--border-soft);
    color: var(--text-faint);
  }
  .spinner {
    width: 11px;
    height: 11px;
    border-radius: 50%;
    border: 2px solid rgba(6, 23, 10, 0.35);
    border-top-color: #06170a;
    animation: spin 0.7s linear infinite;
  }
  .send-btn:disabled .spinner {
    border-color: rgba(255, 255, 255, 0.15);
    border-top-color: var(--text-faint);
  }
  .spinner.muted {
    border-color: var(--border);
    border-top-color: var(--text-faint);
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
