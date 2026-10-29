<script lang="ts">
  // One label group of the rule: its code and name, its filter built in the
  // boxes editor or typed in a text view, the count it takes in the open batch, and the buttons to
  // move, delete and mark it as the one that takes the rest. Every change recounts through
  // `onChange`; the entry itself is the entry the screen holds.
  import { texts } from "../texts";
  import BoxesEditor from "./BoxesEditor.svelte";
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
    onHelp,
  }: {
    entry: GroupEntry;
    // The batch being prepared, for *In batch `<b>`: n*; `null` when no draft is open.
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
    // *Examples*: show the help box at the foot of the screen.
    onHelp: () => void;
  } = $props();

  async function onBoxes() {
    if (await boxesToFilter(entry)) await onChange();
  }

  function onText() {
    entry.filter = entry.text;
    entry.boxesError = "";
    onChange();
  }

  async function toText() {
    entry.text = entry.filter ?? "";
    entry.mode = "text";
    entry.boxesError = "";
    await onChange();
  }

  async function toBoxes() {
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

<section class="group">
  <div class="cat-head">
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
    <span class="grow"></span>
    {#if entry.error}
      <span class="count mark">✗</span>
    {:else if draftBatch !== null && entry.count !== null}
      <span class="count">{texts.sortRules.inBatch(draftBatch, entry.count)}</span>
    {/if}
    <button
      title={texts.sortRules.moveUp}
      disabled={!canUp}
      onclick={() => onMove(-1)}>↑</button
    >
    <button
      title={texts.sortRules.moveDown}
      disabled={!canDown}
      onclick={() => onMove(1)}>↓</button
    >
    <button title={texts.sortRules.delete} onclick={onRemove}>🗑</button>
  </div>

  {#if entry.filter === null}
    <p class="rest">{texts.sortRules.takesRest}</p>
  {:else if entry.mode === "boxes"}
    <BoxesEditor tree={entry.tree} {couriers} onEdit={onBoxes} />
  {:else}
    <textarea
      class="condition-text"
      rows="3"
      bind:value={entry.text}
      oninput={onText}
    ></textarea>
  {/if}

  {#if entry.error}
    <p class="error">{entry.error}</p>
  {/if}

  <div class="cat-foot">
    {#if last}
      <label class="rest-toggle">
        <input
          type="checkbox"
          checked={entry.filter === null}
          onchange={(event) => setRest(event.currentTarget.checked)}
        />
        {texts.sortRules.takesRest}
      </label>
    {/if}
    {#if entry.filter !== null}
      {#if entry.mode === "boxes"}
        <button onclick={toText}>{texts.sortRules.textView}</button>
      {:else}
        <button disabled={!!entry.error} onclick={toBoxes}
          >{texts.sortRules.boxes}</button
        >
        <button class="link" onclick={onHelp}
          >{texts.sortRules.helpExamples}</button
        >
      {/if}
    {/if}
  </div>
</section>

<style>
  .group {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.75rem;
    background: var(--surface);
    box-shadow: var(--shadow);
  }

  .cat-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .cat-head .code {
    width: 3.5rem;
  }

  .cat-head .name {
    width: 14rem;
  }

  .grow {
    flex: 1;
  }

  .count {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .count.mark {
    color: var(--danger);
    font-weight: 600;
  }

  .rest {
    margin: 0.3rem 0;
    color: var(--text-muted);
    font-style: italic;
  }

  .condition-text {
    width: 100%;
    font-family: var(--font-mono);
  }

  .cat-foot {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 0.35rem;
  }

  .rest-toggle {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.9rem;
    color: var(--text-muted);
  }

  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--link);
    cursor: pointer;
  }
</style>
