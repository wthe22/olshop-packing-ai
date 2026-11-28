<script lang="ts">
  // One order of Search's results as a card. Its head is one row on the results' columns, under the
  // heads `SearchScreen` puts above the cards: the tick; the shop and the Order ID; the courier and
  // the tracking ID; the date and time its batch was saved; the batch, the group's code, its name and
  // the order's page in the group PDF (the name and page move to the details when the row has no
  // room); the *Reprinted* mark, **Open** and the details' chevron. Under the head the items, one
  // line each in one size (quantity, name, variation, the Seller SKU as a chip), with the Qty Total
  // at their foot when it says more than the one line. A click on the card, or on the chevron, opens
  // and closes the rest (`SearchDetails`); only the tick ticks. An order of an archived date has
  // the archive icon in its tick's place and no **Open**.
  import type { SearchRow } from "./api/search";
  import { reprintedMark, totalLine } from "./search.svelte";
  import { texts } from "../texts";
  import { formatShortDay } from "../dates";
  import Icon from "./Icon.svelte";
  import SearchDetails from "./SearchDetails.svelte";
  import { access } from "./readOnly.svelte";

  let {
    row,
    ticked,
    traced,
    onToggle,
    onTrace,
    onClickText,
    onOpenOutput,
  }: {
    row: SearchRow;
    ticked: boolean;
    traced: boolean;
    onToggle: () => void;
    onTrace: () => void;
    // A click on the card's text: its click count, and whether it ended a text selection.
    onClickText: (clicks: number, selecting: boolean) => void;
    onOpenOutput: () => void;
  } = $props();

  const words = texts.search;
  const mark = $derived(reprintedMark(row));
  const total = $derived(totalLine(row));

  // The tick and the buttons have their own clicks; the open details are for reading and copying.
  const cardClicks = (card: HTMLElement) => {
    const onClick = (event: MouseEvent) => {
      if ((event.target as Element).closest("button, input, .details")) return;
      onClickText(event.detail, (window.getSelection()?.toString() ?? "") !== "");
    };
    card.addEventListener("click", onClick);
    return () => card.removeEventListener("click", onClick);
  };
</script>

<div class="card" class:ticked {@attach cardClicks}>
  {#if row.archived}
    <span class="tick archived" role="img" title={texts.archive.archivedRow} aria-label={texts.archive.archivedRow}
      ><Icon name="archive" size={16} /></span
    >
  {:else if access.readOnly}
    <span class="tick"></span>
  {:else}
    <input class="tick" type="checkbox" aria-label={words.tick(row.order_id)} checked={ticked} onchange={onToggle} />
  {/if}
  <div class="cell">
    {#if row.shop}
      <span class="shop" title={row.shop}>{row.shop}</span>
    {:else}
      <span class="shop account" title={row.account}>{row.account}</span>
    {/if}
    <span class="order">{row.order_id}</span>
  </div>
  <div class="cell">
    <span class="courier" class:unknown={!row.courier}>{row.courier ?? words.noCourier}</span>
    <span class="tracking" class:faint={!row.tracking_id}>{row.tracking_id ?? words.noTracking}</span>
  </div>
  <div class="cell muted">
    {formatShortDay(row.day)}
    {row.batch_time}
  </div>
  <div class="cell label">
    <span class="strong">{words.batch(row.batch)}</span>
    <span class="code">{row.group}</span>
    <span class="roomy group-name" title={row.group_name}>{row.group_name}</span>
    <span class="roomy strong">{words.pages(row.first_page, row.last_page)}</span>
  </div>
  <div class="cell tools">
    {#if mark}<span class="mark" title={mark.title}>{mark.text}</span>{/if}
    {#if !row.archived}
      <button class="small" title={words.openFile(row.file)} onclick={onOpenOutput}>{words.open}</button>
    {/if}
    <button
      class="icon expand"
      class:open={traced}
      title={words.trace}
      aria-label={words.trace}
      aria-expanded={traced}
      onclick={onTrace}><Icon name="chevron" size={16} /></button
    >
  </div>
  <ul class="items">
    {#each row.items as item, i (i)}
      <li>
        <span class="qty">×{item.quantity}</span>
        <span class="name">{item.name}</span>
        <span class="more">
          <span class="variant" class:none={!item.variant}>{item.variant}</span>
          {#if item.sku}<span class="sku">{item.sku}</span>{:else}<span></span>{/if}
        </span>
      </li>
    {/each}
    {#if total !== null}
      <li class="total">
        <span class="qty">{total}</span>
        <span class="head">{words.total}</span>
      </li>
    {/if}
  </ul>
  {#if traced}
    <SearchDetails {row} />
  {/if}
</div>

<style>
  /* The columns are the results' own (`SearchScreen`), so every card's head lines up under the
     heads. */
  .card {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: subgrid;
    align-items: center;
    padding: 0.5rem 0.8rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
    cursor: pointer;
    transition: background-color 0.12s;
  }

  .card:hover {
    background: var(--surface-hover);
  }

  .card.ticked {
    border-color: var(--accent-border);
    background: var(--selected);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  /* The tick in the first column; the head's cells fill the next ones in their order. */
  .tick {
    grid-row: 1;
    grid-column: 1;
  }

  .cell {
    display: flex;
    grid-row: 1;
    align-items: center;
    gap: 0.55rem;
    min-width: 0;
    white-space: nowrap;
  }

  .tools {
    justify-content: flex-end;
    gap: 0.4rem;
  }

  /* The label's column takes the room left: a long group name ends with dots before the page does. */
  .label {
    overflow: hidden;
  }

  .group-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* A long shop name ends with dots; its tooltip has it whole. */
  .shop {
    max-width: 16rem;
    overflow: hidden;
    color: var(--text-muted);
    font-size: 0.9rem;
    text-overflow: ellipsis;
  }

  /* The label printed no sender: the account stands in the shop's place. */
  .shop.account {
    font-style: italic;
  }

  .order {
    font-size: 0.92rem;
    font-weight: 650;
    font-variant-numeric: tabular-nums;
  }

  .courier {
    padding: 0.05rem 0.5rem;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent-text);
    font-size: 0.8rem;
    font-weight: 600;
  }

  .courier.unknown {
    background: var(--surface-2);
    color: var(--text-faint);
  }

  .tracking {
    color: var(--text-muted);
    font-size: 0.9rem;
    font-variant-numeric: tabular-nums;
  }

  /* An archived date's order: the archive icon, grey, where the tick would be. */
  .archived {
    display: flex;
    justify-content: center;
    color: var(--text-faint);
  }

  .muted {
    color: var(--text-muted);
    font-size: 0.88rem;
  }

  .strong {
    font-weight: 600;
  }

  .faint {
    color: var(--text-faint);
  }

  .code {
    padding: 0 0.4rem;
    border-radius: var(--radius);
    background: var(--chip);
    color: var(--chip-text);
    font-weight: 700;
  }

  .head {
    color: var(--text-faint);
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
  }

  button.small {
    padding: 0.05rem 0.7rem;
    font-size: 0.86rem;
  }

  .mark {
    padding: 0 0.45rem;
    border: 1px solid var(--warn-border);
    border-radius: 999px;
    background: var(--warn-bg);
    color: var(--warn);
    font-size: 0.75rem;
    font-weight: 600;
    line-height: 1.5;
  }

  /* The items as a table without heads, in one size: the quantity, the name taking the room left,
     then the variation and the Seller SKU at fixed widths, so they line up from card to card. */
  .items {
    display: grid;
    grid-row: 2;
    grid-column: 2 / -1;
    grid-template-columns: 2.4rem minmax(0, 1fr) minmax(0, 11rem) minmax(0, 9.5rem);
    align-items: baseline;
    gap: 0.15rem 0.9rem;
    margin: 0.4rem 0 0;
    padding: 0.4rem 0 0;
    border-top: 1px solid var(--border-soft);
    font-size: 0.9rem;
    list-style: none;
  }

  .items li,
  .more {
    display: contents;
  }

  .qty {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .name,
  .variant {
    overflow-wrap: anywhere;
  }

  .variant {
    color: var(--text-muted);
  }

  /* The Seller SKU as a code: a quiet chip at the column's start. */
  .sku {
    justify-self: start;
    max-width: 100%;
    padding: 0 0.4rem;
    border-radius: var(--radius);
    background: var(--surface-2);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 0.8rem;
    overflow-wrap: anywhere;
  }

  .total .head {
    align-self: center;
  }

  button.expand :global(svg) {
    transform: rotate(90deg);
    transition: transform 0.1s;
  }

  button.expand.open :global(svg) {
    transform: rotate(-90deg);
  }

  /* No room for the whole head in one row: the group's name and the page are in the details. */
  @container results (max-width: 72rem) {
    .roomy {
      display: none;
    }
  }
</style>
