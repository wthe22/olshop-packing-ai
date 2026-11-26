<script lang="ts">
  // One order of Search's results as a card. Its head is two lines on the results' own columns, so
  // the cards line up: the shop over the Order ID, the courier over the tracking ID, the batch's date
  // and save time over its number, the ship-by date over the weight; at the right **Open** and the
  // *Reprinted* mark. Under the head, the items one line each (quantity, name, variation, Seller SKU),
  // with the Qty Total at their foot when it says more than the one line. The bar under the items
  // opens the rest (`SearchDetails`). A click on the card's text ticks it, for the mouse only: the
  // tick is the keyboard's way.
  import type { SearchRow } from "./api/search";
  import { reprintedMark, totalLine, weightValue } from "./search.svelte";
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
  const weight = $derived(weightValue(row));
  const total = $derived(totalLine(row));

  // The tick, **Open** and the details' bar have their own clicks.
  const textClicks = (card: HTMLElement) => {
    const onClick = (event: MouseEvent) => {
      if ((event.target as Element).closest("button, input")) return;
      onClickText(event.detail, (window.getSelection()?.toString() ?? "") !== "");
    };
    card.addEventListener("click", onClick);
    return () => card.removeEventListener("click", onClick);
  };
</script>

<div class="card" class:ticked {@attach textClicks}>
  {#if access.readOnly}
    <span></span>
  {:else}
    <input type="checkbox" aria-label={words.tick(row.order_id)} checked={ticked} onchange={onToggle} />
  {/if}
  <div class="cell who">
    {#if row.shop}
      <span class="shop" title={row.shop}>{row.shop}</span>
    {:else}
      <span class="shop account" title={row.account}>{row.account}</span>
    {/if}
    <span class="order">{row.order_id}</span>
  </div>
  <div class="cell track">
    <span><span class="courier" class:unknown={!row.courier}>{row.courier ?? words.noCourier}</span></span>
    <span class="tracking" class:faint={!row.tracking_id}>{row.tracking_id ?? words.noTracking}</span>
  </div>
  <div class="cell saved">
    <span class="muted">{formatShortDay(row.day)} {row.batch_time}</span>
    <span class="strong">{words.batch(row.batch)}</span>
  </div>
  <div class="cell parcel">
    <span>
      {#if row.ship_by}<span class="head">{words.shipBy}</span>{formatShortDay(row.ship_by)}{/if}
    </span>
    <span>
      {#if weight}<span class="head">{words.weight}</span><span
          class:faint={weight.notRead}
          title={weight.notRead ? words.notReadWhy : undefined}>{weight.text}</span
        >{/if}
    </span>
  </div>
  <div class="tools">
    <button class="small" title={words.openFile(row.file)} onclick={onOpenOutput}>{words.open}</button>
    {#if mark}<span class="mark" title={mark.title}>{mark.text}</span>{/if}
  </div>
  <ul class="items">
    {#each row.items as item, i (i)}
      <li>
        <span class="qty">×{item.quantity}</span>
        <span class="name">{item.name}</span>
        <span class="more">
          <span class="variant" class:none={!item.variant}>{item.variant}</span>
          <span class="sku">{item.seller_sku ?? ""}</span>
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
  <button
    class="expand"
    class:open={traced}
    title={words.trace}
    aria-label={words.trace}
    aria-expanded={traced}
    onclick={onTrace}><Icon name="chevron" size={16} /></button
  >
  {#if traced}
    <SearchDetails {row} />
  {/if}
</div>

<style>
  /* The columns are the results' own (`SearchScreen`), so every card's head lines up. */
  .card {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: subgrid;
    align-items: start;
    padding: 0.5rem 0.8rem 0.15rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
    cursor: default;
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

  input[type="checkbox"] {
    grid-row: 1;
    grid-column: 1;
    margin-top: 0.3rem;
  }

  /* Each cell is the head's two lines, so a cell with an empty line keeps the other on its row. */
  .cell {
    display: grid;
    grid-row: 1;
    grid-template-rows: repeat(2, 1.5rem);
    align-items: center;
    min-width: 0;
    white-space: nowrap;
  }

  .who {
    grid-column: 2;
  }

  .track {
    grid-column: 3;
  }

  .saved {
    grid-column: 4;
  }

  .parcel {
    grid-column: 5;
  }

  .tools {
    display: flex;
    grid-row: 1;
    grid-column: 6;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.3rem;
  }

  /* A long shop name never widens the column: it ends at the Order ID's width, whole in its tooltip. */
  .shop {
    contain: inline-size;
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

  .saved,
  .parcel {
    font-size: 0.88rem;
  }

  .muted {
    color: var(--text-muted);
  }

  .strong {
    font-weight: 600;
  }

  .faint {
    color: var(--text-faint);
  }

  .head {
    margin-right: 0.45rem;
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
    white-space: nowrap;
  }

  /* The items as a table without heads: the quantity, the name taking the room left, then the
     variation and the Seller SKU at fixed widths, so they line up from card to card. */
  .items {
    display: grid;
    grid-row: 2;
    grid-column: 2 / -1;
    grid-template-columns: 2.2rem minmax(0, 1fr) minmax(0, 11rem) minmax(0, 9.5rem);
    gap: 0.2rem 0.9rem;
    margin: 0.45rem 0 0;
    padding: 0.45rem 0 0;
    border-top: 1px solid var(--border-soft);
    list-style: none;
  }

  .items li,
  .more {
    display: contents;
  }

  .qty {
    color: var(--accent-text);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .name,
  .variant,
  .sku {
    overflow-wrap: anywhere;
  }

  .variant {
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .sku {
    color: var(--text-faint);
    font-family: var(--font-mono);
    font-size: 0.8rem;
    line-height: 1.6;
  }

  /* The Qty Total under the quantities, a sum's stroke above it. */
  .total .qty {
    padding-top: 0.1rem;
    border-top: 1px solid var(--border);
  }

  .total .head {
    align-self: center;
  }

  /* The details' bar: a line across the card, the chevron at its end. */
  button.expand {
    display: flex;
    grid-row: 3;
    grid-column: 2 / -1;
    align-items: center;
    gap: 0.4rem;
    margin-top: 0.35rem;
    padding: 0.1rem 0;
    border: none;
    background: none;
    box-shadow: none;
    color: var(--text-muted);
  }

  button.expand::before {
    flex: 1;
    border-top: 1px solid var(--border-soft);
    content: "";
  }

  button.expand:hover {
    color: var(--accent-text);
  }

  button.expand:hover::before {
    border-color: var(--accent);
  }

  button.expand :global(svg) {
    transform: rotate(90deg);
    transition: transform 0.1s;
  }

  button.expand.open :global(svg) {
    transform: rotate(-90deg);
  }

  /* Too narrow for the head's four columns: the date and the parcel go under the shop and the
     courier, and each item's variation and Seller SKU under its name. */
  @container results (max-width: 52rem) {
    .saved,
    .parcel {
      grid-row: 2;
      margin-top: 0.2rem;
    }

    .saved {
      grid-column: 2;
    }

    .parcel {
      grid-column: 3;
    }

    .items {
      grid-row: 3;
      grid-template-columns: 2.2rem minmax(0, 1fr);
    }

    .more {
      display: flex;
      flex-wrap: wrap;
      grid-column: 2;
      gap: 0 0.9rem;
      margin-top: -0.15rem;
    }

    .variant.none {
      display: none;
    }

    button.expand {
      grid-row: 4;
    }
  }
</style>
