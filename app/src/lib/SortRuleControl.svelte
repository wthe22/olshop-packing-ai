<script lang="ts">
  // The sort rule a batch uses: the choice of a new batch's file list, with the newest batch's
  // rule preselected, or the amend's selection page's fixed line (an amend reads with the batch's
  // own rule). The screen keeps the chosen rule and passes it to `read_labels`.
  import { texts } from "../texts";

  let {
    amend,
    amendRule,
    rules,
    chosen,
    onChosen,
  }: {
    // The batch being redone; `null` for a new batch's file list, where the choice is shown.
    amend: number | null;
    // The batch's own rule on the amend's selection page; `null` for a new batch.
    amendRule: string | null;
    rules: string[];
    chosen: string;
    onChosen: (name: string) => void;
  } = $props();
</script>

{#if amend === null}
  <label class="rule">
    {texts.newBatch.sortRule}:
    <select value={chosen} onchange={(event) => onChosen(event.currentTarget.value)}>
      {#each rules as rule (rule)}
        <option value={rule}>{rule}</option>
      {/each}
    </select>
  </label>
{:else if amendRule !== null}
  <span class="rule">
    {texts.newBatch.sortRule}: {texts.newBatch.sortRuleFixed(amendRule)}
  </span>
{/if}

<style>
  .rule {
    margin-left: 1rem;
    color: var(--text-muted);
  }

  .rule select {
    max-width: 14rem;
  }
</style>
