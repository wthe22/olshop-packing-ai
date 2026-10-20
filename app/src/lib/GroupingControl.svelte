<script lang="ts">
  // The set a batch is packed with: the *Set* control
  // of a new batch's file list, with the set used last preselected, or the amend's selection page's
  // fixed set line (an amend reads the batch's own set again). The screen keeps the chosen set and
  // passes it to `read_labels`; nothing is written here.
  import type { GroupingView } from "../lib/api/categories";
  import { texts } from "../texts";

  let {
    amend,
    amendGrouping,
    groupings,
    chosen,
    onChosen,
  }: {
    // The batch being redone; `null` for a new batch's file list, where the control is shown.
    amend: number | null;
    // The batch's own set on the amend's selection page, shown fixed; `null` for a new batch.
    amendGrouping: string | null;
    groupings: GroupingView[];
    chosen: string;
    onChosen: (name: string) => void;
  } = $props();
</script>

{#if amend === null}
  <!-- Chosen before *Read labels*. -->
  <label class="grouping">
    {texts.newBatch.grouping}:
    <select value={chosen} onchange={(event) => onChosen(event.currentTarget.value)}>
      {#each groupings as option (option.name)}
        <option value={option.name}>{option.name}</option>
      {/each}
    </select>
  </label>
{:else if amendGrouping !== null}
  <!-- The amend's selection page: the batch's own set, shown and not chosen. -->
  <span class="grouping-fixed">
    {texts.newBatch.grouping}: {texts.newBatch.groupingFixed(amendGrouping)}
  </span>
{/if}

<style>
  /* The Set control before *Read labels*, and the amend's
     selection page's fixed set line. */
  .grouping,
  .grouping-fixed {
    margin-left: 1rem;
    color: #4a4a4a;
  }

  .grouping select {
    max-width: 14rem;
  }

  @media (prefers-color-scheme: dark) {
    .grouping,
    .grouping-fixed {
      color: #b0b0b0;
    }
  }
</style>
