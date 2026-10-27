<script lang="ts">
  // Search: saved orders of the open workspace found by a filter made with the boxes, over a range
  // of dates or all of them, each row with its trace line and its *Reprinted* mark; the ticked rows are
  // reprinted into one PDF from the bar at the foot. Enter in a box searches, so a scanned tracking
  // ID finds its order at once. The state and the commands are `search.svelte.ts`.
  import { onMount } from "svelte";
  import { Search, itemsText, reprintedMark } from "./search.svelte";
  import { RANGE_PRESETS, type RangePreset } from "./searchRange";
  import { searchSummary, texts } from "../texts";
  import { conditionFields } from "../fields";
  import { formatShortDay } from "../dates";
  import BoxesEditor from "./BoxesEditor.svelte";

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
    // A row's group PDF, by the date it was saved on and its file name.
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
  const columns = texts.search.columns;

  // A row's tick has an id, so each of the row's cells can be a label of it.
  const tickId = (row: { day: string; order_id: string }) => `tick-${row.day}-${row.order_id}`;

  onMount(() => filters.querySelector<HTMLInputElement>("input.value")?.focus());

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
  <div bind:this={filters}>
    <BoxesEditor tree={search.tree} fields={conditionFields} onEdit={search.invalidate} />
  </div>

  <div class="controls">
    <label for="search-dates" class="label">{texts.search.dates}</label>
    <select
      id="search-dates"
      value={search.preset}
      onchange={(event) => search.setPreset(event.currentTarget.value as RangePreset)}
    >
      {#each RANGE_PRESETS as preset (preset)}
        <option value={preset}>{texts.search.ranges[preset]}</option>
      {/each}
    </select>
    {#if search.preset !== "all"}
      <input
        type="date"
        aria-label={texts.search.rangeFrom}
        value={search.range.from}
        onchange={(event) => search.setRange({ ...search.range, from: event.currentTarget.value })}
      />
      <span>{texts.search.rangeTo}</span>
      <input
        type="date"
        aria-label={texts.search.rangeUntil}
        value={search.range.to}
        onchange={(event) => search.setRange({ ...search.range, to: event.currentTarget.value })}
      />
    {/if}
    <span class="grow"></span>
    <button class="primary" disabled={search.busy} onclick={search.run}>{texts.search.search}</button>
  </div>

  {#if search.error}
    <p class="error">{search.error}</p>
  {/if}
  {#if search.loading}
    <p class="muted">{texts.search.loading}</p>
  {/if}

  {#if search.result}
    {#if search.result.orders === 0}
      <p class="muted">{texts.search.noMatch}</p>
    {:else}
      <p class="summary">{searchSummary(search.result.orders, search.result.batches)}</p>
      <table>
        <thead>
          <tr>
            <th class="tick">
              <input
                type="checkbox"
                aria-label={texts.search.selectAll}
                title={texts.search.selectAll}
                checked={search.allTicked()}
                indeterminate={search.someTicked()}
                onchange={(event) => search.selectAll(event.currentTarget.checked)}
              />
            </th>
            <th>{columns.orderId}</th>
            <th>{columns.trackingId}</th>
            <th>{columns.courier}</th>
            <th>{columns.items}</th>
            <th>{columns.saved}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each search.result.rows as row (row.day + row.order_id)}
            {@const mark = reprintedMark(row)}
            <tr class:ticked={search.isTicked(row)}>
              <td class="tick">
                <input
                  id={tickId(row)}
                  type="checkbox"
                  aria-label={texts.search.tick(row.order_id)}
                  checked={search.isTicked(row)}
                  onchange={() => search.toggle(row)}
                />
              </td>
              <td class="id">
                <label for={tickId(row)}>
                  {row.order_id}
                  {#if mark}
                    <span class="mark" title={mark.title}>{mark.text}</span>
                  {/if}
                </label>
              </td>
              <td class="id"><label for={tickId(row)}>{row.tracking_id ?? ""}</label></td>
              <td class="courier"><label for={tickId(row)}>{row.courier ?? ""}</label></td>
              <td class="items"><label for={tickId(row)}>{itemsText(row)}</label></td>
              <td class="where"
                ><label for={tickId(row)}>{formatShortDay(row.day)} · {row.group}</label></td
              >
              <td class="tools">
                <button onclick={() => onOpenOutput(row.day, row.file)}>{texts.day.open}</button>
                <button
                  title={texts.search.trace}
                  aria-label={texts.search.trace}
                  onclick={() => search.toggleTrace(row)}>{search.isTraced(row) ? "▾" : "▸"}</button
                >
              </td>
            </tr>
            {#if search.isTraced(row)}
              <tr class="trace">
                <td></td>
                <td colspan="6"
                  >{#each row.trace as part, i (i)}{#if part.code}<code>{part.text}</code
                      >{:else}{part.text}{/if}{/each}</td
                >
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
      {#if search.findings.length > 0}
        <div class="error">
          <p>{texts.search.notKept}</p>
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
          {texts.search.made(search.made.number, search.made.orders, search.made.pages)}
          <button onclick={() => search.made && onOpen(search.made.path)}>{texts.day.open}</button>
        </p>
      {/if}
      {#if search.result && search.result.orders > 0}
        <div class="actions">
          <span>{texts.search.selected(tickedCount, search.result.orders)}</span>
          {#if search.reprintedBefore() > 0}
            <span class="warn">{texts.search.reprintedBefore(search.reprintedBefore())}</span>
          {/if}
          <span class="grow"></span>
          <button class="ghost" disabled={tickedCount === 0} onclick={search.clear}
            >{texts.search.clear}</button
          >
          <button class="primary" disabled={search.busy || tickedCount === 0} onclick={search.reprint}
            >{texts.search.reprint(tickedCount)}</button
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

  .controls {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    margin: 0.75rem 0 0.35rem;
  }

  .controls .label,
  .summary {
    font-weight: 600;
  }


  .summary {
    margin: 0.75rem 0 0.35rem;
  }

  .grow {
    flex: 1;
  }

  .muted {
    color: var(--text-muted);
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    padding: 0.35rem 0.4rem;
    border-bottom: 1px solid var(--border);
    color: var(--text-muted);
    font-size: 0.85rem;
    font-weight: 600;
    text-align: left;
    white-space: nowrap;
  }

  td {
    padding: 0.35rem 0.4rem;
    border-bottom: 1px solid var(--border-soft);
    vertical-align: top;
  }

  /* The tick column: the head row's tick sits above the rows' ticks. */
  .tick {
    width: 1.75rem;
  }

  tbody tr:hover {
    background: var(--surface-hover);
  }

  tr.ticked,
  tr.ticked:hover {
    background: var(--accent-soft);
  }

  /* Each cell's text is a label of the row's tick, so a click on the row ticks it. */
  td label {
    display: block;
    cursor: pointer;
  }

  .id {
    font-variant-numeric: tabular-nums;
  }

  .items {
    color: var(--text-muted);
  }

  .where,
  .courier {
    color: var(--text-muted);
    white-space: nowrap;
  }

  .tools {
    white-space: nowrap;
  }

  .mark {
    display: inline-block;
    margin-left: 0.4rem;
    padding: 0 0.45rem;
    border: 1px solid var(--warn);
    border-radius: 999px;
    color: var(--warn);
    font-size: 0.78rem;
    line-height: 1.5;
    white-space: nowrap;
  }

  .trace td {
    color: var(--text-muted);
    font-size: 0.85rem;
    word-break: break-all;
  }

  /* The reprint bar stays at the foot of the overlay while the rows scroll under it. */
  .bar {
    position: sticky;
    bottom: -1rem;
    margin: auto -1.25rem -1rem;
    padding: 0.6rem 1.25rem 0.75rem;
    border-top: 1px solid var(--border);
    background: var(--surface);
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
