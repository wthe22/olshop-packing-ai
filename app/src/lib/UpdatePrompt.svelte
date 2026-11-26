<script lang="ts">
  // The question at start when the data folder holds data this version keeps more of (start.md,
  // *Update the data folder?*): what changes on each date, then **Update** with its progress, or
  // **Open read-only**. No date opens before the answer.
  import type { MigrationView } from "./api/start";
  import { updateTexts as words } from "../updateTexts";
  import { formatDay } from "../dates";
  import StartCard from "./StartCard.svelte";
  import { Update } from "./update.svelte";

  let {
    version,
    dates,
    // **Open read-only** that failed.
    error,
    onDone,
    onReadOnly,
  }: {
    version: string;
    dates: MigrationView[];
    error: string;
    onDone: () => void;
    onReadOnly: () => void;
  } = $props();

  const update = new Update();
  const batches = $derived(dates.reduce((sum, date) => sum + date.batches, 0));
  const v07Dates = $derived(dates.filter((date) => date.v0_7).length);
  const skipped = $derived(update.lines.some((line) => line.skipped));
</script>

<StartCard title={words.title} {version}>
  <p class="explain">{words.explain(batches, v07Dates)} {words.readOnlyExplain}</p>
  <table>
    <thead>
      <tr><th>{words.dateHead}</th><th>{words.changesHead}</th></tr>
    </thead>
    <tbody>
      {#each dates as date (date.date)}
        <tr>
          <td class="date">{formatDay(date.date)}</td>
          <td>{words.changes(date.batches, date.orders, date.older_views, date.v0_7)}</td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if update.lines.length > 0 || update.current}
    <ul class="progress">
      {#each update.lines as line, i (i)}
        <li class:skipped={line.skipped}>{line.text}</li>
      {/each}
      {#if update.current}
        <li class="running">{update.current}</li>
      {/if}
    </ul>
  {/if}
  {#if update.done}
    <p class="done">{skipped ? words.doneSkipped : words.done}</p>
  {/if}
  {#if update.error}
    <p class="error">{words.failed(update.error)}</p>
  {:else if error}
    <p class="error">{error}</p>
  {/if}

  <div class="buttons">
    {#if update.done}
      <button class="primary" onclick={onDone}>{words.continue}</button>
    {:else}
      <button disabled={update.running} onclick={onReadOnly}>{words.readOnly}</button>
      <button class="primary" disabled={update.running} onclick={update.run}>{words.update}</button>
    {/if}
  </div>
</StartCard>

<style>
  .explain {
    margin: 0 0 1.25rem;
    color: var(--text-muted);
  }

  table {
    width: 100%;
    border: 1px solid var(--border);
    border-collapse: separate;
    border-spacing: 0;
    border-radius: var(--radius);
    overflow: hidden;
  }

  th {
    padding: 0.4rem 0.75rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-muted);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-align: left;
    text-transform: uppercase;
  }

  td {
    padding: 0.45rem 0.75rem;
  }

  tr + tr td {
    border-top: 1px solid var(--border);
  }

  .date {
    white-space: nowrap;
  }

  .progress {
    margin: 1rem 0 0;
    padding: 0;
    list-style: none;
    font-size: 0.9rem;
    color: var(--text-muted);
  }

  .progress li {
    padding: 0.15rem 0;
  }

  .progress .running {
    color: var(--text);
  }

  .progress .skipped {
    color: var(--danger);
  }

  .done {
    margin: 0.75rem 0 0;
  }

  .error {
    margin: 0.75rem 0 0;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    margin-top: 1.5rem;
  }
</style>
