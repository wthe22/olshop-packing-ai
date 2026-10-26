<script lang="ts">
  // The side panel: the open namespace, its dates by month with their saved orders, and the way to
  // another date or namespace. What a click does (asking about an open draft first) is the shell's.
  import type { PanelView } from "./api/panel";
  import { months } from "./panel";
  import { formatMonth, formatShortDay, texts } from "../texts";

  let {
    panel,
    day,
    today,
    onOpenDate,
    onOpenNamespace,
    onNewNamespace,
    onOpenFolder,
    onHide,
  }: {
    panel: PanelView;
    day: string;
    today: string;
    onOpenDate: (date: string) => void;
    onOpenNamespace: (name: string) => void;
    // Resolves to why the name is refused, or `null` when the namespace was made.
    onNewNamespace: (name: string) => Promise<string | null>;
    onOpenFolder: (path: string) => void;
    onHide: () => void;
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
    <label>
      {texts.panel.namespace}
      <select
        value={panel.open ?? ""}
        onchange={(event) => onOpenNamespace(event.currentTarget.value)}
      >
        {#each panel.namespaces as known (known)}
          <option value={known}>{known}</option>
        {/each}
      </select>
    </label>
    <div class="tools">
      <button onclick={() => ((naming = true), (refused = ""))}>{texts.panel.newNamespace}</button>
      <button title={texts.panel.hide} aria-label={texts.panel.hide} onclick={onHide}>«</button>
    </div>
    {#if naming}
      <div class="naming">
        <input
          bind:value={name}
          onkeydown={(event) => event.key === "Enter" && create()}
          placeholder={texts.namespace.placeholder}
        />
        <button disabled={name.trim() === ""} onclick={create}>{texts.panel.create}</button>
        <button onclick={() => (naming = false)}>{texts.panel.cancel}</button>
      </div>
      {#if refused}<p class="refused">{refused}</p>{/if}
    {/if}
  </div>

  <ul class="months">
    {#each list as month (month.key)}
      {@const open = isOpen(month.key, month.startsOpen)}
      <li>
        <button class="month" onclick={() => (toggled[month.key] = !open)}>
          {open ? "▾" : "▸"}
          {formatMonth(month.key)}
        </button>
        {#if open}
          <ul>
            {#each month.rows as row (row.date)}
              <li class="date" class:open={row.date === day}>
                <button class="pick" onclick={() => onOpenDate(row.date)}>
                  <span class="label">
                    {#if row.damaged}⚠{/if}
                    {formatShortDay(row.date)}{#if row.today}{texts.panel.today}{/if}
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
                      }}>{texts.panel.openFolder}</button
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
    border-right: 1px solid #d5d5d5;
    background: #f7f7f7;
    overflow-y: auto;
    padding: 0.6rem 0.5rem;
  }

  .head label {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .head select {
    flex: 1;
    min-width: 0;
  }

  .tools,
  .naming {
    display: flex;
    gap: 0.35rem;
    margin-top: 0.4rem;
  }

  .tools button:first-child,
  .naming input {
    flex: 1;
    min-width: 0;
  }

  .refused {
    margin: 0.3rem 0 0;
    color: #b3261e;
    font-size: 0.85rem;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .months {
    margin-top: 0.75rem;
  }

  button.month {
    width: 100%;
    text-align: left;
    border: none;
    background: none;
    font-weight: 600;
    padding: 0.3rem 0.2rem;
  }

  .date {
    position: relative;
    display: flex;
    align-items: center;
    border-left: 3px solid transparent;
  }

  .date.open {
    border-left-color: #2f6fb5;
    background: #e3ecf7;
  }

  button.pick {
    flex: 1;
    display: flex;
    justify-content: space-between;
    border: none;
    background: none;
    padding: 0.25rem 0.4rem;
    text-align: left;
  }

  .count {
    color: #6a6a6a;
    font-variant-numeric: tabular-nums;
  }

  button.menu {
    visibility: hidden;
    border: none;
    background: none;
    padding: 0 0.35rem;
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
  }

  @media (prefers-color-scheme: dark) {
    aside {
      background: #262626;
      border-right-color: #444444;
    }

    .date.open {
      background: #34455a;
      border-left-color: #6f8fb5;
    }

    .count {
      color: #b0b0b0;
    }
  }
</style>
