<script lang="ts">
  // The side panel: the dates of the data folder by month with their saved orders of every account,
  // and the way to another date. What a click does (asking about an open draft first) is the shell's.
  import type { PanelView } from "./api/panel";
  import { months } from "./panel";
  import { texts } from "../texts";
  import { formatMonth, formatShortDay } from "../dates";
  import Icon from "./Icon.svelte";

  let {
    panel,
    day,
    today,
    onOpenDate,
    onOpenFolder,
  }: {
    panel: PanelView;
    day: string;
    today: string;
    onOpenDate: (date: string) => void;
    onOpenFolder: (path: string) => void;
  } = $props();

  const list = $derived(months(panel.dates, day, today));
  // A month the owner folded or unfolded; the others follow `startsOpen`.
  let toggled = $state<Record<string, boolean>>({});
  // The date whose row menu is open.
  let menuFor = $state<string | null>(null);

  const isOpen = (key: string, startsOpen: boolean) => toggled[key] ?? startsOpen;
</script>

<aside>
  <h2>{texts.panel.dates}</h2>
  <ul class="months">
    {#each list as month (month.key)}
      {@const open = isOpen(month.key, month.startsOpen)}
      <li>
        <button class="month" onclick={() => (toggled[month.key] = !open)}>
          <span class="chevron" class:open>▸</span>
          {formatMonth(month.key)}
        </button>
        {#if open}
          <ul>
            {#each month.rows as row (row.date)}
              <li class="date" class:open={row.date === day}>
                <button class="pick-date" onclick={() => onOpenDate(row.date)}>
                  <span class="label">
                    {#if row.damaged}⚠{/if}
                    {formatShortDay(row.date)}
                    {#if row.today}<span class="today">{texts.panel.today}</span>{/if}
                  </span>
                  <span class="count">
                    {#if row.damaged}{texts.panel.damaged}{:else}{row.orders ?? ""}{/if}
                  </span>
                </button>
                {#if row.folder}
                  <button
                    class="menu"
                    title={texts.panel.menu}
                    aria-label={texts.panel.menu}
                    onclick={() => (menuFor = menuFor === row.date ? null : row.date)}>⋯</button
                  >
                  {#if menuFor === row.date}
                    <button
                      class="item"
                      onclick={() => {
                        menuFor = null;
                        if (row.folder) onOpenFolder(row.folder);
                      }}><Icon name="folder" size={16} />{texts.panel.openFolder}</button
                    >
                  {/if}
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </li>
    {/each}
  </ul>
</aside>

<style>
  aside {
    width: 16rem;
    flex: none;
    border-right: 1px solid var(--border);
    background: var(--panel-bg);
    overflow-y: auto;
    padding: 0.9rem 0.6rem;
  }

  h2 {
    margin: 0 0.2rem 0.4rem;
    color: var(--text-faint);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  button.month {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    width: 100%;
    margin-top: 0.3rem;
    padding: 0.3rem 0.3rem;
    border: none;
    background: none;
    color: var(--text-muted);
    font-size: 0.85rem;
    font-weight: 600;
    text-align: left;
  }

  .chevron {
    display: inline-block;
    font-size: 0.7rem;
    transition: transform 0.1s;
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  .date {
    position: relative;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    margin: 1px 0;
    border-radius: var(--radius);
  }

  .date:hover {
    background: var(--surface-hover);
  }

  .date.open {
    background: var(--accent-soft);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .date.open .label {
    font-weight: 600;
  }

  button.pick-date {
    flex: 1;
    display: flex;
    justify-content: space-between;
    border: none;
    background: none;
    padding: 0.3rem 0.5rem 0.3rem 0.65rem;
    text-align: left;
  }

  button.pick-date:hover {
    background: none;
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

  .count {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  /* Under today's row. */
  button.menu {
    visibility: hidden;
    border: none;
    background: none;
    padding: 0 0.4rem;
    color: var(--text-muted);
  }

  .date:hover button.menu,
  .date:focus-within button.menu {
    visibility: visible;
  }

  button.item {
    position: absolute;
    right: 0;
    top: 100%;
    z-index: 5;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    box-shadow: var(--shadow-lg);
  }
</style>
