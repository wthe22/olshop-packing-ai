<script lang="ts">
  // The Day screen: what has been saved on the date. Batches come from the date's journal, each
  // with its warnings and each group PDF with its *Sent* mark and **Printed** tick (notes), then the
  // date's reprints; *New batch* starts the batch flow and *Day summary* writes the date's summary
  // PDF.
  import { texts } from "../texts";
  import { formatDay } from "../dates";
  import type { BatchView, DayOverview, GroupPdfView, LeftOutView } from "./api/day";
  import type { SentMark } from "./dayOutputs.svelte";
  import GroupPdfRow from "./GroupPdfRow.svelte";
  import OutputButtons from "./OutputButtons.svelte";
  import ReprintsCard from "./ReprintsCard.svelte";
  import DaySummaryButton from "./DaySummaryButton.svelte";
  import { accountsOf } from "./dayAccounts";

  let {
    day,
    today,
    overview,
    dataFolder,
    saved,
    reading,
    writing,
    onOpen,
    onOpenOutput,
    onPrint,
    onOpenFolder,
    onUnsend,
    onSetPrinted,
    onNewBatch,
    onDaySummary,
    onAccountSummary,
    onAmend,
    onRevert,
    onRestore,
  }: {
    // The open date, the screen's title, and today, which it is marked as.
    day: string;
    today: string;
    overview: DayOverview | null;
    // The data folder, for the note about data of an older version.
    dataFolder: string | null;
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
    // **Print** an output of the date, by its file name; a group PDF names the group it marks
    // *Sent*.
    onPrint: (file: string, mark: SentMark | null) => void;
    onOpenFolder: (path: string) => void;
    // The *Sent* mark's ✕.
    onUnsend: (batch: BatchView, pdf: GroupPdfView) => void;
    // The **Printed** tick of one group PDF: the owner ticks it to note that the file has been
    // printed; it is written to the date's journal the moment it is clicked.
    onSetPrinted: (batch: BatchView, pdf: GroupPdfView, printed: boolean) => void;
    onNewBatch: () => void;
    // `null`: the summary of every account.
    onDaySummary: (account: string | null) => void;
    // One account's summary of a batch, written and opened.
    onAccountSummary: (batch: BatchView, account: string) => void;
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

<main>
  <div class="bar">
    <h1>{formatDay(day)}</h1>
    {#if day === today}<span class="today">{texts.day.today}</span>{/if}
    <span class="stats">
      {#if overview}
        {@const { batches, orders, pdfs } = overview.totals}
        {#each texts.day.stats(batches, orders, pdfs, overview.reprints.length) as stat (stat.word)}
          <span class="stat"><b>{stat.n}</b> {stat.word}</span>
        {/each}
      {/if}
    </span>
    <DaySummaryButton
      accounts={accountsOf(overview?.batches ?? [])}
      disabled={writing || !overview || overview.batches.length === 0}
      onWrite={onDaySummary}
    />
    <button class="primary" onclick={onNewBatch}>{texts.day.newBatch}</button>
  </div>

  {#if saved}
    <p class="saved">{saved}</p>
  {/if}

  {#if overview}
    {#each overview.problems as problem}
      <p class="problem">{problem}</p>
    {/each}
    {#each overview.older as older (older.version)}
      <p class="muted">
        {texts.day.older(older.version, older.names)}
        {#if dataFolder}
          <button onclick={() => onOpenFolder(dataFolder)}>{texts.day.openFolder}</button>
        {/if}
      </p>
    {/each}
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
              >{texts.day.batchFacts(batch.time, batch.files, batch.orders)} · {texts.day.batchAccounts(
                batch.accounts,
              )}{#if batch.amended}{texts.day.amended}{/if}</span
            >
          </span>
          {#if i === 0}
            <span class="grow"></span>
            <button onclick={() => onAmend(batch)}>{texts.day.amend}</button>
            <button onclick={() => onRevert(batch)}>{texts.day.revert}</button>
          {/if}
        </div>
        {#each batch.pdfs as pdf (pdf.group)}
          <GroupPdfRow
            {batch}
            {pdf}
            {onRestore}
            {onPrint}
            {onOpenOutput}
            {onUnsend}
            {onSetPrinted}
          />
        {/each}
        <div class="batch-foot">
          <span class="packing">
            {texts.day.summaryPdf}
            <OutputButtons
              file={batch.summary}
              problem={batch.summary_problem}
              mark={null}
              printable={false}
              {onRestore}
              {onPrint}
              {onOpenOutput}
            />
            {#if batch.accounts.length > 1}
              {#each batch.accounts as { account } (account)}
                <button
                  title={texts.day.accountSummaryTitle(account)}
                  disabled={writing}
                  onclick={() => onAccountSummary(batch, account)}>{account}</button
                >
              {/each}
            {/if}
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
      <ReprintsCard
        day={overview.day}
        reprints={overview.reprints}
        {onOpen}
        onPrint={(file) => onPrint(file, null)}
      />
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

  .today {
    margin-right: 0.35rem;
    padding: 0.05rem 0.55rem;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent-text);
    font-size: 0.8rem;
    font-weight: 600;
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
