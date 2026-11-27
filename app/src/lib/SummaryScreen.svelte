<script lang="ts">
  // The Summary screen (ui/summary.md): a batch's or the open date's summary read in the app, part
  // by part as its PDF prints them, without writing a file. The account list shows one account's
  // orders; **PDF** writes this summary as a PDF and opens it; **Back** returns to the Day screen.
  import { texts } from "../texts";
  import type { DayOverview } from "./api/day";
  import { summaryView, type Count, type SummaryView } from "./api/summary";
  import { accountsOf } from "./dayAccounts";
  import DayTitle from "./DayTitle.svelte";
  import { access } from "./readOnly.svelte";

  let {
    day,
    today,
    overview,
    batch,
    writing,
    onPdf,
    onBack,
  }: {
    day: string;
    today: string;
    overview: DayOverview;
    // The batch whose summary this is; `null`: the date's.
    batch: number | null;
    // A PDF the owner waits for is being written.
    writing: boolean;
    // `null`: every account.
    onPdf: (account: string | null) => void;
    onBack: () => void;
  } = $props();

  const words = texts.summary;
  const accounts = $derived(
    batch === null
      ? accountsOf(overview.batches)
      : (overview.batches.find((view) => view.batch === batch)?.accounts ?? []).map(
          ({ account }) => account,
        ),
  );
  let account = $state<string | null>(null);
  let view = $state<SummaryView | null>(null);
  let error = $state("");

  $effect(() => {
    const [date, of, chosen] = [day, batch, account];
    view = null;
    error = "";
    summaryView(date, of, chosen).then(
      (read) => (view = read),
      (e) => (error = String(e)),
    );
  });

  // An archived batch has no summary PDF of every account left to open.
  const noPdf = $derived(
    access.dateLocked || writing || (access.archived && batch !== null && account === null),
  );
  const counted = (count: Count) => `${count.n} ${count.n === 1 ? count.one : count.many}`;
</script>

<main>
  <DayTitle {day} {today} />
  <div class="bar">
    <h2>{batch === null ? words.day : words.batch(batch)}</h2>
    {#if accounts.length > 1}
      <select aria-label={words.account} bind:value={account}>
        <option value={null}>{words.allAccounts}</option>
        {#each accounts as name (name)}
          <option value={name}>{name}</option>
        {/each}
      </select>
    {/if}
    <span class="grow"></span>
    <button title={words.pdfTitle} disabled={noPdf} onclick={() => onPdf(account)}>{words.pdf}</button>
    <button onclick={onBack}>{words.back}</button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {:else if !view}
    <p class="muted">{words.loading}</p>
  {:else}
    <p class="counts">{view.counts}</p>
    {#each view.parts as part, index (index)}
      <section class="part">
        <div class="head">
          <span class="heading">{part.heading}</span>
          {#if part.kind !== "picks" && part.right}<span class="right">{part.right}</span>{/if}
        </div>
        {#if part.kind === "facts"}
          {#each part.rows as row, i (i)}
            <div class="row facts" class:bold={row.bold}>
              <span class="key">{row.key ?? ""}</span>
              <span class="text">{row.text}</span>
              {#each row.counts as count, c (c)}
                <span class="count">{count ? counted(count) : ""}</span>
              {/each}
              <span class="tail">{row.tail ?? ""}</span>
            </div>
          {/each}
        {:else if part.kind === "contents"}
          {#each part.rows as row (row.number)}
            <div class="row contents">
              <span class="key">{row.number}</span>
              <span class="items">
                {#each row.items as item, i (i)}<span>{item}</span>{/each}
              </span>
              <span class="orders">{row.orders}</span>
            </div>
          {/each}
        {:else}
          <div class="picks">
            {#each part.rows as pick, i (i)}
              <span class="units">{pick.units}</span>
              <span>{pick.item}</span>
            {/each}
          </div>
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
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin: 0.6rem 0 0.4rem;
  }

  .bar h2 {
    margin: 0 0.4rem 0 0;
    font-size: 1.1rem;
  }

  .grow {
    flex: 1;
  }

  .muted,
  .counts {
    color: var(--text-muted);
  }

  .counts {
    margin: 0 0 0.9rem;
  }

  .error {
    color: var(--danger);
  }

  /* Each part as a card under its shaded head, as the PDF's grey bars. */
  .part {
    margin-bottom: 0.9rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
  }

  .head {
    display: flex;
    gap: 0.8rem;
    padding: 0.45rem 0.75rem;
    border-bottom: 1px solid var(--border-soft);
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    background: var(--surface-2);
    font-weight: 600;
  }

  .heading {
    flex: 1;
    white-space: pre-wrap;
  }

  .right {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  /* Ruled rows: the key, the text taking the room left, then the counts and the hash. */
  .row {
    display: grid;
    gap: 0.9rem;
    align-items: baseline;
    padding: 0.3rem 0.75rem;
    border-bottom: 1px solid var(--border-soft);
  }

  .row:last-child {
    border-bottom: none;
  }

  .facts {
    grid-template-columns: 3.5rem minmax(0, 1fr) 7rem 7rem auto;
  }

  .contents {
    grid-template-columns: 3.5rem minmax(0, 1fr) auto;
  }

  .bold,
  .key,
  .orders {
    font-weight: 600;
  }

  .text,
  .items span {
    overflow-wrap: anywhere;
  }

  .items {
    display: flex;
    flex-direction: column;
  }

  .count,
  .orders,
  .units {
    font-variant-numeric: tabular-nums;
  }

  .tail {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 0.85rem;
  }

  /* The picks in two columns: the units, then the item. */
  .picks {
    display: grid;
    grid-template-columns: 3.5rem minmax(0, 1fr) 3.5rem minmax(0, 1fr);
    gap: 0.25rem 0.9rem;
    padding: 0.5rem 0.75rem;
  }

  .units {
    font-weight: 600;
    text-align: right;
  }
</style>
