<script lang="ts">
  // The side panel: the dates of the data folder by month with their saved orders of every account
  // and their size, an archived date muted, and the way to another date. What a click does (asking
  // about an open draft first) and **Archive** are the shell's.
  import type { PanelView } from "./api/panel";
  import { canArchive, months } from "./panel";
  import { texts } from "../texts";
  import { formatMonth, formatShortDay } from "../dates";
  import { formatSize } from "../size";
  import { dismiss } from "./dismiss";
  import { access } from "./readOnly.svelte";
  import Icon from "./Icon.svelte";

  let {
    panel,
    day,
    today,
    onOpenDate,
    onOpenFolder,
    onArchive,
  }: {
    panel: PanelView;
    day: string;
    today: string;
    onOpenDate: (date: string) => void;
    onOpenFolder: (path: string) => void;
    // **Archive** in a date's row menu; the shell asks first.
    onArchive: (date: string) => void;
  } = $props();

  const list = $derived(months(panel.dates, day, today));
  // A month the owner folded or unfolded; the others follow `startsOpen`.
  let toggled = $state<Record<string, boolean>>({});
  // The date whose row menu is open.
  let menuFor = $state<string | null>(null);

  const isOpen = (key: string, startsOpen: boolean) => toggled[key] ?? startsOpen;
  const closeMenu = () => (menuFor = null);
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
              <li class="date" class:open={row.date === day} class:archived={row.archived}>
                <button
                  class="pick-date"
                  title={row.archived ? texts.archive.archivedRow : undefined}
                  onclick={() => onOpenDate(row.date)}
                >
                  <span class="label">
                    {#if row.damaged}⚠{/if}
                    {#if row.archived}<Icon name="archive" size={14} />{/if}
                    {formatShortDay(row.date)}
                    {#if row.today}<span class="today">{texts.panel.today}</span>{/if}
                  </span>
                  <span class="count">
                    {#if row.damaged}{texts.panel.damaged}{:else}{row.orders ?? ""}{/if}
                  </span>
                  <span class="size">{row.size_bytes > 0 ? formatSize(row.size_bytes) : ""}</span>
                </button>
                {#if row.folder}
                  <span class="row-menu" {@attach menuFor === row.date && dismiss(closeMenu)}>
                    <button
                      class="menu"
                      title={texts.panel.menu}
                      aria-label={texts.panel.menu}
                      aria-haspopup="menu"
                      aria-expanded={menuFor === row.date}
                      onclick={() => (menuFor = menuFor === row.date ? null : row.date)}>⋯</button
                    >
                    {#if menuFor === row.date}
                      <span class="items" role="menu">
                        <button
                          role="menuitem"
                          onclick={() => {
                            closeMenu();
                            if (row.folder) onOpenFolder(row.folder);
                          }}><Icon name="folder" size={16} />{texts.panel.openFolder}</button
                        >
                        {#if canArchive(row) && !access.readOnly}
                          <button
                            role="menuitem"
                            onclick={() => {
                              closeMenu();
                              onArchive(row.date);
                            }}><Icon name="archive" size={16} />{texts.archive.archive}</button
                          >
                        {/if}
                      </span>
                    {/if}
                  </span>
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
    width: 18rem;
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
    gap: 0.5rem;
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

  .label {
    flex: 1;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .count {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  /* The size after the orders, in a column of its own so the sizes stand under each other. */
  .size {
    width: 4.2rem;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .date.archived .label,
  .date.archived .count {
    color: var(--text-faint);
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

  .items {
    position: absolute;
    right: 0;
    top: 100%;
    z-index: 5;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0.3rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    box-shadow: var(--shadow-lg);
  }

  .items button {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    border: none;
    background: none;
    text-align: left;
    white-space: nowrap;
  }

  .items button:hover {
    background: var(--surface-hover);
  }
</style>
