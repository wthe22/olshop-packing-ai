<script lang="ts">
  // One category of the list: its code and name, its condition built in the
  // boxes editor or typed in a text view, the count it takes in the open batch, and the buttons to
  // move, delete and mark it as the one that takes the rest. Every change recounts through
  // `onChange`; the entry itself is the entry the screen holds.
  import { texts } from "../texts";
  import BoxesEditor from "./BoxesEditor.svelte";
  import { boxesToCondition, textToTree, type CatEntry } from "./categoryEntry";
  import { newTree } from "./conditions";

  let {
    entry,
    draftBatch,
    canUp,
    canDown,
    last,
    onChange,
    onMove,
    onRemove,
    onHelp,
  }: {
    entry: CatEntry;
    // The batch being prepared, for *In batch `<b>`: n*; `null` when no draft is open.
    draftBatch: number | null;
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
    if (await boxesToCondition(entry)) await onChange();
  }

  function onText() {
    entry.when = entry.text;
    entry.boxesError = "";
    onChange();
  }

  async function toText() {
    entry.text = entry.when ?? "";
    entry.mode = "text";
    entry.boxesError = "";
    await onChange();
  }

  async function toBoxes() {
    if (await textToTree(entry)) await onChange();
  }

  async function setRest(checked: boolean) {
    if (checked) {
      entry.when = null;
      entry.boxesError = "";
      entry.error = "";
      onChange();
    } else {
      entry.when = "";
      entry.text = "";
      entry.tree = newTree();
      entry.mode = "boxes";
      entry.boxesError = "";
      await onBoxes();
    }
  }
</script>

<section class="category">
  <div class="cat-head">
    <input
      class="code"
      aria-label={texts.categories.code}
      bind:value={entry.code}
      oninput={onChange}
    />
    <input
      class="name"
      aria-label={texts.categories.name}
      bind:value={entry.name}
      oninput={onChange}
    />
    <span class="grow"></span>
    {#if entry.error}
      <span class="count mark">✗</span>
    {:else if draftBatch !== null && entry.count !== null}
      <span class="count">{texts.categories.inBatch(draftBatch, entry.count)}</span>
    {/if}
    <button
      title={texts.categories.moveUp}
      disabled={!canUp}
      onclick={() => onMove(-1)}>↑</button
    >
    <button
      title={texts.categories.moveDown}
      disabled={!canDown}
      onclick={() => onMove(1)}>↓</button
    >
    <button title={texts.categories.delete} onclick={onRemove}>🗑</button>
  </div>

  {#if entry.when === null}
    <p class="rest">{texts.categories.takesRest}</p>
  {:else if entry.mode === "boxes"}
    <BoxesEditor tree={entry.tree} onEdit={onBoxes} />
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
          checked={entry.when === null}
          onchange={(event) => setRest(event.currentTarget.checked)}
        />
        {texts.categories.takesRest}
      </label>
    {/if}
    {#if entry.when !== null}
      {#if entry.mode === "boxes"}
        <button onclick={toText}>{texts.categories.textView}</button>
      {:else}
        <button disabled={!!entry.error} onclick={toBoxes}
          >{texts.categories.boxes}</button
        >
        <button class="link" onclick={onHelp}
          >{texts.categories.helpExamples}</button
        >
      {/if}
    {/if}
  </div>
</section>

<style>
  .category {
    border: 1px solid #d5d5d5;
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.75rem;
    background: #ffffff;
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
    color: #4a4a4a;
    font-variant-numeric: tabular-nums;
  }

  .count.mark {
    color: #8a1f1f;
    font-weight: 600;
  }

  .rest {
    margin: 0.3rem 0;
    color: #6a6a6a;
    font-style: italic;
  }

  .condition-text {
    width: 100%;
    font-family: Consolas, monospace;
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
    color: #4a4a4a;
  }

  .link {
    padding: 0;
    border: none;
    background: none;
    color: #1f6feb;
    cursor: pointer;
  }

  @media (prefers-color-scheme: dark) {
    .category {
      background: #2b2b2b;
      border-color: #444444;
    }

    .count,
    .rest,
    .rest-toggle {
      color: #b0b0b0;
    }
  }
</style>
