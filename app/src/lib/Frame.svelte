<script lang="ts">
  // The window frame: the panel's switch, the program's name, the open workspace and date, and the
  // screens that open over the current one.
  import { texts } from "../texts";
  import { formatDay } from "../dates";
  import type { OverlayId } from "./shell.svelte";
  import Icon, { type IconName } from "./Icon.svelte";
  import Logo from "./Logo.svelte";

  let {
    workspace,
    day,
    overlay,
    // The program's shown version (`v0.2`, from `StartInfo.version`): with the name it is the
    // same text as the OS window title.
    version,
    onTogglePanel,
    onNavigate,
  }: {
    workspace: string | null;
    day: string;
    overlay: OverlayId | null;
    version: string;
    onTogglePanel: () => void;
    onNavigate: (id: OverlayId) => void;
  } = $props();

  const items: { id: OverlayId; label: string; icon: IconName }[] = [
    { id: "search", label: texts.nav.search, icon: "search" },
    { id: "sort-rules", label: texts.nav.sortRules, icon: "rules" },
    { id: "settings", label: texts.nav.settings, icon: "settings" },
  ];
</script>

<header>
  <button
    class="icon"
    title={texts.nav.togglePanel}
    aria-label={texts.nav.togglePanel}
    onclick={onTogglePanel}><Icon name="panel" /></button
  >
  <span class="brand"><Logo size={22} />{texts.title}<span class="version">{version}</span></span>
  <span class="where">
    <span class="workspace">{workspace ?? ""}</span>
    <span class="sep">›</span>
    <span class="day">{formatDay(day)}</span>
  </span>

  <span class="grow"></span>

  <nav>
    {#each items as item (item.id)}
      <button
        class="ghost"
        class:active={overlay === item.id}
        onclick={() => onNavigate(item.id)}><Icon name={item.icon} size={16} />{item.label}</button
      >
    {/each}
  </nav>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    height: 3rem;
    padding: 0 0.75rem;
    border-bottom: 1px solid var(--border);
    background: var(--header-bg);
  }

  .brand {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-weight: 650;
    color: var(--text);
  }

  .version {
    margin-left: 0.15rem;
    padding: 0 0.45rem;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent-text);
    font-size: 0.75rem;
    font-weight: 600;
  }

  .where {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    margin-left: 0.75rem;
    padding-left: 0.9rem;
    border-left: 1px solid var(--border);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .workspace {
    font-weight: 600;
  }

  .sep {
    color: var(--text-faint);
  }

  .day {
    color: var(--text-muted);
  }

  .grow {
    flex: 1;
  }

  nav {
    display: flex;
    gap: 0.2rem;
  }

  nav button.active {
    background: var(--accent-soft);
    color: var(--accent-text);
    font-weight: 600;
  }
</style>
