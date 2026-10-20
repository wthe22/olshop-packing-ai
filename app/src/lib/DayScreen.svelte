<script lang="ts">
  // The Day screen: what has been saved today. Batches come from state.json,
  // each with its stored warnings and each saved-PDF row with its **Printed** tick (a note);
  // *New batch* starts the batch flow.
  import { texts } from "../texts";
  import type { BatchView, DayOverview, PdfView } from "./api/day";

  let {
    overview,
    saved,
    reading,
    onOpen,
    onOpenFolder,
    onSetPrinted,
    onNewBatch,
    onAmend,
    onRevert,
  }: {
    overview: DayOverview | null;
    saved: string | null;
    // *Amend*'s read of the batch's own files (`read-progress`): its page progress is shown until
    // the Plan screen opens; `null` when nothing reads.
    reading: { page: number; pages: number } | null;
    onOpen: (path: string) => void;
    onOpenFolder: (path: string) => void;
    // The **Printed** tick of one saved-PDF row: the owner ticks it to note that
    // this saved PDF has been printed; it is written to the day's `state.json` the moment it is
    // clicked.
    onSetPrinted: (batch: BatchView, pdf: PdfView, printed: boolean) => void;
    onNewBatch: () => void;
    onAmend: (batch: BatchView) => void;
    onRevert: (batch: BatchView) => void;
  } = $props();

  // The batch numbers whose *Warnings ▸* is expanded, and those whose *Saved with `<n>` errors ▸*
  // is.
  let openWarnings = $state<number[]>([]);
  let openErrors = $state<number[]>([]);

  function toggleWarnings(batch: number) {
    openWarnings = openWarnings.includes(batch)
      ? openWarnings.filter((item) => item !== batch)
      : [...openWarnings, batch];
  }

  function toggleErrors(batch: number) {
    openErrors = openErrors.includes(batch)
      ? openErrors.filter((item) => item !== batch)
      : [...openErrors, batch];
  }

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
    <button class="primary" onclick={onNewBatch}>{texts.day.newBatch}</button>
  </div>

  {#if saved}
    <p class="saved">{saved}</p>
  {/if}

  {#if reading}
    <div class="reading">
      <p class="muted">{texts.day.amendReading(reading.page, reading.pages)}</p>
      <progress max={reading.pages} value={reading.page}></progress>
    </div>
  {/if}

  {#if !overview}
    <p class="muted">{texts.day.loading}</p>
  {:else if overview.batches.length === 0}
    <p class="muted">{texts.day.nothing}</p>
  {:else}
    {#each overview.batches as batch, i (batch.batch)}
      <section class="batch">
        <div class="batch-head">
          <span class="head-text">
            {texts.day.batch(batch.batch, batch.time, batch.files, batch.orders)}
          </span>
          {#if i === 0}
            <span class="grow"></span>
            <button onclick={() => onAmend(batch)}>{texts.day.amend}</button>
            <button onclick={() => onRevert(batch)}>{texts.day.revert}</button>
          {/if}
        </div>
        {#each batch.pdfs as pdf (pdf.number)}
          <div class="row">
            <span class="num">{pdf.number}</span>
            <span class="code">{pdf.code}</span>
            <span class="name">{pdf.name}</span>
            <span class="counts">{texts.day.counts(pdf.orders, pdf.runs)}</span>
            <label class="printed">
              <input
                type="checkbox"
                checked={pdf.printed}
                aria-label={texts.day.printedLabel(pdf.file)}
                title={texts.day.printedLabel(pdf.file)}
                onchange={(event) => onSetPrinted(batch, pdf, event.currentTarget.checked)}
              />
            </label>
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
          {#if batch.errors.length > 0}
            <button class="link errors" onclick={() => toggleErrors(batch.batch)}>
              {texts.day.savedWithErrors(batch.errors.length)}
            </button>
          {/if}
          {#if batch.warnings.length === 0}
            <span class="muted">{texts.day.warningsNone}</span>
          {:else}
            <button class="link" onclick={() => toggleWarnings(batch.batch)}>
              {texts.day.warnings(batch.warnings.length)}
            </button>
          {/if}
          <span class="grow"></span>
          <button onclick={() => onOpenFolder(batch.folder)}
            >{texts.day.openFolder}</button
          >
        </div>
        {#if openErrors.includes(batch.batch)}
          <ul class="warnings errors">
            {#each batch.errors as error}
              <li>{error}</li>
            {/each}
          </ul>
        {/if}
        {#if openWarnings.includes(batch.batch)}
          <ul class="warnings">
            {#each batch.warnings as warning}
              <li>{warning}</li>
            {/each}
          </ul>
        {/if}
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

  .saved {
    margin: 0 0 0.75rem;
    padding: 0.4rem 0.75rem;
    border: 1px solid #a6d8a6;
    border-radius: 4px;
    background: #eaf7ea;
    color: #1f6f2f;
  }

  /* *Amend*'s read of the batch's own files, shown with a progress bar
     until the Plan screen opens. */
  .reading {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin: 0 0 0.75rem;
  }

  .reading progress {
    width: 100%;
    height: 1.1rem;
  }

  .reading .muted {
    margin: 0;
  }

  .link {
    padding: 0;
    border: none;
    background: none;
    color: #1f6feb;
    cursor: pointer;
  }

  .warnings {
    margin: 0;
    padding: 0.4rem 0.75rem 0.6rem 2rem;
    color: #4a4a4a;
    font-size: 0.9rem;
  }

  /* The check before saving's errors, kept apart from the warnings. */
  .link.errors,
  .warnings.errors {
    color: #8a1f1f;
  }

  .batch {
    border: 1px solid #d5d5d5;
    border-radius: 6px;
    background: #ffffff;
    margin-bottom: 0.75rem;
  }

  .batch-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid #ececec;
    font-weight: 600;
  }

  .batch-head .grow {
    flex: 1;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0.75rem;
    /* The row's *Open* button and the **Printed** tick share this height, so the tick is as large
       as the button beside it. The value is the button's own box: its text line,
       its padding and its border. */
    --control-height: calc(1.45rem + 0.7rem + 2px);
  }

  .row button,
  .printed input {
    height: var(--control-height);
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

  /* The **Printed** tick beside *Open*: a note only. It carries no word — the tick
     alone says the saved PDF is printed — and it is as large as the button beside it. */
  .printed {
    display: flex;
    align-items: center;
  }

  .printed input {
    width: var(--control-height);
    /* No default margin: the tick is exactly as tall as the button, so the row keeps its height. */
    margin: 0;
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
    .muted,
    .warnings {
      color: #b0b0b0;
    }

    .link.errors,
    .warnings.errors {
      color: #f0b8b8;
    }

    .saved {
      border-color: #4a7a4a;
      background: #1f3320;
      color: #a6d8a6;
    }
  }
</style>
