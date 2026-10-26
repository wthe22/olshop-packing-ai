<script lang="ts">
  // The side panel: the open namespace, its dates by month with their saved orders, and the way to
  // another date or namespace. What a click does (asking about an open draft first) is the shell's.
  import type { PanelView } from "./api/panel";
  import { months } from "./panel";
  import { formatMonth, formatShortDay, texts } from "../texts";
  import Icon from "./Icon.svelte";

  let {
    panel,
    day,
    today,
    onOpenDate,
    onOpenNamespace,
    onNewNamespace,
    onOpenFolder,
  }: {
    panel: PanelView;
    day: string;
    today: string;
    onOpenDate: (date: string) => void;
    onOpenNamespace: (name: string) => void;
    // Resolves to why the name is refused, or `null` when the namespace was made.
    onNewNamespace: (name: string) => Promise<string | null>;
    onOpenFolder: (path: string) => void;
  } = $props();

  const list = $derived(months(panel.dates, day, today));
  // A month the owner folded or unfolded; the others follow `startsOpen`.
  let toggled = $state<Record<string, boolean>>({});
  // The date whose row menu is open.
  let menuFor = $state<string | null>(null);
  // **+ New namespace**'s field, and why a name was refused.
  let naming = $state(false);
  let name = $state("");
  let refused = $state("");

  const isOpen = (key: string, startsOpen: boolean) => toggled[key] ?? startsOpen;

  async function create() {
    refused = (await onNewNamespace(name.trim())) ?? "";
    if (!refused) {
      naming = false;
      name = "";
    }
  }
</script>

<aside>
  <div class="head">
    <label for="panel-namespace">{texts.panel.namespace}</label>
    <div class="pick">
      <select
        id="panel-namespace"
        value={panel.open ?? ""}
        onchange={(event) => onOpenNamespace(event.currentTarget.value)}
      >
        {#each panel.namespaces as known (known)}
          <option value={known}>{known}</option>
        {/each}
      </select>
      <button
        class="add"
        title={texts.panel.newNamespace}
        aria-label={texts.panel.newNamespace}
        onclick={() => ((naming = !naming), (refused = ""))}><Icon name="plus" size={16} /></button
      >
    </div>
    {#if naming}
      <form
        class="naming"
        onsubmit={(event) => {
          event.preventDefault();
          if (name.trim()) create();
        }}
      >
        <input bind:value={name} placeholder={texts.namespace.placeholder} maxlength="40" />
        <div class="naming-buttons">
          <button type="button" onclick={() => (naming = false)}>{texts.panel.cancel}</button>
          <button class="primary" type="submit" disabled={name.trim() === ""}
            >{texts.panel.create}</button
          >
        </div>
      </form>
      {#if refused}<p class="refused">{refused}</p>{/if}
    {/if}
  </div>

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
    background: var(--surface-2);
    overflow-y: auto;
    padding: 0.9rem 0.6rem;
  }

  .head {
    padding: 0 0.2rem 0.9rem;
    border-bottom: 1px solid var(--border);
  }

  .head label {
    display: block;
    margin-bottom: 0.3rem;
    color: var(--text-faint);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .pick {
    display: flex;
    gap: 0.35rem;
  }

  .pick select {
    flex: 1;
    min-width: 0;
    font-weight: 600;
  }

  button.add {
    display: inline-flex;
    align-items: center;
    padding: 0 0.45rem;
  }

  .naming {
    margin-top: 0.5rem;
  }

  .naming input {
    width: 100%;
  }

  .naming-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 0.35rem;
    margin-top: 0.4rem;
  }

  .refused {
    margin: 0.3rem 0 0;
    color: var(--danger);
    font-size: 0.85rem;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .months {
    margin-top: 0.6rem;
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
    margin-left: 0.2rem;
    color: var(--link);
    font-size: 0.85rem;
  }

  .count {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

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
