<script lang="ts">
  // One order of Search's results as a card: its tick, Order ID, *Reprinted* mark and date; its
  // tracking ID and courier; its items; the group PDF and the page it is on there, with **Open**
  // and the chevron of its trace line. A click on the card's text ticks it, for the mouse only: the
  // tick is the keyboard's way.
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
    // A click on the card's text: its click count, and whether it ended a text selection.
    onClickText: (clicks: number, selecting: boolean) => void;
    onOpenOutput: () => void;
  } = $props();

  const words = texts.search;
  const mark = $derived(reprintedMark(row));

  // The tick, **Open** and the chevron have their own clicks.
  const textClicks = (card: HTMLElement) => {
    const onClick = (event: MouseEvent) => {
      if ((event.target as Element).closest("button, input")) return;
      onClickText(event.detail, (window.getSelection()?.toString() ?? "") !== "");
    };
    card.addEventListener("click", onClick);
    return () => card.removeEventListener("click", onClick);
  };
</script>

<article class="card" class:ticked {@attach textClicks}>
  <div class="top">
    <input type="checkbox" aria-label={words.tick(row.order_id)} checked={ticked} onchange={onToggle} />
    <span class="order">{row.order_id}</span>
    {#if mark}<span class="mark" title={mark.title}>{mark.text}</span>{/if}
    <span class="grow"></span>
    <span class="date">{formatShortDay(row.day)}</span>
  </div>
  <div class="ship">
    <span class="field">{words.trackingId}</span>
    <span class="tracking" class:faint={!row.tracking_id}>{row.tracking_id ?? words.none}</span>
    <span class="grow"></span>
    <span class="courier" class:unknown={!row.courier}>{row.courier ?? words.noCourier}</span>
  </div>
  <ul class="items">
    {#each row.items as item, i (i)}
      <li><span class="qty">×{item.quantity}</span><span>{item.name}</span></li>
    {/each}
  </ul>
  <div class="where">
    <span class="group">{row.group}</span>
    <span class="file" title={row.file}>{fileLabel(row)}</span>
    <span class="page">{words.pages(row.first_page, row.last_page)}</span>
    <span class="grow"></span>
    <button class="small" title={words.openFile(row.file)} onclick={onOpenOutput}>{words.open}</button>
    <button
      class="icon chevron"
      class:open={traced}
      title={words.trace}
      aria-label={words.trace}
      aria-expanded={traced}
      onclick={onTrace}><Icon name="chevron" size={16} /></button
    >
  </div>
  {#if traced}
    <p class="trace">
      {#each row.trace as part, i (i)}{#if part.code}<code>{part.text}</code>{:else}{part.text}{/if}{/each}
    </p>
  {/if}
</article>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.7rem 0.8rem 0.6rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
    cursor: default;
    transition:
      border-color 0.12s,
      background-color 0.12s;
  }

  .card:hover {
    border-color: var(--border-strong);
  }

  .card.ticked {
    border-color: var(--accent);
    background: var(--selected);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .top,
  .ship,
  .where {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .grow {
    flex: 1;
  }

  .faint {
    color: var(--text-faint);
  }

  .order {
    font-size: 1.02rem;
    font-weight: 650;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.01em;
  }

  .date {
    color: var(--text-muted);
    font-size: 0.88rem;
    white-space: nowrap;
  }

  .ship {
    align-items: baseline;
    padding-left: 1.6rem;
    font-size: 0.92rem;
  }

  .field {
    color: var(--text-faint);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .tracking {
    font-variant-numeric: tabular-nums;
  }

  .courier {
    padding: 0.05rem 0.55rem;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent-text);
    font-size: 0.82rem;
    font-weight: 600;
    white-space: nowrap;
  }

  .courier.unknown {
    background: var(--surface-2);
    color: var(--text-faint);
  }

  .items {
    margin: 0;
    padding: 0.4rem 0.6rem;
    border-radius: var(--radius);
    background: var(--surface-2);
    list-style: none;
  }

  .items li {
    display: flex;
    gap: 0.55rem;
    padding: 0.1rem 0;
  }

  .qty {
    flex: none;
    min-width: 2.2rem;
    color: var(--accent-text);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .where {
    font-size: 0.9rem;
  }

  .group {
    padding: 0 0.45rem;
    border-radius: var(--radius);
    background: var(--chip);
    color: var(--chip-text);
    font-weight: 700;
    white-space: nowrap;
  }

  .file {
    min-width: 0;
    overflow: hidden;
    color: var(--text-muted);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .page {
    font-weight: 600;
    white-space: nowrap;
  }

  button.small {
    padding: 0.1rem 0.65rem;
    font-size: 0.88rem;
  }

  button.chevron {
    padding: 0.2rem;
  }

  button.chevron :global(svg) {
    transition: transform 0.1s;
  }

  button.chevron.open :global(svg) {
    transform: rotate(90deg);
  }

  .mark {
    padding: 0 0.5rem;
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
    margin: 0;
    padding-top: 0.45rem;
    border-top: 1px solid var(--border-soft);
    color: var(--text-muted);
    font-size: 0.84rem;
    word-break: break-word;
  }
</style>
