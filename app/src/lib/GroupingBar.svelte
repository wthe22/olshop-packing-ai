<script lang="ts">
  // The set row of the Categories screen (08 › *5. Categories*; 04 › *Sets*): the chooser, **+ New
  // set** with its name form, **Delete set** and the file the shown set is kept in. The set
  // `default` is the data folder's own `categories.toml` and cannot be deleted. The screen does the
  // writing: a refusal of its call is caught here and shown under the row.
  import type { GroupingView } from "./api/categories";
  import { groupingProblem, normalizeGroupingName, type GroupingNameProblem } from "./groupings";
  import { texts } from "../texts";

  let {
    groupings,
    current,
    file,
    canDelete,
    busy,
    onSelect,
    onCreate,
    onDelete,
  }: {
    groupings: GroupingView[];
    current: string;
    // The file the shown set is kept in, relative to the data folder; `null` when it is unknown.
    file: string | null;
    // *Delete set* is offered at all (the set `default` is never deleted).
    canDelete: boolean;
    busy: boolean;
    onSelect: (name: string) => void;
    // **Create** the named set and show it; `default` and a taken name are refused by the screen.
    onCreate: (name: string) => Promise<void>;
    // *Delete set*: the screen asks first and falls back to `default`.
    onDelete: () => Promise<void>;
  } = $props();

  let newOpen = $state(false);
  let newName = $state("");
  let error = $state("");

  function openNew() {
    newOpen = true;
    newName = "";
    error = "";
  }

  function problemText(problem: GroupingNameProblem): string {
    if (problem === "empty") return texts.categories.groupingNeedsName;
    if (problem === "reserved") return texts.categories.groupingNameReserved;
    return texts.categories.groupingNameTaken;
  }

  async function create() {
    error = "";
    const problem = groupingProblem(newName, groupings);
    if (problem !== null) {
      error = problemText(problem);
      return;
    }
    try {
      await onCreate(normalizeGroupingName(newName));
      newOpen = false;
    } catch (err) {
      error = String(err);
    }
  }

  async function remove() {
    error = "";
    try {
      await onDelete();
    } catch (err) {
      error = String(err);
    }
  }

  function select(name: string) {
    error = "";
    newOpen = false;
    onSelect(name);
  }
</script>

<!-- 08 › *5. Categories*: the set row — the chooser, **+ New set** and **Delete set** — and the
     file the chosen set is kept in. The set `default` is the data folder's own
     `categories.toml` and cannot be deleted. -->
<div class="grouping-row">
  <label class="grouping">
    {texts.categories.grouping}:
    <select value={current} onchange={(event) => select(event.currentTarget.value)}>
      {#each groupings as option (option.name)}
        <option value={option.name}>{option.name}</option>
      {/each}
    </select>
  </label>
  <button disabled={busy} onclick={openNew}>{texts.categories.newGrouping}</button>
  <button disabled={!canDelete || busy} onclick={remove}>
    {texts.categories.deleteGrouping}
  </button>
  {#if newOpen}
    <input
      class="grouping-name"
      placeholder={texts.categories.name}
      bind:value={newName}
      onkeydown={(event) => event.key === "Enter" && create()}
    />
    <button class="primary" disabled={busy} onclick={create}>
      {texts.categories.createGrouping}
    </button>
    <button disabled={busy} onclick={() => (newOpen = false)}>
      {texts.categories.cancel}
    </button>
  {/if}
  <span class="grow"></span>
  {#if file}
    <span class="file-line">{texts.categories.groupingFile(file)}</span>
  {/if}
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

<style>
  /* 08 › *5. Categories*: the set row at the top of the screen. */
  .grouping-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.6rem;
    flex-wrap: wrap;
  }

  .grouping {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    color: #4a4a4a;
  }

  .grouping select {
    max-width: 14rem;
  }

  .grouping-row .grouping-name {
    width: 12rem;
  }

  .grouping-row .file-line {
    color: #6a6a6a;
    font-variant-numeric: tabular-nums;
  }

  .grow {
    flex: 1;
  }

  @media (prefers-color-scheme: dark) {
    .grouping,
    .grouping-row .file-line {
      color: #b0b0b0;
    }
  }
</style>
