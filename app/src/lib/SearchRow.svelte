<script lang="ts">
  // One order of Search's results as a row of the results' grid: its tick; its Order ID over its
  // courier and tracking ID; its items, which take the room left; the date and group code over the
  // group's name and the page the label is on; **Open**, the chevron of its details and its
  // *Reprinted* mark. The details are what the row does not show: its batch's save time and the
  // input PDF it was read from. The order and label columns take only their own width, so a long
  // product name keeps the rest; on a narrow window the items go under them. A click on the row's
  // text ticks it, for the mouse only: the tick is the keyboard's way.
  import type { SearchRow } from "./api/search";
  import { fileLabel, reprintedMark } from "./search.svelte";
  import { texts } from "../texts";
  import { formatShortDay } from "../dates";
  import Icon from "./Icon.svelte";

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
    // A click on the row's text: its click count, and whether it ended a text selection.
    onClickText: (clicks: number, selecting: boolean) => void;
    onOpenOutput: () => void;
  } = $props();

  const words = texts.search;
  const mark = $derived(reprintedMark(row));

  // The tick, **Open** and the chevron have their own clicks.
  const textClicks = (line: HTMLElement) => {
    const onClick = (event: MouseEvent) => {
      if ((event.target as Element).closest("button, input")) return;
      onClickText(event.detail, (window.getSelection()?.toString() ?? "") !== "");
    };
    line.addEventListener("click", onClick);
    return () => line.removeEventListener("click", onClick);
  };
</script>

<div class="row" class:ticked {@attach textClicks}>
  <input type="checkbox" aria-label={words.tick(row.order_id)} checked={ticked} onchange={onToggle} />
  <div class="ids">
    <span class="order">{row.order_id}</span>
    <span class="line">
      <span class="courier" class:unknown={!row.courier}>{row.courier ?? words.noCourier}</span>
      <span class="tracking" class:faint={!row.tracking_id}>{row.tracking_id ?? words.noTracking}</span>
    </span>
  </div>
  <ul class="items">
    {#each row.items as item, i (i)}
      <li><span class="qty">×{item.quantity}</span><span class="name">{item.name}</span></li>
    {/each}
  </ul>
  <div class="where">
    <span class="line">
      <span class="date">{formatShortDay(row.day)}</span>
      <span class="group">{row.group}</span>
    </span>
    <span class="line">
      <span class="file" title={row.file}>{fileLabel(row)}</span>
      <span class="page">{words.pages(row.first_page, row.last_page)}</span>
    </span>
  </div>
  <div class="tools">
    <span class="buttons">
      <button class="small" title={words.openFile(row.file)} onclick={onOpenOutput}>{words.open}</button>
      <button
        class="icon chevron"
        class:open={traced}
        title={words.trace}
        aria-label={words.trace}
        aria-expanded={traced}
        onclick={onTrace}><Icon name="chevron" size={16} /></button
      >
    </span>
    {#if mark}<span class="mark" title={mark.title}>{mark.text}</span>{/if}
  </div>
  {#if traced}
    <p class="trace">
      <span class="batch">{words.batch(row.batch, row.batch_time)}</span>
      <span class="source"
        >{words.from}
        <span class="input"><Icon name="file" size={14} />{row.input}</span>
        {words.inputPages(row.input_first_page, row.input_last_page)}</span
      >
    </p>
  {/if}
</div>

<style>
  /* The columns are the results' own (`SearchScreen`), so every row's IDs and group PDF line up. */
  .row {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: subgrid;
    align-items: start;
    row-gap: 0.35rem;
    padding: 0.45rem 0.75rem;
    border-top: 1px solid var(--border-soft);
    cursor: default;
    transition: background-color 0.12s;
  }

  .row:hover {
    background: var(--surface-hover);
  }

  .row.ticked {
    background: var(--selected);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  /* Each in its own column, so the narrow layouts below move them without the grid's own placing. */
  input[type="checkbox"] {
    grid-column: 1;
    margin-top: 0.2rem;
  }

  .ids {
    grid-column: 2;
  }

  .items {
    grid-column: 3;
  }

  .where {
    grid-column: 4;
  }

  .tools {
    grid-column: 5;
  }

  .ids,
  .where,
  .tools {
    grid-row: 1;
  }

  .ids,
  .where {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    white-space: nowrap;
  }

  .order {
    font-weight: 650;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.01em;
  }

  .tracking {
    color: var(--text-muted);
    font-size: 0.9rem;
    font-variant-numeric: tabular-nums;
  }

  .faint {
    color: var(--text-faint);
  }

  .items {
    min-width: 0;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .items li {
    display: flex;
    gap: 0.5rem;
  }

  .items li + li {
    margin-top: 0.15rem;
  }

  .qty {
    flex: none;
    min-width: 1.9rem;
    color: var(--accent-text);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .name {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .where {
    font-size: 0.88rem;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .group {
    padding: 0 0.4rem;
    border-radius: var(--radius);
    background: var(--chip);
    color: var(--chip-text);
    font-weight: 700;
  }

  /* The group's name only hints; its code and the page find the label. */
  .file {
    max-width: 10rem;
    overflow: hidden;
    color: var(--text-muted);
    text-overflow: ellipsis;
  }

  .page {
    font-weight: 600;
  }

  .courier {
    padding: 0 0.5rem;
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

  .date {
    color: var(--text-muted);
  }

  .tools {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.2rem;
  }

  .buttons {
    display: flex;
    align-items: center;
    gap: 0.15rem;
  }

  button.small {
    padding: 0.05rem 0.6rem;
    font-size: 0.86rem;
  }

  button.chevron {
    padding: 0.15rem;
  }

  button.chevron :global(svg) {
    transition: transform 0.1s;
  }

  button.chevron.open :global(svg) {
    transform: rotate(90deg);
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

  .trace {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem 1.25rem;
    grid-column: 2 / -1;
    margin: 0;
    padding-top: 0.4rem;
    border-top: 1px solid var(--border-soft);
    color: var(--text-muted);
    font-size: 0.84rem;
  }

  .batch {
    color: var(--text);
    font-weight: 600;
  }

  .source {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
    min-width: 0;
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

  .input :global(svg) {
    flex: none;
    color: var(--text-faint);
  }

  /* Too narrow for the items beside the order and the label: the label stands next to the order and
     the items go under both, full width. */
  @container results (max-width: 54rem) {
    .where {
      grid-column: 3;
    }

    .items {
      grid-column: 2 / -1;
      grid-row: 2;
    }
  }

  /* Narrower still: the label's date, code, name and page in one line under the order. */
  @container results (max-width: 34rem) {
    .where {
      flex-flow: row wrap;
      align-items: center;
      gap: 0.2rem 0.9rem;
      grid-column: 2 / -1;
      grid-row: 2;
    }

    .items {
      grid-row: 3;
    }
  }
</style>
