<script lang="ts">
  // A Search row's details, under the row: every field of the order, each under its head (its account,
  // the receiver's province, city and district, the ship-by date, Qty Total, the pickup code and what
  // named its courier); its items with their variation and Seller SKU; and where it came from: the
  // input PDF, its pages there, its batch's save time and the group's code.
  import type { SearchRow } from "./api/search";
  import { detailFields } from "./search.svelte";
  import { texts } from "../texts";
  import Icon from "./Icon.svelte";

  let { row }: { row: SearchRow } = $props();

  const words = texts.search;
  const heads = words.details;
  const fields = $derived(detailFields(row));
</script>

<div class="details">
  <dl class="fields">
    {#each fields as field (field.head)}
      <div>
        <dt>{field.head}</dt>
        <dd class:missing={field.missing}>{field.value}</dd>
      </div>
    {/each}
  </dl>
  <table class="lines">
    <thead>
      <tr>
        <th class="qty">{heads.quantity}</th>
        <th>{heads.item}</th>
        <th>{heads.variation}</th>
        <th>{heads.sellerSku}</th>
      </tr>
    </thead>
    <tbody>
      {#each row.items as item, i (i)}
        <tr>
          <td class="qty">×{item.quantity}</td>
          <td class="name">{item.name}</td>
          <td class:missing={!item.variant}>{item.variant || words.none}</td>
          <td class="sku" class:missing={!item.seller_sku}>{item.seller_sku ?? words.none}</td>
        </tr>
      {/each}
    </tbody>
  </table>
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
    gap: 0.7rem;
    grid-column: 2 / -1;
    padding-top: 0.55rem;
    border-top: 1px solid var(--border-soft);
    font-size: 0.86rem;
  }

  /* Each field's head over its value, as many to a line as fit. */
  .fields {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1.75rem;
    margin: 0;
  }

  dt,
  th {
    color: var(--text-faint);
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
  }

  dd {
    margin: 0.1rem 0 0;
  }

  .missing {
    color: var(--text-faint);
  }

  /* The items as the slip lists them, a line between items. */
  .lines {
    width: 100%;
    border: 1px solid var(--border);
    border-collapse: separate;
    border-spacing: 0;
    border-radius: var(--radius);
    overflow: hidden;
  }

  th {
    padding: 0.3rem 0.6rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
    text-align: left;
  }

  td {
    padding: 0.3rem 0.6rem;
    vertical-align: top;
  }

  tr + tr td {
    border-top: 1px solid var(--border-soft);
  }

  .qty {
    width: 1%;
    white-space: nowrap;
  }

  td.qty {
    color: var(--accent-text);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .name {
    overflow-wrap: anywhere;
  }

  .sku {
    font-family: var(--font-mono);
    font-size: 0.8rem;
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
