<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { EditorState, Compartment } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, indentOnInput } from "@codemirror/language";
  import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
  import { closeBrackets, closeBracketsKeymap, autocompletion, completionKeymap } from "@codemirror/autocomplete";
  import { languages } from "@codemirror/language-data";
  import { api, b64decode, b64encode } from "./api";

  let {
    id,
    path,
    active,
    onDirtyChange,
  }: { id: string; path: string; active: boolean; onDirtyChange?: (dirty: boolean) => void } = $props();

  let container: HTMLDivElement;
  let view: EditorView;
  let langCompartment = new Compartment();
  let savedDoc = "";
  let status = $state<"loading" | "ready" | "saving" | "error">("loading");
  let errorText = $state("");

  const theme = EditorView.theme(
    {
      "&": { height: "100%", fontSize: "13px", backgroundColor: "var(--bg)", color: "var(--text)" },
      ".cm-content": { fontFamily: "var(--font-mono)", caretColor: "var(--accent)" },
      ".cm-cursor": { borderLeftColor: "var(--accent)" },
      ".cm-gutters": { backgroundColor: "var(--bg)", color: "var(--text-faint)", border: "none" },
      ".cm-activeLine": { backgroundColor: "var(--border-soft)" },
      ".cm-activeLineGutter": { backgroundColor: "var(--border-soft)" },
      ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": { backgroundColor: "#2ea04355 !important" },
      ".cm-scroller": { fontFamily: "var(--font-mono)", overflow: "auto" },
      "&.cm-focused": { outline: "none" },
    },
    { dark: true },
  );

  function isDirty(): boolean {
    return view ? view.state.doc.toString() !== savedDoc : false;
  }

  async function save() {
    if (!view || status === "saving") return;
    status = "saving";
    try {
      const text = view.state.doc.toString();
      await api.writeFile(path, b64encode(new TextEncoder().encode(text)));
      savedDoc = text;
      status = "ready";
      onDirtyChange?.(false);
    } catch (e) {
      status = "error";
      errorText = String(e);
    }
  }

  onMount(async () => {
    const startState = EditorState.create({
      doc: "",
      extensions: [
        lineNumbers(),
        highlightActiveLine(),
        highlightActiveLineGutter(),
        history(),
        bracketMatching(),
        closeBrackets(),
        indentOnInput(),
        autocompletion(),
        highlightSelectionMatches(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        langCompartment.of([]),
        keymap.of([
          { key: "Mod-s", preventDefault: true, run: () => (save(), true) },
          indentWithTab,
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...searchKeymap,
          ...historyKeymap,
          ...completionKeymap,
        ]),
        theme,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) onDirtyChange?.(isDirty());
        }),
      ],
    });
    view = new EditorView({ state: startState, parent: container });

    try {
      const b64 = await api.readFile(path);
      const text = new TextDecoder().decode(b64decode(b64));
      savedDoc = text;
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text } });
      status = "ready";
    } catch (e) {
      status = "error";
      errorText = String(e);
    }

    const lang = languages.find((l) => l.extensions.some((ext) => path.toLowerCase().endsWith(`.${ext}`)));
    if (lang) {
      const support = await lang.load().catch(() => null);
      if (support) view.dispatch({ effects: langCompartment.reconfigure(support) });
    }
  });

  onDestroy(() => view?.destroy());

  $effect(() => {
    if (active) view?.focus();
  });
</script>

<div class="editor-wrap" class:hidden={!active}>
  {#if status === "loading"}
    <div class="center-msg">Loading {path}…</div>
  {:else if status === "error"}
    <div class="center-msg error">{errorText}</div>
  {/if}
  <div class="cm-host" class:invisible={status !== "ready" && status !== "saving"} bind:this={container}></div>
</div>

<style>
  .editor-wrap {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    user-select: text;
  }
  .hidden {
    visibility: hidden;
    pointer-events: none;
  }
  .cm-host {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .cm-host :global(.cm-editor) {
    height: 100%;
  }
  .invisible {
    display: none;
  }
  .center-msg {
    padding: 16px;
    color: var(--text-faint);
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .center-msg.error {
    color: var(--danger);
  }
</style>
