<script lang="ts">
  // A batch's file list (08 › *2. New batch*): the drop hint with *Add files…*, the chosen label
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
    onAdd,
    onRemove,
    onRead,
  }: {
    listed: FileCheck[];
    totalPages: number;
    busy: boolean;
    error: string;
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

<ul class="files">
  {#each listed as file (file.path)}
    <li>
      <span class="name">{file.name}</span>
      <span class="pages">{texts.newBatch.pages(file.pages)}</span>
      <span class="time">{file.time ?? ""}</span>
      {#if file.used_in_batch !== null}
        <span class="used">{texts.newBatch.usedInBatch(file.used_in_batch)}</span>
      {/if}
      <button class="remove" title={texts.newBatch.removeTitle} onclick={() => onRemove(file.path)}
        >{texts.newBatch.remove}</button
      >
    </li>
  {/each}
</ul>

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
  .drop {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 1.5rem;
    margin-bottom: 1rem;
    border: 2px dashed #b8b8b8;
    border-radius: 8px;
    color: #4a4a4a;
  }

  .files {
    list-style: none;
    margin: 0 0 1rem;
    padding: 0;
  }

  .files li {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0.25rem;
    border-bottom: 1px solid #ececec;
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
    min-width: 3.5rem;
    text-align: right;
    color: #6a6a6a;
    font-variant-numeric: tabular-nums;
  }

  .used {
    color: #c07a00;
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

  @media (prefers-color-scheme: dark) {
    .drop {
      color: #b0b0b0;
      border-color: #555555;
    }

    .files li {
      border-color: #3a3a3a;
    }

    .time {
      color: #b0b0b0;
    }
  }
</style>
