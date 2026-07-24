<script lang="ts">
  import type { Tab } from "./api";

  let {
    tabs,
    activeId,
    onSelect,
    onClose,
    onNew,
  }: {
    tabs: Tab[];
    activeId: string;
    onSelect: (id: string) => void;
    onClose: (id: string) => void;
    onNew: () => void;
  } = $props();
</script>

<div class="tabbar">
  {#each tabs as tab (tab.id)}
    <button class="tab" class:active={tab.id === activeId} onclick={() => onSelect(tab.id)}>
      <span class="title">{tab.title}</span>
      {#if tabs.length > 1}
        <span
          class="close"
          role="button"
          tabindex="0"
          onclick={(e) => {
            e.stopPropagation();
            onClose(tab.id);
          }}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.stopPropagation();
              e.preventDefault();
              onClose(tab.id);
            }
          }}>&#x2715;</span
        >
      {/if}
    </button>
  {/each}
  <button class="new" onclick={onNew} title="New terminal">+</button>
</div>

<style>
  .tabbar {
    display: flex;
    align-items: center;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    padding: 4px 6px 0 6px;
    gap: 4px;
    flex-shrink: 0;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: 1px solid transparent;
    border-bottom: none;
    color: var(--text-dim);
    padding: 6px 10px;
    border-radius: 8px 8px 0 0;
    font-size: 12px;
  }
  .tab.active {
    background: var(--bg);
    color: var(--text);
    border-color: var(--border);
  }
  .close {
    opacity: 0.6;
    font-size: 10px;
    padding: 2px;
    border-radius: 4px;
  }
  .close:hover {
    opacity: 1;
    background: var(--border-soft);
  }
  .new {
    background: transparent;
    border: none;
    color: var(--text-faint);
    font-size: 15px;
    padding: 4px 10px;
    border-radius: 6px;
  }
  .new:hover {
    background: var(--border-soft);
    color: var(--text);
  }
</style>
