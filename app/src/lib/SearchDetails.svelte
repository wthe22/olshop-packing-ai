<script lang="ts">
  // A Search row's details, under the row: what the label prints that the row does not show, each head
  // before its value in one line that wraps (the receiver's area, the weight, the ship-by date, Qty
  // Total, the pickup code, the account of a shop of another name); and where it came from: the input
  // PDF, its pages there, its batch's save time and the group's code. The items' Seller SKUs are in
  // the row, under each item.
  import type { SearchRow } from "./api/search";
  import { detailFacts } from "./search.svelte";
  import { texts } from "../texts";
  import Icon from "./Icon.svelte";

  let { row }: { row: SearchRow } = $props();

  const words = texts.search;
  const facts = $derived(detailFacts(row));
</script>

<div class="details">
  {#if facts.length > 0}
    <p class="facts">
      {#each facts as fact (fact.head)}
        <span class="fact"><span class="head">{fact.head}</span><span class:faint={fact.notRead}>{fact.value}</span></span>
      {/each}
    </p>
  {/if}
  <p class="trace">
    {words.from}
    <span class="input"><Icon name="file" size={14} />{row.input}<span class="hash">{row.input_hash}</span></span>
    {words.inputPages(row.input_first_page, row.input_last_page)}{words.savedInto(row.batch_time)}
    <span class="group">{row.group}</span>
  </p>
</div>

<style>
  /* Under the row, from the order's column to its end. */
  .details {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    grid-column: 2 / -1;
    padding-top: 0.55rem;
    border-top: 1px solid var(--border-soft);
    font-size: 0.86rem;
  }

  .facts {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem 1.5rem;
    margin: 0;
  }

  .fact {
    display: inline-flex;
    align-items: baseline;
    gap: 0.45rem;
  }

  .head {
    color: var(--text-faint);
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
  }

  .faint {
    color: var(--text-faint);
  }

  .trace {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem 0.4rem;
    margin: 0;
    color: var(--text-muted);
  }

  /* The input's file name, set apart as a file: its icon, a box, monospace. */
  .input {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    min-width: 0;
    padding: 0.05rem 0.45rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 0.8rem;
    overflow-wrap: anywhere;
  }

  .hash {
    padding-left: 0.4rem;
    border-left: 1px solid var(--border);
    opacity: 0.7;
  }

  .input :global(svg) {
    flex: none;
    color: var(--text-faint);
  }

  .group {
    padding: 0 0.4rem;
    border-radius: var(--radius);
    background: var(--chip);
    color: var(--chip-text);
    font-weight: 700;
  }
</style>
