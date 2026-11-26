<script lang="ts">
  // A Search card's details, under its bar: one line per head. The receiver's address and, when they
  // apply, the pickup code and the account (`detailLines`); the input PDF it was read from, its pages
  // there and its short hash; the group's code, its PDF's name and the order's pages in it.
  import type { SearchRow } from "./api/search";
  import { detailLines, fileLabel } from "./search.svelte";
  import { texts } from "../texts";
  import Icon from "./Icon.svelte";

  let { row }: { row: SearchRow } = $props();

  const words = texts.search;
  const heads = words.details;
  const lines = $derived(detailLines(row));
</script>

<dl class="details">
  {#each lines as line (line.head)}
    <dt>{line.head}</dt>
    <dd class:faint={line.notRead} title={line.notRead ? words.notReadWhy : undefined}>{line.text}</dd>
  {/each}
  <dt>{heads.input}</dt>
  <dd>
    <span class="file"><Icon name="file" size={14} />{row.input}</span>
    {words.pages(row.input_first_page, row.input_last_page)}
    <span class="hash">{row.input_hash}</span>
  </dd>
  <dt>{heads.group}</dt>
  <dd>
    <span class="code">{row.group}</span>
    <span title={row.file}>{fileLabel(row)}</span>
    <span class="strong">{words.pages(row.first_page, row.last_page)}</span>
  </dd>
</dl>

<style>
  /* Under the bar, from the shop's column to the card's end: the heads in a column of their own. */
  .details {
    display: grid;
    grid-row: 4;
    grid-column: 2 / -1;
    grid-template-columns: max-content minmax(0, 1fr);
    align-items: center;
    gap: 0.3rem 1.1rem;
    margin: 0.2rem 0 0.45rem;
    font-size: 0.88rem;
  }

  dt {
    color: var(--text-faint);
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
  }

  dd {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.2rem 0.6rem;
    margin: 0;
    overflow-wrap: anywhere;
  }

  .faint {
    color: var(--text-faint);
  }

  .strong {
    font-weight: 600;
  }

  /* The input's file name, set apart as a file: its icon, a box, monospace. */
  .file {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    min-width: 0;
    padding: 0.05rem 0.45rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
    font-family: var(--font-mono);
    font-size: 0.8rem;
  }

  .file :global(svg) {
    flex: none;
    color: var(--text-faint);
  }

  .hash {
    color: var(--text-faint);
    font-family: var(--font-mono);
    font-size: 0.8rem;
  }

  .code {
    padding: 0 0.4rem;
    border-radius: var(--radius);
    background: var(--chip);
    color: var(--chip-text);
    font-weight: 700;
  }

  @container results (max-width: 52rem) {
    .details {
      grid-row: 5;
    }
  }
</style>
