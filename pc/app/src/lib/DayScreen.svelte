<script lang="ts">
  // The Day screen (08 › *1. Day*): what has been saved today. Batches come from state.json;
  // *New batch* is a disabled placeholder until 2.9.
  import { texts } from "../texts";
  import type { DayOverview } from "./api";

  let {
    overview,
    onOpen,
    onOpenFolder,
    onNewBatch,
  }: {
    overview: DayOverview | null;
    onOpen: (path: string) => void;
    onOpenFolder: (path: string) => void;
    onNewBatch: () => void;
  } = $props();

  const join = (folder: string, file: string) => `${folder}\\${file}`;
</script>

<main>
  <div class="bar">
    <span class="summary">
      {#if overview}
        {texts.day.summary(
          overview.totals.batches,
          overview.totals.orders,
          overview.totals.pdfs,
        )}
      {/if}
    </span>
    <button
      disabled
      title={texts.comingSoon.newBatch}
      onclick={onNewBatch}>{texts.day.newBatch}</button
    >
  </div>

  {#if !overview}
    <p class="muted">{texts.day.loading}</p>
  {:else if overview.batches.length === 0}
    <p class="muted">{texts.day.nothing}</p>
  {:else}
    {#each overview.batches as batch (batch.batch)}
      <section class="batch">
        <div class="batch-head">
          {texts.day.batch(
            batch.batch,
            batch.time,
            batch.files,
            batch.orders,
          )}
        </div>
        {#each batch.pdfs as pdf (pdf.number)}
          <div class="row">
            <span class="num">{pdf.number}</span>
            <span class="code">{pdf.code}</span>
            <span class="name">{pdf.name}</span>
            <span class="counts">{texts.day.counts(pdf.orders, pdf.runs)}</span>
            <button onclick={() => onOpen(join(batch.folder, pdf.file))}
              >{texts.day.open}</button
            >
          </div>
        {/each}
        <div class="batch-foot">
          {#each batch.packing_lists as list (list)}
            <span class="packing">
              {texts.day.packingList}
              <button onclick={() => onOpen(join(batch.folder, list))}
                >{texts.day.open}</button
              >
            </span>
          {/each}
          <span class="grow"></span>
          <button onclick={() => onOpenFolder(batch.folder)}
            >{texts.day.openFolder}</button
          >
        </div>
      </section>
    {/each}
  {/if}
</main>

<style>
  main {
    padding: 1rem;
  }

  .bar {
    display: flex;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .summary {
    flex: 1;
    font-weight: 600;
  }

  .muted {
    color: #6a6a6a;
  }

  .batch {
    border: 1px solid #d5d5d5;
    border-radius: 6px;
    background: #ffffff;
    margin-bottom: 0.75rem;
  }

  .batch-head {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid #ececec;
    font-weight: 600;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0.75rem;
  }

  .num {
    width: 2rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .code {
    width: 1.5rem;
    font-weight: 600;
  }

  .name {
    flex: 1;
  }

  .counts {
    min-width: 11rem;
    color: #4a4a4a;
  }

  .batch-foot {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 0.75rem;
    border-top: 1px solid #ececec;
  }

  .packing {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .grow {
    flex: 1;
  }

  @media (prefers-color-scheme: dark) {
    .batch {
      background: #2b2b2b;
      border-color: #444444;
    }

    .batch-head,
    .batch-foot {
      border-color: #3a3a3a;
    }

    .counts,
    .muted {
      color: #b0b0b0;
    }
  }
</style>
