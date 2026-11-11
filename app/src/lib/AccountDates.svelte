<script lang="ts">
  // The Dates section of an account's card: the dates with its orders, newest first, months folded
  // as in the panel, with the orders and the batches that hold them as count columns. A click opens
  // the date; what that asks and closes is the shell's.
  import type { AccountDates } from "./api/accounts";
  import { months } from "./panel";
  import { texts } from "../texts";
  import { formatMonth, formatShortDay } from "../dates";
  import Count from "./Count.svelte";

  let {
    dates,
    today,
    onOpenDate,
  }: {
    dates: AccountDates | null;
    today: string;
    onOpenDate: (date: string) => void;
  } = $props();

  const words = texts.accounts;

  const list = $derived(months(dates?.dates ?? [], "", today));
  // A month the owner folded or unfolded; the others follow `startsOpen`.
  let toggled = $state<Record<string, boolean>>({});

  const isOpen = (key: string, startsOpen: boolean) => toggled[key] ?? startsOpen;
  const total = (rows: { orders: number }[]) => rows.reduce((sum, row) => sum + row.orders, 0);
</script>

<div class="dates">
  <div class="columns">
    <h4>{words.dates}</h4>
    <span class="grow"></span>
    <span class="head">{words.orders}</span>
    <span class="head">{words.batches}</span>
    <span class="chevron-room"></span>
  </div>
  {#if !dates || dates.loading}
    <p class="note">{words.loading}</p>
  {:else if list.length === 0}
    <p class="note">{words.noDates}</p>
  {/if}
  {#each list as month (month.key)}
    {@const open = isOpen(month.key, month.startsOpen)}
    <button class="month" onclick={() => (toggled[month.key] = !open)}>
      <span class="chevron" class:open>▸</span>
      <span class="grow">{formatMonth(month.key)}</span>
      {#if !open}
        <Count n={total(month.rows)} word={words.orderWord(total(month.rows))} />
        <span class="batches"></span>
        <span class="chevron-room"></span>
      {/if}
    </button>
    {#if open}
      {#each month.rows as row (row.date)}
        <button class="date" onclick={() => onOpenDate(row.date)}>
          <span class="grow"
            >{formatShortDay(row.date)}{#if row.today}<span class="today">{words.today}</span
              >{/if}</span
          >
          <Count n={row.orders} word={words.orderWord(row.orders)} />
          <Count n={row.batches} word={words.batchWord(row.batches)} />
          <span class="chevron-room">›</span>
        </button>
      {/each}
    {/if}
  {/each}
</div>

<style>
  .dates {
    border-top: 1px solid var(--border-soft);
  }

  .columns {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.45rem 1rem;
  }

  h4,
  .head {
    margin: 0;
    color: var(--text-muted);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .head {
    flex: none;
    width: 13ch;
    text-align: right;
  }

  .grow {
    flex: 1;
    text-align: left;
  }

  .chevron-room {
    flex: none;
    width: 1rem;
    color: var(--text-faint);
    text-align: center;
  }

  .batches {
    flex: none;
    width: 13ch;
  }

  .note {
    margin: 0;
    padding: 0.4rem 1rem 0.8rem;
    color: var(--text-muted);
  }

  button.month,
  button.date {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.35rem 1rem;
    border: none;
    border-top: 1px solid var(--border-soft);
    border-radius: 0;
    background: none;
    text-align: left;
  }

  button.month {
    color: var(--text-muted);
    font-weight: 600;
  }

  button.date:hover,
  button.month:hover {
    background: var(--surface-hover);
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
</style>
