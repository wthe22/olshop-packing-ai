<script lang="ts">
  // The Archive page (ui/archive.md): *Automatic* with its number of days, and every date by month
  // with its orders and size, a tick on each date that can be archived, and a foot with what the
  // ticked dates' PDFs take and **Archive**. Read-only, the dates and sizes only. The state is
  // `archivePage.svelte.ts`.
  import { onMount } from "svelte";
  import type { Archiving } from "./archiving.svelte";
  import { ArchivePage } from "./archivePage.svelte";
  import { Folds } from "./folds.svelte";
  import { months } from "./panel";
  import { access } from "./readOnly.svelte";
  import { texts } from "../texts";
  import { formatMonth, formatShortDay } from "../dates";
  import { formatSize } from "../size";
  import Icon from "./Icon.svelte";

  let { archiving, today }: { archiving: Archiving; today: string } = $props();

  const words = texts.archive;
  // Made as the page opens.
  let page = $state<ArchivePage | null>(null);
  onMount(() => {
    const opened = new ArchivePage(archiving);
    page = opened;
    opened.load();
    return opened.close;
  });

  const folds = new Folds();
  const list = $derived(months(page?.view?.dates ?? [], "", today));
</script>

{#if page}
  {@const shown = page}
  <div class="archive">
    <section>
      <h3>{words.automatic}</h3>
      <div class="group">
        <div class="row">
          <label class="field"
            >{words.olderThan}
            <input
              class="days"
              type="number"
              min="1"
              value={shown.days}
              disabled={access.readOnly}
              onblur={(event) => shown.changeDays(event.currentTarget.valueAsNumber)}
            />
            {words.days}</label
          >
          <span class="grow"></span>
          <div class="segmented" role="radiogroup" aria-label={words.automatic}>
            {#each [false, true] as on (on)}
              <button
                role="radio"
                aria-checked={shown.auto === on}
                class:chosen={shown.auto === on}
                disabled={access.readOnly}
                onclick={() => shown.turn(on)}>{on ? words.on : words.off}</button
              >
            {/each}
          </div>
        </div>
      </div>
      {#if shown.error}
        <p class="problem">{shown.error}</p>
      {/if}
    </section>

    <section>
      <div class="head">
        <h3>{words.dates}</h3>
        {#if shown.view}
          <span class="total">{words.total(formatSize(shown.view.total_bytes))}</span>
        {/if}
      </div>
      <div class="group">
        <table>
          <thead>
            <tr>
              <th class="tick"></th>
              <th>{words.dateHead}</th>
              <th class="number">{words.ordersHead}</th>
              <th class="number">{words.sizeHead}</th>
              <th></th>
            </tr>
          </thead>
          {#each list as month (month.key)}
            {@const open = folds.isOpen(month)}
            <tbody>
              <tr class="month">
                <td class="tick"></td>
                <td colspan="4">
                  <span class="month-head">
                    <button class="month" onclick={() => folds.toggle(month)}>
                      <span class="chevron" class:open>▸</span>
                      {formatMonth(month.key)}
                    </button>
                    {#if !access.readOnly && shown.tickable(month.rows).length > 0}
                      <input
                        type="checkbox"
                        aria-label={words.tickMonth(formatMonth(month.key))}
                        title={words.tickMonth(formatMonth(month.key))}
                        checked={shown.monthTicked(month.rows)}
                        onchange={(event) => {
                          shown.tickMonth(month.rows);
                          event.currentTarget.checked = shown.monthTicked(month.rows);
                        }}
                      />
                    {/if}
                  </span>
                </td>
              </tr>
              {#if open}
                {#each month.rows as row (row.date)}
                  <tr class:archived={row.archived}>
                    <td class="tick">
                      {#if row.archive && !access.readOnly}
                        <input
                          type="checkbox"
                          aria-label={words.tickDate(row.date)}
                          checked={shown.isTicked(row.date)}
                          onclick={(event) => {
                            shown.tick(row.date, event.shiftKey);
                            event.currentTarget.checked = shown.isTicked(row.date);
                          }}
                        />
                      {/if}
                    </td>
                    <td class="date"
                      >{formatShortDay(row.date)}{#if row.today}<span class="today">{texts.panel.today}</span
                        >{/if}</td
                    >
                    <td class="number">{row.orders ?? ""}</td>
                    <td class="number">{formatSize(row.size_bytes)}</td>
                    <td class="state">{row.archived ? words.archived : ""}</td>
                  </tr>
                {/each}
              {/if}
            </tbody>
          {/each}
        </table>
        {#if !access.readOnly}
          <div class="foot">
            <span class="grow"
              >{shown.ticked.length > 0 ? words.foot(shown.ticked.length, formatSize(shown.bytes())) : ""}</span
            >
            <button class="primary" disabled={shown.ticked.length === 0} onclick={shown.archive}
              ><Icon name="archive" size={16} />{words.archive}</button
            >
          </div>
        {/if}
      </div>
    </section>
  </div>
{/if}

<style>
  .archive {
    display: flex;
    flex-direction: column;
    gap: 1.6rem;
  }

  h3 {
    margin: 0 0 0.5rem;
    color: var(--text-muted);
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .total {
    color: var(--text-muted);
    font-size: 0.9rem;
    font-variant-numeric: tabular-nums;
  }

  /* A group of rows: one box, as on Settings. */
  .group {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 1rem;
    min-height: 3.4rem;
    padding: 0.6rem 1rem;
  }

  .grow {
    flex: 1;
  }

  .field {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    white-space: nowrap;
  }

  input.days {
    width: 4.5rem;
    font-variant-numeric: tabular-nums;
  }

  .problem {
    margin: 0.5rem 0 0;
    color: var(--danger);
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  /* The column heads of the Plan screen. */
  th {
    padding: 0.4rem 0.4rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-muted);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-align: left;
    text-transform: uppercase;
  }

  th:first-child {
    border-top-left-radius: var(--radius-lg);
  }

  th:last-child {
    border-top-right-radius: var(--radius-lg);
  }

  td {
    padding: 0.3rem 0.4rem;
    border-top: 1px solid var(--border-soft);
    white-space: nowrap;
  }

  th.number,
  td.number {
    width: 7rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .tick {
    width: 2rem;
    padding-left: 1rem;
  }

  .month-head {
    display: inline-flex;
    align-items: center;
    gap: 0.6rem;
  }

  button.month {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.1rem 0.2rem;
    border: none;
    background: none;
    color: var(--text-muted);
    font-weight: 600;
  }

  .chevron {
    display: inline-block;
    font-size: 0.7rem;
    transition: transform 0.1s;
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  .today {
    margin-left: 0.35rem;
    padding: 0 0.4rem;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent-text);
    font-size: 0.75rem;
    font-weight: 600;
  }

  td.state,
  tr.archived td.number {
    color: var(--text-faint);
  }

  td.state {
    padding-left: 1.5rem;
    width: 100%;
  }

  /* Stays in view at the page's foot while the dates scroll. */
  .foot {
    position: sticky;
    bottom: 0;
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.6rem 1rem;
    border-top: 1px solid var(--border);
    border-radius: 0 0 var(--radius-lg) var(--radius-lg);
    background: var(--surface);
    font-variant-numeric: tabular-nums;
  }

  .foot button {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }
</style>
