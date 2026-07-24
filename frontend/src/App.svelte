<script lang="ts">
  import { onMount } from "svelte";
  import TitleBar from "./lib/TitleBar.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import TabBar from "./lib/TabBar.svelte";
  import TerminalView from "./lib/TerminalView.svelte";
  import AiPanel from "./lib/AiPanel.svelte";
  import StatusBar from "./lib/StatusBar.svelte";
  import { api, b64encode, type Tab } from "./lib/api";

  const initialTabId: string = crypto.randomUUID();
  let tabs = $state<Tab[]>([{ id: initialTabId, title: "bash" }]);
  let activeId = $state<string>(initialTabId);
  let termRefs: Record<string, { getBufferText: (n?: number) => string }> = {};

  let containerUp = $state(false);
  let torOn = $state(false);
  let aiEnabled = $state(true);
  let aiOpen = $state(false);

  function newTab() {
    const id = crypto.randomUUID();
    tabs = [...tabs, { id, title: "bash" }];
    activeId = id;
  }

  function closeTab(id: string) {
    if (tabs.length <= 1) return;
    const idx = tabs.findIndex((t) => t.id === id);
    tabs = tabs.filter((t) => t.id !== id);
    delete termRefs[id];
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
    <Sidebar onInsertText={insertText} activeTabId={activeId} />
    <div class="main">
      <TabBar {tabs} {activeId} onSelect={(id) => (activeId = id)} onClose={closeTab} onNew={newTab} />
      <div class="term-stack">
        {#each tabs as tab (tab.id)}
          <TerminalView id={tab.id} active={tab.id === activeId} bind:this={termRefs[tab.id]} />
        {/each}
      </div>
      <AiPanel open={aiOpen} enabled={aiEnabled} getContext={() => termRefs[activeId]?.getBufferText() ?? ""} />
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
