<script lang="ts">
  let {
    containerUp,
    torOn,
    aiOpen,
    aiEnabled,
    onToggleTor,
    onToggleAiPanel,
    onToggleAiEnabled,
  }: {
    containerUp: boolean;
    torOn: boolean;
    aiOpen: boolean;
    aiEnabled: boolean;
    onToggleTor: () => void;
    onToggleAiPanel: () => void;
    onToggleAiEnabled: () => void;
  } = $props();
</script>

<div class="statusbar">
  <div class="left">
    <span class="pill">
      <span class="dot" class:up={containerUp} class:down={!containerUp}></span>
      {containerUp ? "container up" : "container down"}
    </span>
    <button class="pill clickable" class:active={torOn} onclick={onToggleTor}>
      <span class="dot" class:up={torOn}></span>
      TOR {torOn ? "ON" : "OFF"}
    </button>
    <button class="pill clickable" class:active={aiEnabled} onclick={onToggleAiEnabled}>
      AI {aiEnabled ? "ON" : "OFF"}
    </button>
  </div>
  <div class="spacer"></div>
  <button class="ai-btn" onclick={onToggleAiPanel} class:active={aiOpen}>
    Open AI agent <span class="kbd">Ctrl+I</span>
  </button>
</div>

<style>
  .statusbar {
    height: 30px;
    display: flex;
    align-items: center;
    background: var(--bg-elevated);
    border-top: 1px solid var(--border);
    padding: 0 10px;
    gap: 8px;
    flex-shrink: 0;
    font-size: 11px;
  }
  .left {
    display: flex;
    gap: 8px;
  }
  .spacer {
    flex: 1;
  }
  .pill {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: 999px;
    background: transparent;
    border: none;
    color: var(--text-dim);
  }
  .pill.clickable {
    cursor: pointer;
  }
  .pill.clickable:hover {
    background: var(--border-soft);
  }
  .pill.active {
    color: var(--accent);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-faint);
  }
  .dot.up {
    background: var(--accent);
  }
  .dot.down {
    background: var(--danger);
  }
  .ai-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--border-soft);
    border: 1px solid var(--border);
    color: var(--text-dim);
    padding: 3px 10px;
    border-radius: 999px;
  }
  .ai-btn.active {
    color: var(--accent);
    border-color: var(--accent-dim);
  }
  .kbd {
    font-size: 9px;
    color: var(--text-faint);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 4px;
  }
</style>
