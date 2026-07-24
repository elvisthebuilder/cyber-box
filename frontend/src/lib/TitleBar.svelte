<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const win = getCurrentWindow();

  function minimize() {
    win.minimize();
  }
  function toggleMaximize() {
    win.toggleMaximize();
  }
  function close() {
    win.close();
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <span class="dot"></span>
    <span class="name">cyber-box</span>
  </div>
  <div class="spacer" data-tauri-drag-region></div>
  <div class="controls">
    <button class="ctl" onclick={minimize} title="Minimize">&#x2013;</button>
    <button class="ctl" onclick={toggleMaximize} title="Maximize">&#x25A1;</button>
    <button class="ctl close" onclick={close} title="Close">&#x2715;</button>
  </div>
</div>

<style>
  .titlebar {
    height: 38px;
    display: flex;
    align-items: center;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    padding: 0 10px;
    flex-shrink: 0;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-dim);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent);
  }
  .spacer {
    flex: 1;
  }
  .controls {
    display: flex;
    gap: 4px;
  }
  .ctl {
    width: 28px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--text-dim);
    border-radius: 6px;
    font-size: 13px;
    line-height: 1;
  }
  .ctl:hover {
    background: var(--border-soft);
    color: var(--text);
  }
  .ctl.close:hover {
    background: var(--danger);
    color: white;
  }
</style>
