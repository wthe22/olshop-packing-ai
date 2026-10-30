<script lang="ts">
  // One label group of the rule, as a row of the rule's card: its code and name, its filter built
  // in the boxes editor or typed as text, the count it takes in the open batch, and the buttons to
  // move and delete it; the last group can instead take the rest. Every change recounts through
  // `onChange`; the entry itself is the entry the screen holds.
  import { texts } from "../texts";
  import BoxesEditor from "./BoxesEditor.svelte";
  import Icon from "./Icon.svelte";
  import type { Courier } from "./api/rules";
  import { boxesToFilter, textToTree, type GroupEntry } from "./groupEntry";
  import { newTree } from "./conditions";

  let {
    entry,
    draftBatch,
    couriers,
    canUp,
    canDown,
    last,
    onChange,
    onMove,
    onRemove,
  }: {
    entry: GroupEntry;
    // The batch being prepared, for its count; `null` when no draft is open.
    draftBatch: number | null;
    // The Courier box's list.
    couriers: Courier[];
    canUp: boolean;
    canDown: boolean;
    // The last entry of the list: the one that can take the rest.
    last: boolean;
    // A change landed: recount and re-check every entry (the screen does that).
    onChange: () => void;
    onMove: (delta: number) => void;
    onRemove: () => void;
  } = $props();

  // The filter language's examples, under a typed filter.
  let showHelp = $state(false);

  async function onBoxes() {
    if (await boxesToFilter(entry)) await onChange();
  }

  function onText() {
    entry.filter = entry.text;
    entry.boxesError = "";
    onChange();
  }

  async function toText() {
    if (entry.mode === "text") return;
    entry.text = entry.filter ?? "";
    entry.mode = "text";
    entry.boxesError = "";
    await onChange();
  }

  async function toBoxes() {
    if (entry.mode === "boxes") return;
    if (await textToTree(entry)) await onChange();
  }

  async function setRest(checked: boolean) {
    if (checked) {
      entry.filter = null;
      entry.boxesError = "";
      entry.error = "";
      onChange();
    } else {
      entry.filter = "";
      entry.text = "";
      entry.tree = newTree();
      entry.mode = "boxes";
      entry.boxesError = "";
      await onBoxes();
    }
  }
</script>

<div class="group">
  <div class="head">
    <input
      class="code"
      aria-label={texts.sortRules.code}
      bind:value={entry.code}
      oninput={onChange}
    />
    <input
      class="name"
      aria-label={texts.sortRules.name}
      bind:value={entry.name}
      oninput={onChange}
    />
    {#if entry.filter !== null}
      <div class="mode" role="group">
        <button
          class:chosen={entry.mode === "boxes"}
          disabled={entry.mode === "text" && entry.error !== ""}
          onclick={toBoxes}>{texts.sortRules.boxes}</button
        >
        <button class:chosen={entry.mode === "text"} onclick={toText}
          >{texts.sortRules.textMode}</button
        >
      </div>
    {/if}
    <span class="grow"></span>
    <span class="count">
      {#if entry.error}
        <span class="mark">✗</span>
      {:else if draftBatch !== null && entry.count !== null}
        {entry.count}
      {/if}
    </span>
    <span class="tools">
      <button
        class="icon"
        title={texts.sortRules.moveUp}
        aria-label={texts.sortRules.moveUp}
        disabled={!canUp}
        onclick={() => onMove(-1)}><Icon name="up" size={16} /></button
      >
      <button
        class="icon"
        title={texts.sortRules.moveDown}
        aria-label={texts.sortRules.moveDown}
        disabled={!canDown}
        onclick={() => onMove(1)}><Icon name="down" size={16} /></button
      >
      <button
        class="icon"
        title={texts.sortRules.delete}
        aria-label={texts.sortRules.delete}
        onclick={onRemove}><Icon name="trash" size={16} /></button
      >
    </span>
  </div>

  <div class="body">
    {#if entry.filter !== null && entry.mode === "boxes"}
      <BoxesEditor tree={entry.tree} {couriers} onEdit={onBoxes} />
    {:else if entry.filter !== null}
      <textarea class="filter-text" rows="3" bind:value={entry.text} oninput={onText}></textarea>
      <button class="link" onclick={() => (showHelp = !showHelp)}>{texts.sortRules.helpExamples}</button>
      {#if showHelp}
        <ul class="help">
          {#each texts.sortRules.examples as example (example)}
            <li><code>{example}</code></li>
          {/each}
        </ul>
      {/if}
    {/if}

    {#if entry.error}
      <p class="error">{entry.error}</p>
    {/if}

    {#if last}
      <label class="rest">
        <input
          type="checkbox"
          checked={entry.filter === null}
          onchange={(event) => setRest(event.currentTarget.checked)}
        />
        {texts.sortRules.takesRest}
      </label>
    {/if}
  </div>
</div>

<style>
  /* A row of the rule's card, under a thin rule. */
  .group {
    padding: 0.7rem 1rem 0.8rem;
    border-top: 1px solid var(--border-soft);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  /* The code as the group chips of the Plan and Day screens draw it. */
  .code {
    width: 3rem;
    flex: none;
    border-color: transparent;
    background: var(--chip);
    color: var(--chip-text);
    font-weight: 700;
    text-align: center;
  }

  .name {
    width: 18rem;
    min-width: 6rem;
    flex: 0 1 18rem;
    font-weight: 600;
  }

  .grow {
    flex: 1;
  }

  /* Boxes or Text, side by side; the one shown is raised. */
  .mode {
    display: inline-flex;
    flex: none;
    margin-left: 0.25rem;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
  }

  .mode button {
    padding: 0.1rem 0.6rem;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-muted);
    font-size: 0.88rem;
  }

  .mode button:hover:not(.chosen) {
    border-color: transparent;
    background: var(--surface-hover);
    color: var(--text);
  }

  .mode button.chosen,
  .mode button.chosen:hover {
    border-color: var(--border);
    background: var(--surface);
    color: var(--text);
    font-weight: 600;
    box-shadow: var(--shadow);
  }

  /* As wide as the card head's *In batch n* (the screen sets both widths), so the numbers stand
     in one column under it. */
  .count {
    flex: none;
    width: var(--count-width);
    text-align: right;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .mark {
    color: var(--danger);
  }

  .tools {
    display: inline-flex;
    flex: none;
    justify-content: flex-end;
    gap: 0.1rem;
    width: var(--tools-width);
  }

  /* Under the name, so the code stands alone at the left edge. */
  .body {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.4rem;
    margin: 0.5rem 0 0 3.5rem;
  }

  /* As wide as the row up to a line's reading length, so a value box does not run the width of a
     full window. */
  .body > :global(.boxes),
  .filter-text,
  .error {
    align-self: stretch;
    max-width: 58rem;
  }

  .filter-text {
    font-family: var(--font-mono);
  }

  .error {
    margin: 0;
  }

  .rest {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    color: var(--text-muted);
  }

  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--link);
    cursor: pointer;
  }

  .link:hover:not(:disabled) {
    background: none;
    text-decoration: underline;
  }

  .help {
    margin: 0;
    padding: 0.5rem 0.75rem 0.5rem 1.8rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
  }

  .help code {
    font-family: var(--font-mono);
  }
</style>
