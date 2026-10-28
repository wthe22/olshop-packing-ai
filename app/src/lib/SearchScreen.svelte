<script lang="ts">
  // Search: saved orders of the open workspace found by a filter made with the boxes, over a range
  // of dates or all of them. Each row names the group PDF and the page the order is on, opens its
  // trace line and carries its *Reprinted* mark; the ticked rows are reprinted into one PDF from the
  // bar at the foot. Enter in a box searches, so a scanned tracking ID finds its order at once. The
  // state and the commands are `search.svelte.ts`.
  import { onMount } from "svelte";
  import { Search, reprintedMark } from "./search.svelte";
  import type { SearchRow } from "./api/search";
  import { RANGE_PRESETS, type RangePreset } from "./searchRange";
  import { searchSummary, texts } from "../texts";
  import { conditionFields } from "../fields";
  import { formatShortDay } from "../dates";
  import BoxesEditor from "./BoxesEditor.svelte";
  import Icon from "./Icon.svelte";

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
  const words = texts.search;
  const columns = words.columns;

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

  // A click on the row's text; the tick, **Open** and the chevron have their own.
  function onRowClick(event: MouseEvent, row: SearchRow) {
    if ((event.target as Element).closest("button, input")) return;
    search.clickRow(row, event.detail, (window.getSelection()?.toString() ?? "") !== "");
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="search">
  <div class="query">
    <div bind:this={filters}>
      <BoxesEditor
        tree={search.tree}
        fields={conditionFields}
        couriers={search.couriers}
        onEdit={search.invalidate}
      />
    </div>
    <div class="controls">
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
      {#if search.preset !== "all"}
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
      {/if}
      <span class="grow"></span>
      <button class="primary" disabled={search.busy} onclick={search.run}
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
      <p class="muted">{words.noMatch}</p>
    {:else}
      <p class="summary">{searchSummary(search.result.orders, search.result.batches)}</p>
      <div class="results">
        <table>
          <thead>
            <tr>
              <th class="tick">
                <input
                  type="checkbox"
                  aria-label={words.selectAll}
                  title={words.selectAll}
                  checked={search.allTicked()}
                  indeterminate={search.someTicked()}
                  onchange={(event) => search.selectAll(event.currentTarget.checked)}
                />
              </th>
              <th>{columns.orderId}</th>
              <th>{columns.trackingId}</th>
              <th>{columns.courier}</th>
              <th>{columns.items}</th>
              <th>{columns.date}</th>
              <th>{columns.file}</th>
              <th class="page">{columns.page}</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {#each search.result.rows as row (row.day + row.order_id)}
              {@const mark = reprintedMark(row)}
              {@const traced = search.isTraced(row)}
              <tr
                class:ticked={search.isTicked(row)}
                class:traced
                onclick={(event) => onRowClick(event, row)}
              >
                <td class="tick">
                  <input
                    type="checkbox"
                    aria-label={words.tick(row.order_id)}
                    checked={search.isTicked(row)}
                    onchange={() => search.toggle(row)}
                  />
                </td>
                <td class="id">
                  {row.order_id}
                  {#if mark}<span class="mark" title={mark.title}>{mark.text}</span>{/if}
                </td>
                <td class="id">{row.tracking_id ?? ""}</td>
                <td class="muted nowrap" class:faint={!row.courier}>{row.courier ?? words.noCourier}</td>
                <td class="items">
                  {#each row.items as item, i (i)}<div>{words.item(item.name, item.quantity)}</div>{/each}
                </td>
                <td class="muted nowrap">{formatShortDay(row.day)}</td>
                <td class="file" title={row.file}>{row.file.replace(/\.pdf$/i, "")}</td>
                <td class="page">{words.pages(row.first_page, row.last_page)}</td>
                <td class="tools">
                  <button
                    class="small"
                    title={words.openFile(row.file)}
                    onclick={() => onOpenOutput(row.day, row.file)}>{words.open}</button
                  >
                  <button
                    class="icon chevron"
                    class:open={traced}
                    title={words.trace}
                    aria-label={words.trace}
                    aria-expanded={traced}
                    onclick={() => search.toggleTrace(row)}><Icon name="chevron" size={16} /></button
                  >
                </td>
              </tr>
              {#if traced}
                <tr class="trace">
                  <td></td>
                  <td colspan="8"
                    >{#each row.trace as part, i (i)}{#if part.code}<code>{part.text}</code
                        >{:else}{part.text}{/if}{/each}</td
                  >
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
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

  /* The filter and the dates in one box; the boxes editor's outer box is this one. */
  .query {
    padding: 0.6rem 0.75rem 0.7rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
  }

  .query :global(.group.root) {
    padding: 0;
    border: none;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    margin-top: 0.6rem;
    padding-top: 0.6rem;
    border-top: 1px solid var(--border-soft);
  }

  .controls .label {
    font-weight: 600;
  }

  .to {
    color: var(--text-muted);
  }

  .controls button.primary {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .summary {
    margin: 1rem 0 0.4rem;
    font-weight: 600;
  }

  .grow {
    flex: 1;
  }

  .muted {
    color: var(--text-muted);
  }

  .faint {
    color: var(--text-faint);
  }

  .nowrap {
    white-space: nowrap;
  }

  .results {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    overflow: hidden;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    padding: 0.45rem 0.6rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-muted);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-align: left;
    text-transform: uppercase;
    white-space: nowrap;
  }

  td {
    padding: 0.45rem 0.6rem;
    border-top: 1px solid var(--border-soft);
    vertical-align: top;
  }

  tbody tr:first-child td {
    border-top: none;
  }

  /* The tick column: the head row's tick sits above the rows' ticks. */
  .tick {
    width: 2rem;
    padding-right: 0;
  }

  tbody tr {
    cursor: default;
  }

  tbody tr:hover {
    background: var(--surface-hover);
  }

  tr.ticked,
  tr.ticked:hover {
    background: var(--accent-soft);
  }

  .id {
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .items {
    width: 100%;
  }

  .file {
    max-width: 14rem;
    overflow: hidden;
    color: var(--text-muted);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .page {
    font-variant-numeric: tabular-nums;
    text-align: right;
    white-space: nowrap;
  }

  .tools {
    padding-top: 0.3rem;
    padding-bottom: 0.3rem;
    white-space: nowrap;
  }

  button.small {
    padding: 0.1rem 0.6rem;
    font-size: 0.88rem;
  }

  button.chevron {
    margin-left: 0.15rem;
    padding: 0.2rem;
    vertical-align: middle;
  }

  button.chevron :global(svg) {
    transition: transform 0.1s;
  }

  button.chevron.open :global(svg) {
    transform: rotate(90deg);
  }

  .mark {
    display: inline-block;
    margin-left: 0.4rem;
    padding: 0 0.45rem;
    border: 1px solid var(--warn);
    border-radius: 999px;
    color: var(--warn);
    font-size: 0.75rem;
    line-height: 1.5;
    white-space: nowrap;
  }

  tr.traced td {
    border-bottom: none;
  }

  .trace td {
    padding-top: 0;
    border-top: none;
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
