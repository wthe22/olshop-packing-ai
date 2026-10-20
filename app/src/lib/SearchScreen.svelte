<script lang="ts">
  // Search: find a saved order by the same condition boxes as Categories,
  // over *This day* (the day in the title bar, the default) or *All days*. It only reads
  // `state.json`: nothing is changed or saved, and an order of an open draft is never found.
  // A row's *Open* goes through the app shell (`open_path`), as on the Day screen.
  import * as search from "../lib/api/search";
  import { formatDay, rowItemsText, searchFields, searchSummary, texts } from "../texts";
  import BoxesEditor from "./BoxesEditor.svelte";
  import { newTree, toRustTree, type UiTree } from "./conditions";

  let {
    onOpen,
  }: {
    // *Open* on a row: the app shell opens that saved PDF in the PC's viewer.
    onOpen: (path: string) => void;
  } = $props();

  // The condition boxes: the same component as Categories, with the fields Search offers.
  let tree = $state<UiTree>(newTree());
  // The scope: *This day* is the default.
  let scope = $state<search.SearchScope>("day");
  let busy = $state(false);
  // A bad condition, or a read error, in plain text.
  let error = $state("");
  // The last search's rows; `null` before the first *Search*, and after an error or any change,
  // so the rows on screen always match the boxes and the scope shown.
  let result = $state<search.SearchResults | null>(null);

  const scopes: { value: search.SearchScope; label: string }[] = [
    { value: "day", label: texts.search.scopeDay },
    { value: "all", label: texts.search.scopeAll },
  ];

  function invalidate() {
    error = "";
    result = null;
  }

  function setScope(value: search.SearchScope) {
    scope = value;
    invalidate();
  }

  async function run() {
    busy = true;
    invalidate();
    try {
      result = await search.searchOrders(toRustTree(tree), scope);
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }
</script>

<div class="search">
  <div class="controls">
    <span class="label">{texts.search.scope}</span>
    {#each scopes as option (option.value)}
      <label>
        <input
          type="radio"
          name="search-scope"
          value={option.value}
          checked={scope === option.value}
          onchange={() => setScope(option.value)}
        />
        {option.label}
      </label>
    {/each}
    <span class="grow"></span>
    <button class="primary" disabled={busy} onclick={run}>{texts.search.search}</button>
  </div>

  <BoxesEditor {tree} fields={searchFields} onEdit={invalidate} />

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if result}
    {#if result.orders === 0}
      <p class="muted">{texts.search.noMatch}</p>
    {:else}
      <p class="summary">{searchSummary(result.orders, result.batches)}</p>
      <table>
        <tbody>
          {#each result.rows as row (row.path + row.order_id)}
            <tr>
              {#if scope === "all"}
                <td class="day">{formatDay(row.day)}</td>
              {/if}
              <td class="order">{row.order_id}</td>
              <td class="tracking">{row.tracking_id ?? ""}</td>
              <td class="courier">{row.courier ?? ""}</td>
              <td class="items">{rowItemsText(row)}</td>
              <td class="batch">{texts.search.batch(row.batch)}</td>
              <td class="file">{row.file}</td>
              <td class="tools">
                <button onclick={() => onOpen(row.path)}>{texts.day.open}</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</div>

<style>
  .controls {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    margin-bottom: 0.75rem;
  }

  .controls .label {
    font-weight: 600;
  }

  .controls label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .grow {
    flex: 1;
  }

  .summary {
    margin: 0.75rem 0 0.35rem;
    font-weight: 600;
  }

  .muted {
    margin-top: 0.75rem;
    color: #6a6a6a;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  td {
    padding: 0.35rem 0.4rem;
    border-bottom: 1px solid #ececec;
    vertical-align: top;
  }

  .day {
    color: #6a6a6a;
    white-space: nowrap;
  }

  .order,
  .tracking {
    font-variant-numeric: tabular-nums;
  }

  .items {
    color: #4a4a4a;
  }

  .batch {
    color: #6a6a6a;
    white-space: nowrap;
  }

  .file {
    color: #6a6a6a;
    font-size: 0.9rem;
  }

  .tools {
    width: 5rem;
    white-space: nowrap;
  }

  @media (prefers-color-scheme: dark) {
    td {
      border-color: #3a3a3a;
    }

    .day,
    .batch,
    .file,
    .muted {
      color: #b0b0b0;
    }

    .items {
      color: #b0b0b0;
    }
  }
</style>
