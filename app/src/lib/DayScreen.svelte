<script lang="ts">
  // The Day screen: what has been saved on the date. Batches come from the date's journal, each
  // with its warnings and each group PDF with its **Printed** tick (a note), then the date's
  // reprints; *New batch* starts the batch flow and *Day summary* writes the date's summary PDF.
  import { texts } from "../texts";
  import type { BatchView, DayOverview, FileProblem, GroupPdfView, LeftOutView } from "./api/day";
  import Count from "./Count.svelte";
  import ReprintsCard from "./ReprintsCard.svelte";

  let {
    overview,
    saved,
    reading,
    onOpen,
    onOpenFolder,
    onSetPrinted,
    onNewBatch,
    onDaySummary,
    onAmend,
    onRevert,
    onRestore,
  }: {
    overview: DayOverview | null;
    saved: string | null;
    // *Amend*'s read of the batch's own files (`read-progress`): its page progress is shown until
    // the Plan screen opens; `null` when nothing reads.
    reading: { page: number; pages: number } | null;
    onOpen: (path: string) => void;
    onOpenFolder: (path: string) => void;
    // The **Printed** tick of one group PDF: the owner ticks it to note that the file has been
    // printed; it is written to the date's journal the moment it is clicked.
    onSetPrinted: (batch: BatchView, pdf: GroupPdfView, printed: boolean) => void;
    onNewBatch: () => void;
    onDaySummary: () => void;
    onAmend: (batch: BatchView) => void;
    onRevert: (batch: BatchView) => void;
    // **Restore** an output the open-time check found missing or changed.
    onRestore: (file: string) => void;
  } = $props();

  // The batch numbers whose *Warnings ▸*, *Saved with `<n>` errors ▸* and *Left out ▸* are
  // expanded.
  let openWarnings = $state<number[]>([]);
  let openErrors = $state<number[]>([]);
  let openLeftOut = $state<number[]>([]);

  const toggled = (list: number[], batch: number) =>
    list.includes(batch) ? list.filter((item) => item !== batch) : [...list, batch];

  const join = (folder: string, file: string) => `${folder}\\${file}`;

  function leftOutLine(copy: LeftOutView): string {
    const where = texts.day.pages(copy.input, copy.page_start, copy.page_end);
    return copy.reason === "repeat"
      ? texts.day.leftOutRepeat(copy.order_id, where, copy.kept_input, copy.kept_page)
      : texts.day.leftOutSaved(copy.order_id, where, copy.batch);
  }
</script>

{#snippet fileButton(folder: string, file: string, problem: FileProblem | null)}
  {#if problem}
    <span class="problem">{texts.day.problem(problem)}</span>
    <button onclick={() => onRestore(file)}>{texts.day.restore}</button>
  {:else}
    <button onclick={() => onOpen(join(folder, file))}>{texts.day.open}</button>
  {/if}
{/snippet}

<main>
  <div class="bar">
    <span class="summary">
      {#if overview}
        {texts.day.summary(
          overview.totals.batches,
          overview.totals.orders,
          overview.totals.pdfs,
          overview.reprints.length,
        )}
      {/if}
    </span>
    <button
      title={texts.day.daySummaryTitle}
      disabled={!overview || overview.batches.length === 0}
      onclick={onDaySummary}>{texts.day.daySummary}</button
    >
    <button class="primary" onclick={onNewBatch}>{texts.day.newBatch}</button>
  </div>

  {#if saved}
    <p class="saved">{saved}</p>
  {/if}

  {#if overview}
    {#each overview.problems as problem}
      <p class="problem">{problem}</p>
    {/each}
    {#if overview.old_data}
      <p class="muted">{texts.day.oldData}</p>
    {/if}
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
            {#if batch.amended}{texts.day.amended}{/if}
          </span>
          {#if i === 0}
            <span class="grow"></span>
            <button onclick={() => onAmend(batch)}>{texts.day.amend}</button>
            <button onclick={() => onRevert(batch)}>{texts.day.revert}</button>
          {/if}
        </div>
        {#each batch.pdfs as pdf (pdf.group)}
          <div class="row">
            <span class="code">{batch.batch}-{pdf.group}</span>
            <span class="name">{pdf.name}</span>
            <Count n={pdf.orders} word={texts.day.orderWord(pdf.orders)} />
            <Count n={pdf.contents} word={texts.day.contentWord(pdf.contents)} />
            <label class="printed">
              <input
                type="checkbox"
                checked={pdf.printed}
                aria-label={texts.day.printedLabel(pdf.file)}
                title={texts.day.printedLabel(pdf.file)}
                onchange={(event) => onSetPrinted(batch, pdf, event.currentTarget.checked)}
              />
            </label>
            {@render fileButton(batch.folder, pdf.file, pdf.problem)}
          </div>
        {/each}
        <div class="batch-foot">
          <span class="packing">
            {texts.day.summaryPdf}
            {@render fileButton(batch.folder, batch.summary, batch.summary_problem)}
          </span>
          {#if batch.left_out.length > 0}
            <button class="link" onclick={() => (openLeftOut = toggled(openLeftOut, batch.batch))}>
              {texts.day.leftOut(batch.left_out.length)}
            </button>
          {/if}
          {#if batch.errors.length > 0}
            <button
              class="link errors"
              onclick={() => (openErrors = toggled(openErrors, batch.batch))}
            >
              {texts.day.savedWithErrors(batch.errors.length)}
            </button>
          {/if}
          {#if batch.warnings.length === 0}
            <span class="muted">{texts.day.warningsNone}</span>
          {:else}
            <button class="link" onclick={() => (openWarnings = toggled(openWarnings, batch.batch))}>
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
        {#if openLeftOut.includes(batch.batch)}
          <ul class="warnings">
            {#each batch.left_out as copy}
              <li>{leftOutLine(copy)}</li>
            {/each}
          </ul>
        {/if}
      </section>
    {/each}
  {/if}

  {#if overview && overview.reprints.length > 0}
    {#key overview.day}
      <ReprintsCard day={overview.day} reprints={overview.reprints} {onOpen} />
    {/key}
  {/if}
</main>

<style>
  main {
    padding: 1rem;
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .summary {
    flex: 1;
    font-weight: 600;
  }

  .muted {
    color: var(--text-muted);
  }

  .saved {
    margin: 0 0 0.75rem;
    padding: 0.4rem 0.75rem;
    border: 1px solid var(--ok-border);
    border-radius: 4px;
    background: var(--ok-bg);
    color: var(--ok);
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
    color: var(--link);
    cursor: pointer;
  }

  .warnings {
    margin: 0;
    padding: 0.4rem 0.75rem 0.6rem 2rem;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  /* The check before saving's errors, kept apart from the warnings. */
  .link.errors,
  .warnings.errors {
    color: var(--danger);
  }

  .batch {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    margin-bottom: 0.75rem;
  }

  .batch-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-soft);
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

  /* The batch and group code (`2-SN`) on one line, codes of several letters included. */
  .code {
    min-width: 3.5rem;
    white-space: nowrap;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .name {
    flex: 1;
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
    border-top: 1px solid var(--border-soft);
  }

  .packing {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .grow {
    flex: 1;
  }
</style>
