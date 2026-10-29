<script lang="ts">
  // One plan row: the *Use* tick, the group's code, name, counts and file name with its move
  // buttons, and the order contents of its group PDF when the row is expanded.
  import type { PlanRow } from "./api/batch";
  import { texts } from "../texts";

  let {
    row,
    open,
    onToggleUse,
    onMove,
    onRename,
    onToggleOpen,
  }: {
    row: PlanRow;
    // The row's runs are shown.
    open: boolean;
    onToggleUse: (checked: boolean) => void;
    onMove: (up: boolean) => void;
    onRename: (name: string) => void;
    onToggleOpen: () => void;
  } = $props();
</script>

<tr class:skipped={row.skipped} class:empty={row.orders === 0 && !row.skipped} class:rest={row.kind === "generated"}>
  <td class="c-use">
    {#if row.kind !== "generated"}
      <input
        type="checkbox"
        checked={!row.skipped}
        onchange={(e) => onToggleUse(e.currentTarget.checked)}
      />
    {/if}
  </td>
  <td class="c-group">
    <span class="code">{row.code}</span>
    <span class="name">{row.name}</span>
    {#if row.kind !== "entry"}<span class="tag">{texts.plan.rest}</span>{/if}
    {#if row.skipped}<span class="tag">{texts.plan.skipped}</span>{/if}
    {#if row.condition}
      <div class="condition">{row.condition}</div>
    {/if}
  </td>
  <td class="c-orders">{row.orders > 0 ? row.orders : ""}</td>
  <td class="c-runs">{row.orders > 0 ? row.runs.length : ""}</td>
  <td class="c-file">
    {#if row.orders === 0 && !row.skipped}
      <span class="no-pdf">{texts.plan.noPdf}</span>
    {:else}
      <input
        class="filename"
        value={row.file_name}
        onchange={(e) => onRename(e.currentTarget.value)}
      />
      {#if row.duplicate_of !== null}
        <div class="dup">{texts.plan.duplicate(row.duplicate_of)}</div>
      {/if}
    {/if}
  </td>
  <td class="c-tools">
    {#if row.can_move_up}
      <button title={texts.plan.moveUp} onclick={() => onMove(true)}>↑</button>
    {/if}
    {#if row.can_move_down}
      <button title={texts.plan.moveDown} onclick={() => onMove(false)}>↓</button>
    {/if}
    {#if row.runs.length > 0}
      <button title={texts.plan.expand} onclick={onToggleOpen}>▸</button>
    {/if}
  </td>
</tr>
{#if open}
  {#each row.runs as run}
    <tr class="run">
      <td class="c-use"></td>
      <td class="c-group" colspan="2">{run.contents}</td>
      <td class="c-runs run-orders">
        {run.orders} {run.orders === 1 ? "order" : "orders"}
      </td>
      <td class="c-file"></td>
      <td class="c-tools"></td>
    </tr>
  {/each}
{/if}

<style>
  td {
    padding: 0.45rem 0.4rem;
    border-bottom: 1px solid var(--border-soft);
    vertical-align: top;
  }

  td:first-child {
    padding-left: 0.75rem;
  }

  .c-orders,
  .c-runs {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .c-tools {
    white-space: nowrap;
  }

  .c-tools button {
    padding: 0.05rem 0.4rem;
    margin-right: 0.15rem;
  }

  .code {
    display: inline-block;
    min-width: 1.6rem;
    padding: 0 0.4rem;
    border-radius: var(--radius);
    background: var(--chip);
    color: var(--chip-text);
    font-weight: 700;
    text-align: center;
    white-space: nowrap;
  }

  .tag {
    margin-left: 0.4rem;
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .condition {
    color: var(--text-muted);
    font-size: 0.85rem;
    white-space: pre-wrap;
  }

  .filename {
    width: 100%;
    min-width: 12rem;
  }

  .dup {
    color: var(--danger);
    font-size: 0.85rem;
  }

  .no-pdf {
    color: var(--text-faint);
  }

  tr.skipped td,
  tr.empty td {
    color: var(--text-faint);
  }

  tr.run td {
    border-bottom: none;
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 0.9rem;
  }

  .run-orders {
    font-family: "Segoe UI", sans-serif;
  }
</style>
