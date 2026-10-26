<script lang="ts">
  // Search: saved orders of the open workspace found by a filter made with the boxes, over *This
  // day* or *All days*, each row with its trace line; the ticked rows are reprinted into one PDF.
  // The state and the commands are `search.svelte.ts`.
  import { Search, itemsText } from "./search.svelte";
  import type { SearchScope } from "./api/search";
  import { conditionFields, formatShortDay, searchSummary, texts } from "../texts";
  import BoxesEditor from "./BoxesEditor.svelte";

  let {
    day,
    onOpen,
    onReprinted,
  }: {
    // The open date: *This day*.
    day: string;
    onOpen: (path: string) => void;
    onReprinted: () => void;
  } = $props();

  const search = new Search(
    () => day,
    () => onReprinted(),
  );

  const scopes: { value: SearchScope; label: string }[] = [
    { value: "day", label: texts.search.scopeDay },
    { value: "all", label: texts.search.scopeAll },
  ];

  const tickedCount = $derived(search.ticked.length);
</script>

<div class="search">
  <BoxesEditor tree={search.tree} fields={conditionFields} onEdit={search.invalidate} />

  <div class="controls">
    <span class="label">{texts.search.scope}</span>
    {#each scopes as option (option.value)}
      <label>
        <input
          type="radio"
          name="search-scope"
          value={option.value}
          checked={search.scope === option.value}
          onchange={() => search.setScope(option.value)}
        />
        {option.label}
      </label>
    {/each}
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
      <div class="summary">
        <span>{searchSummary(search.result.orders, search.result.batches)}</span>
        <span class="grow"></span>
        <label>
          <input
            type="checkbox"
            checked={search.allTicked()}
            onchange={(event) => search.selectAll(event.currentTarget.checked)}
          />
          {texts.search.selectAll}
        </label>
      </div>
      <table>
        <tbody>
          {#each search.result.rows as row (row.day + row.order_id)}
            <tr>
              <td class="tick">
                <input
                  type="checkbox"
                  aria-label={texts.search.tick(row.order_id)}
                  checked={search.isTicked(row)}
                  onchange={() => search.toggle(row)}
                />
              </td>
              <td class="id">{row.order_id}</td>
              <td class="id">{row.tracking_id ?? ""}</td>
              <td class="courier">{row.courier ?? ""}</td>
              <td class="items">{itemsText(row)}</td>
              <td class="where">{formatShortDay(row.day)} · {row.group}</td>
              <td class="tools">
                <button onclick={() => onOpen(row.path)}>{texts.day.open}</button>
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
                <td colspan="6">{row.trace}</td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
      <div class="foot">
        {#if search.made}
          <span class="made">
            {texts.search.made(search.made.number, search.made.orders, search.made.pages)}
            <button onclick={() => search.made && onOpen(search.made.path)}>{texts.day.open}</button>
          </span>
        {/if}
        <span class="grow"></span>
        <button class="primary" disabled={search.busy || tickedCount === 0} onclick={search.reprint}
          >{texts.search.reprint(tickedCount)}</button
        >
      </div>
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
</div>

<style>
  .controls,
  .summary,
  .foot {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    margin: 0.75rem 0 0.35rem;
  }

  .controls .label,
  .summary {
    font-weight: 600;
  }

  .controls label,
  .summary label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-weight: normal;
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

  td {
    padding: 0.35rem 0.4rem;
    border-bottom: 1px solid var(--border-soft);
    vertical-align: top;
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

  .trace td {
    color: var(--text-muted);
    font-size: 0.85rem;
    word-break: break-all;
  }

  .made {
    color: var(--ok);
  }
</style>
