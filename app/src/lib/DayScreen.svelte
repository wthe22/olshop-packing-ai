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
    writing,
    onOpen,
    onOpenOutput,
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
    // A write the owner waits for runs (its dialog is the shell's): *Day summary* waits.
    writing: boolean;
    // A reprint's file.
    onOpen: (path: string) => void;
    // A group PDF or summary PDF of the date, by its file name: one that is not there any more is
    // written again first.
    onOpenOutput: (file: string) => void;
    onOpenFolder: (path: string) => void;
    // The **Printed** tick of one group PDF: the owner ticks it to note that the file has been
    // printed; it is written to the date's journal the moment it is clicked.
    onSetPrinted: (batch: BatchView, pdf: GroupPdfView, printed: boolean) => void;
    onNewBatch: () => void;
    onDaySummary: () => void;
    onAmend: (batch: BatchView) => void;
    onRevert: (batch: BatchView) => void;
    // **Restore** an output the open-time check found changed.
    onRestore: (file: string) => void;
  } = $props();

  // The batch numbers whose *Warnings ▸*, *Saved with `<n>` errors ▸* and *Left out ▸* are
  // expanded.
  let openWarnings = $state<number[]>([]);
  let openErrors = $state<number[]>([]);
  let openLeftOut = $state<number[]>([]);

  const toggled = (list: number[], batch: number) =>
    list.includes(batch) ? list.filter((item) => item !== batch) : [...list, batch];

  function leftOutLine(copy: LeftOutView): string {
    const where = texts.day.pages(copy.input, copy.page_start, copy.page_end);
    return copy.reason === "repeat"
      ? texts.day.leftOutRepeat(copy.order_id, where, copy.kept_input, copy.kept_page)
      : texts.day.leftOutSaved(copy.order_id, where, copy.batch);
  }
</script>

{#snippet fileButton(file: string, problem: FileProblem | null)}
  {#if problem}
    <span class="problem">{texts.day.problem(problem)}</span>
  {/if}
  {#if problem === "changed"}
    <button onclick={() => onRestore(file)}>{texts.day.restore}</button>
  {:else}
    <button onclick={() => onOpenOutput(file)}>{texts.day.open}</button>
  {/if}
{/snippet}

<main>
  <div class="bar">
    <h1>{texts.day.thisDay}</h1>
    <span class="stats">
      {#if overview}
        {@const { batches, orders, pdfs } = overview.totals}
        {#each texts.day.stats(batches, orders, pdfs, overview.reprints.length) as stat (stat.word)}
          <span class="stat"><b>{stat.n}</b> {stat.word}</span>
        {/each}
      {/if}
    </span>
    <button
      title={texts.day.daySummaryTitle}
      disabled={writing || !overview || overview.batches.length === 0}
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
            <span class="batch-name">{texts.day.batch(batch.batch)}</span>
            <span class="facts"
              >{texts.day.batchFacts(batch.time, batch.files, batch.orders)}{#if batch.amended}{texts
                  .day.amended}{/if}</span
            >
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
            {@render fileButton(pdf.file, pdf.problem)}
          </div>
        {/each}
        <div class="batch-foot">
          <span class="packing">
            {texts.day.summaryPdf}
            {@render fileButton(batch.summary, batch.summary_problem)}
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
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.9rem;
  }

  h1 {
    margin: 0 0.35rem 0 0;
    font-size: 1.3rem;
    font-weight: 700;
  }

  /* The day's counts as pills beside the title, each number in bold. */
  .stats {
    display: flex;
    flex: 1;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .stat {
    padding: 0.1rem 0.65rem;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface);
    color: var(--text-muted);
    font-size: 0.9rem;
    white-space: nowrap;
  }

  .stat b {
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .muted {
    color: var(--text-muted);
  }

  .saved {
    margin: 0 0 0.75rem;
    padding: 0.4rem 0.75rem;
    border: 1px solid var(--ok-border);
    border-radius: var(--radius);
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
    margin-bottom: 0.9rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
    overflow: hidden;
  }

  .batch-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-soft);
    background: var(--surface-2);
  }

  .head-text {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.25rem 0.75rem;
  }

  .batch-name {
    font-size: 1.05rem;
    font-weight: 700;
  }

  .facts {
    color: var(--text-muted);
  }

  .batch-head .grow {
    flex: 1;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0.75rem;
    border-bottom: 1px solid var(--border-soft);
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
    padding: 0.05rem 0.5rem;
    border-radius: var(--radius);
    background: var(--chip);
    color: var(--chip-text);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    text-align: center;
    white-space: nowrap;
  }

  .name {
    flex: 1;
    font-weight: 600;
  }

  /* The **Printed** tick beside *Open*: a note only. It carries no word — the tick
     alone says the saved PDF is printed — and it is as large as the button beside it. */
  .printed {
    display: flex;
    align-items: center;
  }

  .printed input {
    width: var(--control-height);
    border-radius: var(--radius);
  }

  .printed input:checked {
    border-color: var(--ok-solid);
    background-color: var(--ok-solid);
  }

  .batch-foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 1rem;
    padding: 0.5rem 0.75rem;
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
