<script lang="ts">
  // A batch's file list: the drop hint with *Add files…*, the chosen label
  // PDFs with their pages, download time and used-in-batch lines and *Remove*, and the foot with
  // the selection summary and *Read labels*. The screen keeps the list and runs the commands;
  // nothing is read or written here.
  import type { FileCheck } from "../lib/api/batch";
  import { texts } from "../texts";

  let {
    listed,
    totalPages,
    busy,
    error,
    note,
    chips,
    onAdd,
    onRemove,
    onRead,
  }: {
    listed: FileCheck[];
    totalPages: number;
    busy: boolean;
    error: string;
    // The files just chosen that were already listed by their bytes, named with the listed one.
    note: string;
    // A file's group chip and line when it holds a repeat, by its number in the list (from 1).
    chips: Map<number, { chip: string; line: string }>;
    onAdd: () => void;
    onRemove: (path: string) => void;
    onRead: () => void;
  } = $props();
</script>

<div class="drop">
  <span>{texts.newBatch.dropHint}</span>
  <button disabled={busy} onclick={onAdd}>{texts.newBatch.addFiles}</button>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}
{#if note}
  <p class="note">{note}</p>
{/if}

{#if listed.length > 0}
  <ul class="files">
    {#each listed as file, i (file.path)}
      {@const chip = chips.get(i + 1)}
      <li>
        {#if chip}<span class="chip">▌{chip.chip}</span>{/if}
        <span class="name">{file.name}</span>
        <span class="pages">{texts.newBatch.pages(file.pages)}</span>
        <span class="time">{texts.newBatch.downloaded(file.time)}</span>
        {#if file.used_in_batch !== null}
          <span class="used">{texts.newBatch.usedInBatch(file.used_in_batch)}</span>
        {/if}
        {#if chip}<span class="used">{chip.line}</span>{/if}
        <button class="remove" title={texts.newBatch.removeTitle} onclick={() => onRemove(file.path)}
          >{texts.newBatch.remove}</button
        >
      </li>
    {/each}
  </ul>
{/if}

<div class="foot">
  <span class="selected">
    {#if listed.length > 0}
      {texts.newBatch.selected(listed.length, totalPages)}
    {/if}
  </span>
  <button class="primary" disabled={listed.length === 0} onclick={onRead}>
    {texts.newBatch.read}
  </button>
</div>

<style>
  .note {
    color: var(--text-muted);
  }

  .chip {
    color: var(--warn);
    font-weight: 600;
  }

  .drop {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 1.5rem;
    margin-bottom: 1rem;
    border: 2px dashed var(--accent-border);
    border-radius: var(--radius-lg);
    background: var(--surface-2);
    color: var(--text-muted);
  }

  .files {
    list-style: none;
    margin: 0 0 1rem;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
    overflow: hidden;
  }

  .files li {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.45rem 0.75rem;
    border-bottom: 1px solid var(--border-soft);
  }

  .files li:last-child {
    border-bottom: none;
  }

  .name {
    flex: 1;
    overflow-wrap: anywhere;
  }

  .pages {
    min-width: 5rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .time {
    min-width: 7.5rem;
    text-align: right;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .used {
    color: var(--warn);
    min-width: 8rem;
  }

  .remove {
    padding: 0.1rem 0.5rem;
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding-top: 0.5rem;
  }

  .selected {
    flex: 1;
    font-weight: 600;
  }
</style>
