<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "./Icon.svelte";

  let { sidebarCollapsed, onToggleSidebar }: { sidebarCollapsed: boolean; onToggleSidebar: () => void } =
    $props();

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
  <div class="brand-col" data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <span class="logo"><Icon name="shield" size={15} /></span>
      <span class="name">Cyber Box</span>
    </div>
    <button
      class="collapse-btn"
      onclick={onToggleSidebar}
      title={sidebarCollapsed ? "Show sidebar" : "Hide sidebar"}
    >
      <Icon name="panel-left" size={13} />
    </button>
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
  .brand-col {
    width: 260px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-dim);
  }
  .logo {
    display: inline-flex;
    color: var(--accent);
  }
  .collapse-btn {
    width: 24px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    color: var(--text-faint);
    border-radius: 6px;
    flex-shrink: 0;
    margin-left: 12px;
  }
  .collapse-btn:hover {
    background: var(--border-soft);
    color: var(--text);
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
