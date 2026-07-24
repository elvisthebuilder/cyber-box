<script lang="ts">
  import { onMount } from "svelte";
  import { api, onEvent, type ToolInfo } from "./api";
  import { listChildren, type FileNode } from "./fileTree";
  import FileTreeRow from "./FileTreeRow.svelte";

  let {
    onInsertText,
    onOpenFile,
    activeTabId,
  }: {
    onInsertText: (text: string, run: boolean) => void;
    onOpenFile: (path: string) => void;
    activeTabId: string;
  } = $props();

  let tools = $state<ToolInfo[]>([]);
  let activeTab = $state<"tools" | "files">("tools");
  let expandedCategories = $state<Set<string>>(new Set());
  let installingLog = $state<Record<string, string[]>>({});

  const unlisteners: (() => void)[] = [];

  function categories() {
    const map = new Map<string, ToolInfo[]>();
    for (const t of tools) {
      if (!map.has(t.category)) map.set(t.category, []);
      map.get(t.category)!.push(t);
    }
    return [...map.entries()].sort(([a], [b]) => a.localeCompare(b));
  }

  function toggleCategory(cat: string) {
    const next = new Set(expandedCategories);
    next.has(cat) ? next.delete(cat) : next.add(cat);
    expandedCategories = next;
  }

  async function loadTools() {
    await api.refreshInstallStatus().catch(() => {});
    tools = await api.listTools().catch(() => []);
    expandedCategories = new Set(categories().map(([c]) => c));
  }

  async function onToolClick(tool: ToolInfo) {
    if (tool.install_status.kind === "Installed" || tool.install_status.kind === "Unknown") {
      const cmd = tool.invocation.replace("{flags}", tool.default_flags).replace("{target}", "TARGET");
      onInsertText(cmd, false);
      return;
    }
    if (tool.install_status.kind === "NotInstalled" || tool.install_status.kind === "Failed") {
      installingLog[tool.name] = [];
      const unData = await onEvent<string>(`install:${tool.name}:line`, (line) => {
        installingLog[tool.name] = [...(installingLog[tool.name] ?? []), line];
      });
      const unDone = await onEvent<boolean>(`install:${tool.name}:done`, async () => {
        unData();
        unDone();
        await loadTools();
      });
      api.installTool(tool.name).catch(() => {});
    }
  }

  function badgeFor(tool: ToolInfo): { text: string; cls: string } {
    switch (tool.install_status.kind) {
      case "NotInstalled":
        return { text: "install", cls: "badge-missing" };
      case "Installing":
        return { text: "installing…", cls: "badge-installing" };
      case "Failed":
        return { text: "failed", cls: "badge-failed" };
      default:
        return { text: "", cls: "" };
    }
  }

  // --- filesystem browser: follows the active terminal's live cwd ---
  let cwd = $state("/");
  let fsRoot = $state<FileNode>({ path: "/", name: "/", isDir: true, expanded: true });
  let cwdPollTimer: ReturnType<typeof setInterval> | undefined;

  /** Re-lists `node`'s children in place if expanded, preserving the
   * expanded/children state of any subfolders that still exist, then
   * recurses into them — so a poll never collapses what's open. */
  async function refreshNode(node: FileNode) {
    if (!node.isDir || !node.expanded) return;
    const fresh = await listChildren(node);
    const existingByPath = new Map((node.children ?? []).map((c) => [c.path, c]));
    node.children = fresh.map((f) => {
      const existing = existingByPath.get(f.path);
      if (existing) {
        existing.name = f.name;
        existing.isDir = f.isDir;
        return existing;
      }
      return f;
    });
    for (const child of node.children) {
      await refreshNode(child);
    }
  }

  async function refreshCwd() {
    const next = await api.ptyCwd(activeTabId).catch(() => null);
    if (!next) return;
    if (next !== cwd) {
      cwd = next;
      const root: FileNode = { path: next, name: next, isDir: true, expanded: true };
      root.children = await listChildren(root);
      fsRoot = root;
    } else {
      await refreshNode(fsRoot);
    }
  }

  function onFileClick(node: FileNode) {
    onOpenFile(node.path);
  }

  $effect(() => {
    // Re-poll whenever the active tab changes; keep polling while the Files
    // tab is visible so `cd` in the terminal is reflected without a manual
    // refresh.
    void activeTabId;
    refreshCwd();
  });

  onMount(() => {
    loadTools();
    refreshCwd();
    cwdPollTimer = setInterval(() => {
      if (activeTab === "files") refreshCwd();
    }, 1000);
    return () => {
      unlisteners.forEach((u) => u());
      if (cwdPollTimer) clearInterval(cwdPollTimer);
    };
  });
</script>

<div class="sidebar">
  <div class="chips">
    <button class="chip" class:active={activeTab === "tools"} onclick={() => (activeTab = "tools")}>
      Tools
    </button>
    <button class="chip" class:active={activeTab === "files"} onclick={() => (activeTab = "files")}>
      Files
    </button>
  </div>

  {#if activeTab === "tools"}
    <div class="section-body">
      {#each categories() as [cat, catTools] (cat)}
        <button class="category" onclick={() => toggleCategory(cat)}>
          <span class="chevron" class:open={expandedCategories.has(cat)}>&#9656;</span>
          {cat}
        </button>
        {#if expandedCategories.has(cat)}
          {#each catTools as tool (tool.name)}
            {@const badge = badgeFor(tool)}
            <button class="tool-row" onclick={() => onToolClick(tool)} title={tool.description}>
              <span class="tool-name" class:dim={badge.cls !== ""}>{tool.name}</span>
              {#if badge.text}
                <span class="badge {badge.cls}">{badge.text}</span>
              {/if}
            </button>
            {#if installingLog[tool.name]?.length}
              <div class="install-log">
                {#each installingLog[tool.name].slice(-3) as line}
                  <div class="log-line">{line}</div>
                {/each}
              </div>
            {/if}
          {/each}
        {/if}
      {/each}
    </div>
  {:else}
    <div class="cwd-bar" title={cwd}>
      <span class="cwd-path">{cwd}</span>
      <button class="refresh" onclick={refreshCwd} title="Refresh">&#8635;</button>
    </div>
    <div class="section-body">
      {#each fsRoot.children ?? [] as node (node.path)}
        <FileTreeRow {node} depth={0} {onFileClick} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .sidebar {
    width: 260px;
    flex-shrink: 0;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    font-size: 12.5px;
    padding: 6px 0;
  }
  .chips {
    display: flex;
    gap: 6px;
    padding: 8px 10px;
    border-bottom: 1px solid var(--border-soft);
    margin-bottom: 4px;
  }
  .chip {
    flex: 1;
    background: var(--border-soft);
    border: 1px solid transparent;
    color: var(--text-dim);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    padding: 6px 0;
    border-radius: 999px;
    text-align: center;
  }
  .chip:hover {
    color: var(--text);
  }
  .chip.active {
    background: rgba(57, 211, 83, 0.12);
    border-color: var(--accent-dim);
    color: var(--accent);
  }
  .section-body {
    padding-bottom: 8px;
  }
  .cwd-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px 8px;
    color: var(--text-faint);
    font-family: var(--font-mono);
    font-size: 10.5px;
  }
  .cwd-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
  }
  .refresh {
    background: transparent;
    border: none;
    color: var(--text-faint);
    font-size: 12px;
    padding: 2px 4px;
    border-radius: 4px;
    flex-shrink: 0;
  }
  .refresh:hover {
    background: var(--border-soft);
    color: var(--text);
  }
  .chevron {
    display: inline-block;
    font-size: 9px;
    transition: transform 0.1s;
    color: var(--text-faint);
  }
  .chevron.open {
    transform: rotate(90deg);
  }
  .chevron.invisible {
    visibility: hidden;
  }
  .category {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text-dim);
    padding: 5px 12px;
    display: flex;
    align-items: center;
    gap: 6px;
    text-transform: capitalize;
    font-weight: 600;
  }
  .category:hover {
    background: var(--border-soft);
  }
  .tool-row {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text);
    padding: 4px 12px 4px 28px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }
  .tool-row:hover {
    background: var(--border-soft);
  }
  .tool-name.dim {
    color: var(--text-faint);
  }
  .badge {
    font-size: 9px;
    padding: 1px 6px;
    border-radius: 999px;
    flex-shrink: 0;
  }
  .badge-missing {
    background: #21262d;
    color: var(--text-dim);
    border: 1px solid var(--border);
  }
  .badge-installing {
    background: #3a2f00;
    color: var(--warn);
  }
  .badge-failed {
    background: #3a1414;
    color: var(--danger);
  }
  .install-log {
    padding: 2px 12px 4px 28px;
  }
  .log-line {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-faint);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
