<script lang="ts">
  // Search: saved orders of the open workspace found by a filter made with the boxes, over a range
  // of dates or all of them. Each order is a card that names the group PDF and the page the order
  // is on, opens its trace line and carries its *Reprinted* mark; the ticked cards are reprinted
  // into one PDF from the bar at the foot. Enter in a box searches, so a scanned tracking ID finds
  // its order at once. The state and the commands are `search.svelte.ts`.
  import { onMount } from "svelte";
  import { Search } from "./search.svelte";
  import { RANGE_PRESETS, type RangePreset } from "./searchRange";
  import { searchSummary, texts } from "../texts";
  import { conditionFields } from "../fields";
  import BoxesEditor from "./BoxesEditor.svelte";
  import Icon from "./Icon.svelte";
  import SearchCard from "./SearchCard.svelte";

  let {
    day,
    today,
    onOpen,
    onOpenOutput,
    onReprinted,
  }: {
    // The open date: *This day*.
    day: string;
    // The other choices of dates end today.
    today: string;
    // The reprint just written.
    onOpen: (path: string) => void;
    // A card's group PDF, by the date it was saved on and its file name.
    onOpenOutput: (day: string, file: string) => void;
    onReprinted: () => void;
  } = $props();

  const search = new Search(
    () => day,
    () => today,
    () => onReprinted(),
  );

  let filters: HTMLDivElement;

  const tickedCount = $derived(search.ticked.length);
  const words = texts.search;

  onMount(() => {
    filters.querySelector<HTMLInputElement>("input.value")?.focus();
    search.loadCouriers();
  });

  function onKey(event: KeyboardEvent) {
    const target = event.target;
    if (event.key !== "Enter" || !(target instanceof HTMLInputElement)) return;
    if (!target.classList.contains("value") || !filters.contains(target) || search.busy) return;
    event.preventDefault();
    search.run();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="search">
  <div class="query">
    <div class="filter" bind:this={filters}>
      <BoxesEditor
        tree={search.tree}
        fields={conditionFields}
        couriers={search.couriers}
        onEdit={search.invalidate}
      />
    </div>
    <div class="scope">
      <div class="dates">
        <label for="search-dates" class="label">{words.dates}</label>
        <select
          id="search-dates"
          value={search.preset}
          onchange={(event) => search.setPreset(event.currentTarget.value as RangePreset)}
        >
          {#each RANGE_PRESETS as preset (preset)}
            <option value={preset}>{words.ranges[preset]}</option>
          {/each}
        </select>
      </div>
      {#if search.preset !== "all"}
        <div class="range">
          <input
            type="date"
            aria-label={words.rangeFrom}
            value={search.range.from}
            onchange={(event) => search.setRange({ ...search.range, from: event.currentTarget.value })}
          />
          <span class="to">{words.rangeTo}</span>
          <input
            type="date"
            aria-label={words.rangeUntil}
            value={search.range.to}
            onchange={(event) => search.setRange({ ...search.range, to: event.currentTarget.value })}
          />
        </div>
      {/if}
      <button class="primary go" disabled={search.busy} onclick={search.run}
        ><Icon name="search" size={16} />{words.search}</button
      >
    </div>
  </div>

  {#if search.error}
    <p class="error">{search.error}</p>
  {/if}
  {#if search.loading}
    <p class="muted">{words.loading}</p>
  {/if}

  {#if search.result}
    {#if search.result.orders === 0}
      <p class="empty">{words.noMatch}</p>
    {:else}
      <div class="found">
        <span class="summary">{searchSummary(search.result.orders, search.result.batches)}</span>
        <label class="all">
          <input
            type="checkbox"
            checked={search.allTicked()}
            indeterminate={search.someTicked()}
            onchange={(event) => search.selectAll(event.currentTarget.checked)}
          />
          {words.selectAll}
        </label>
      </div>
      <div class="cards">
        {#each search.result.rows as row (row.day + row.order_id)}
          <SearchCard
            {row}
            ticked={search.isTicked(row)}
            traced={search.isTraced(row)}
            onToggle={() => search.toggle(row)}
            onTrace={() => search.toggleTrace(row)}
            onClickText={(clicks, selecting) => search.clickRow(row, clicks, selecting)}
            onOpenOutput={() => onOpenOutput(row.day, row.file)}
          />
        {/each}
      </div>
      {#if search.findings.length > 0}
        <div class="error">
          <p>{words.notKept}</p>
          <ul>
            {#each search.findings as finding}
              <li>{finding}</li>
            {/each}
          </ul>
        </div>
      {/if}
    {/if}
  {/if}

  {#if search.made || (search.result && search.result.orders > 0)}
    <div class="bar">
      {#if search.made}
        <p class="made">
          {words.made(search.made.number, search.made.orders, search.made.pages)}
          <button onclick={() => search.made && onOpen(search.made.path)}>{texts.day.open}</button>
        </p>
      {/if}
      {#if search.result && search.result.orders > 0}
        <div class="actions">
          <span>{words.selected(tickedCount, search.result.orders)}</span>
          {#if search.reprintedBefore() > 0}
            <span class="warn">{words.reprintedBefore(search.reprintedBefore())}</span>
          {/if}
          <span class="grow"></span>
          <button class="ghost" disabled={tickedCount === 0} onclick={search.clear}
            >{words.clear}</button
          >
          <button class="primary" disabled={search.busy || tickedCount === 0} onclick={search.reprint}
            >{words.reprint(tickedCount)}</button
          >
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .search {
    display: flex;
    flex-direction: column;
    min-height: 100%;
  }

  /* The filter at the left, the dates and **Search** at the right; one above the other when the
     window is narrow. The boxes editor's outer box is this one. */
  .query {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1.25rem;
    padding: 0.75rem 0.9rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
  }

  .filter {
    flex: 1 1 34rem;
    min-width: 0;
  }

  .query :global(.group.root) {
    padding: 0;
    border: none;
    background: none;
  }

  .scope {
    display: flex;
    flex: 0 0 auto;
    flex-direction: column;
    gap: 0.5rem;
    padding-left: 1.25rem;
    border-left: 1px solid var(--border-soft);
  }

  .dates,
  .range {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .dates select {
    flex: 1;
  }

  .label {
    min-width: 3rem;
    font-weight: 600;
  }

  .to {
    color: var(--text-muted);
  }

  .go {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    margin-top: auto;
  }

  .found {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin: 1rem 0 0.6rem;
  }

  .summary {
    font-size: 1.05rem;
    font-weight: 650;
  }

  .all {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    color: var(--text-muted);
    cursor: pointer;
  }

  .empty {
    margin: 1.5rem 0;
    color: var(--text-muted);
    text-align: center;
  }

  .grow {
    flex: 1;
  }

  .muted {
    color: var(--text-muted);
  }

  /* As many columns of cards as fit: three on a wide window, one on a narrow one. */
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 23rem), 1fr));
    gap: 0.75rem;
    align-items: start;
    margin-bottom: 1rem;
  }

  /* The reprint bar stays at the foot of the overlay while the cards scroll under it. */
  .bar {
    position: sticky;
    bottom: -1rem;
    margin: auto -1.25rem -1rem;
    padding: 0.6rem 1.25rem 0.75rem;
    border-top: 1px solid var(--border);
    background: var(--surface);
    box-shadow: 0 -6px 16px var(--bar-shadow);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.8rem;
  }

  .warn {
    color: var(--warn);
  }

  .made {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    margin: 0 0 0.5rem;
    padding: 0.35rem 0.75rem;
    border: 1px solid var(--ok-border);
    border-radius: var(--radius);
    background: var(--ok-bg);
    color: var(--ok);
  }
</style>
