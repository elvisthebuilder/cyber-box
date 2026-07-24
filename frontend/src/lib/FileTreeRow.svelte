<script lang="ts">
  import { fileIcon, folderIcon } from "./fileIcons";
  import { listChildren, type FileNode } from "./fileTree";
  import Self from "./FileTreeRow.svelte";

  let {
    node,
    depth = 0,
    onFileClick,
  }: { node: FileNode; depth?: number; onFileClick: (node: FileNode) => void } = $props();

  async function toggle() {
    if (!node.isDir) {
      onFileClick(node);
      return;
    }
    node.expanded = !node.expanded;
    if (node.expanded && !node.children) {
      node.children = await listChildren(node);
    }
  }

  const icon = $derived(node.isDir ? folderIcon(node.expanded) : fileIcon(node.name));
</script>

<button class="row" style="padding-left: {6 + depth * 12}px" onclick={toggle} title={node.name}>
  <span class="chevron" class:open={node.expanded} class:invisible={!node.isDir}>&#9656;</span>
  <!-- eslint-disable-next-line svelte/no-at-html-tags -- icon.svg comes from the fixed local icon map in fileIcons.ts, not user input -->
  <span class="icon" style="color: {icon.color}">{@html icon.svg}</span>
  <span class="name">{node.name}</span>
</button>

{#if node.isDir && node.expanded}
  {#each node.children ?? [] as child (child.path)}
    <Self node={child} depth={depth + 1} {onFileClick} />
  {/each}
{/if}

<style>
  .row {
    width: 100%;
    height: 24px;
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    color: var(--text);
    text-align: left;
    font-size: 13px;
    padding-right: 8px;
  }
  .row:hover {
    background: var(--border-soft);
  }
  .chevron {
    width: 10px;
    flex-shrink: 0;
    font-size: 9px;
    color: var(--text-faint);
    transition: transform 0.1s;
  }
  .chevron.open {
    transform: rotate(90deg);
  }
  .chevron.invisible {
    visibility: hidden;
  }
  .icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .icon :global(svg) {
    width: 16px;
    height: 16px;
  }
  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
