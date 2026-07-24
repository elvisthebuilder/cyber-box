<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";
  import { api, b64decode, b64encode, onEvent } from "./api";

  let { id, active }: { id: string; active: boolean } = $props();

  let container: HTMLDivElement;
  let term: Terminal;
  let fit: FitAddon;
  let unlistenData: () => void;
  let unlistenExit: () => void;
  let resizeObserver: ResizeObserver;

  onMount(async () => {
    term = new Terminal({
      fontFamily: "var(--font-mono)",
      fontSize: 13,
      cursorBlink: true,
      theme: {
        background: "#0d1117",
        foreground: "#c9d1d9",
        cursor: "#39d353",
        selectionBackground: "#2ea04355",
      },
    });
    fit = new FitAddon();
    term.loadAddon(fit);
    term.open(container);
    fit.fit();

    term.onData((data) => {
      api.ptyWrite(id, b64encode(new TextEncoder().encode(data)));
    });

    unlistenData = await onEvent<string>(`pty:${id}:data`, (payload) => {
      term.write(b64decode(payload));
    });
    unlistenExit = await onEvent<void>(`pty:${id}:exit`, () => {
      term.write("\r\n\x1b[90m[session ended]\x1b[0m\r\n");
    });

    resizeObserver = new ResizeObserver(() => {
      fit.fit();
      api.ptyResize(id, term.cols, term.rows).catch(() => {});
    });
    resizeObserver.observe(container);

    await api.ptyOpen(id);
    await api.ptyResize(id, term.cols, term.rows).catch(() => {});
  });

  onDestroy(() => {
    unlistenData?.();
    unlistenExit?.();
    resizeObserver?.disconnect();
    api.ptyClose(id).catch(() => {});
    term?.dispose();
  });

  $effect(() => {
    if (active) {
      fit?.fit();
      term?.focus();
    }
  });

  export function getBufferText(maxLines = 200): string {
    if (!term) return "";
    const buf = term.buffer.active;
    const start = Math.max(0, buf.length - maxLines);
    const lines: string[] = [];
    for (let i = start; i < buf.length; i++) {
      const line = buf.getLine(i);
      if (line) lines.push(line.translateToString(true));
    }
    return lines.join("\n");
  }
</script>

<div class="term-wrap" class:hidden={!active} bind:this={container}></div>

<style>
  .term-wrap {
    position: absolute;
    inset: 0;
    padding: 8px 4px 4px 10px;
  }
  .hidden {
    visibility: hidden;
    pointer-events: none;
  }
</style>
