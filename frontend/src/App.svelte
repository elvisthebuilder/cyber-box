<script lang="ts">
  import { onMount } from "svelte";
  import TitleBar from "./lib/TitleBar.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import TabBar from "./lib/TabBar.svelte";
  import TerminalView from "./lib/TerminalView.svelte";
  import Editor from "./lib/Editor.svelte";
  import AiPanel from "./lib/AiPanel.svelte";
  import StatusBar from "./lib/StatusBar.svelte";
  import { api, b64encode, type Tab } from "./lib/api";

  const initialTabId: string = crypto.randomUUID();
  let tabs = $state<Tab[]>([{ id: initialTabId, title: "bash", kind: "terminal" }]);
  let activeId = $state<string>(initialTabId);
  let termRefs: Record<string, { getBufferText: (n?: number) => string }> = {};
  let dirtyTabs = $state<Record<string, boolean>>({});

  let containerUp = $state(false);
  let torOn = $state(false);
  let aiEnabled = $state(true);
  let aiOpen = $state(false);

  function newTab() {
    const id = crypto.randomUUID();
    tabs = [...tabs, { id, title: "bash", kind: "terminal" }];
    activeId = id;
  }

  function openFile(path: string) {
    const existing = tabs.find((t) => t.kind === "editor" && t.path === path);
    if (existing) {
      activeId = existing.id;
      return;
    }
    const id = crypto.randomUUID();
    const title = path.split("/").pop() || path;
    tabs = [...tabs, { id, title, kind: "editor", path }];
    activeId = id;
  }

  function closeTab(id: string) {
    if (tabs.length <= 1) return;
    const idx = tabs.findIndex((t) => t.id === id);
    tabs = tabs.filter((t) => t.id !== id);
    delete termRefs[id];
    delete dirtyTabs[id];
    if (activeId === id) {
      activeId = tabs[Math.max(0, idx - 1)].id;
    }
  }

  function insertText(text: string, run: boolean) {
    const bytes = new TextEncoder().encode(run ? `${text}\n` : text);
    api.ptyWrite(activeId, b64encode(bytes));
  }

  async function refreshStatus() {
    containerUp = await api.containerStatus().catch(() => false);
    torOn = await api.torStatus().catch(() => false);
  }

  async function toggleTor() {
    torOn = await api.toggleTor(!torOn).catch(() => torOn);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key.toLowerCase() === "i") {
      e.preventDefault();
      aiOpen = !aiOpen;
    }
  }

  const displayTabs = $derived(
    tabs.map((t) => (t.kind === "editor" && dirtyTabs[t.id] ? { ...t, title: `${t.title} ●` } : t)),
  );

  onMount(() => {
    refreshStatus();
    const interval = setInterval(refreshStatus, 2000);
    window.addEventListener("keydown", onKeydown);
    return () => {
      clearInterval(interval);
      window.removeEventListener("keydown", onKeydown);
    };
  });
</script>

<div class="shell">
  <TitleBar />
  <div class="body">
    <Sidebar onInsertText={insertText} onOpenFile={openFile} activeTabId={activeId} />
    <div class="main">
      <TabBar
        tabs={displayTabs}
        {activeId}
        onSelect={(id) => (activeId = id)}
        onClose={closeTab}
        onNew={newTab}
      />
      <div class="term-stack">
        {#each tabs as tab (tab.id)}
          {#if tab.kind === "editor" && tab.path}
            <Editor
              id={tab.id}
              path={tab.path}
              active={tab.id === activeId}
              onDirtyChange={(dirty) => (dirtyTabs[tab.id] = dirty)}
            />
          {:else}
            <TerminalView id={tab.id} active={tab.id === activeId} bind:this={termRefs[tab.id]} />
          {/if}
        {/each}
      </div>
      <AiPanel
        open={aiOpen}
        enabled={aiEnabled}
        activeIsTerminal={tabs.find((t) => t.id === activeId)?.kind === "terminal"}
        getContext={() => termRefs[activeId]?.getBufferText() ?? ""}
      />
    </div>
  </div>
  <StatusBar
    {containerUp}
    {torOn}
    {aiOpen}
    {aiEnabled}
    onToggleTor={toggleTor}
    onToggleAiPanel={() => (aiOpen = !aiOpen)}
    onToggleAiEnabled={() => (aiEnabled = !aiEnabled)}
  />
</div>

<style>
  .shell {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-radius: var(--radius);
    overflow: hidden;
    border: 1px solid var(--border);
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    position: relative;
  }
  .term-stack {
    flex: 1;
    position: relative;
    min-height: 0;
  }
</style>
