<script lang="ts">
  // The open date as a screen's title, "Wed, 7 Oct 2026", with a *today* pill. In the batch flow
  // (`inBatch`) it is a line of its own over the screen's bar, and a date that is not today is
  // marked *not today*: the batch is saved into the open date.
  import { texts } from "../texts";
  import { formatDay } from "../dates";

  let { day, today, inBatch = false }: { day: string; today: string; inBatch?: boolean } = $props();
</script>

<div class="day-title" class:in-batch={inBatch}>
  <h1>{formatDay(day)}</h1>
  {#if day === today}
    <span class="pill today">{texts.day.today}</span>
  {:else if inBatch}
    <span class="pill not-today">{texts.day.notToday}</span>
  {/if}
</div>

<style>
  .day-title {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    margin-right: 0.35rem;
  }

  .in-batch {
    margin: 0 0 0.6rem;
  }

  h1 {
    margin: 0;
    font-size: 1.3rem;
    font-weight: 700;
  }

  .pill {
    padding: 0.05rem 0.55rem;
    border-radius: 999px;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .today {
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  .not-today {
    border: 1px solid var(--warn-border);
    background: var(--warn-bg);
    color: var(--warn);
  }
</style>
