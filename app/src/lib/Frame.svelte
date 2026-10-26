<script lang="ts">
  // The window frame: the panel's switch, the program's name, the open workspace and date, the
  // screens that open over the current one, and the theme menu.
  import { texts, formatDay } from "../texts";
  import type { OverlayId } from "./shell.svelte";
  import type { Theme } from "./theme";
  import Icon, { type IconName } from "./Icon.svelte";
  import ThemeMenu from "./ThemeMenu.svelte";

  let {
    workspace,
    day,
    overlay,
    // The program's shown version (`v0.2`, from `StartInfo.version`): with the name it is the
    // same text as the OS window title.
    version,
    theme,
    onTogglePanel,
    onNavigate,
    onTheme,
  }: {
    workspace: string | null;
    day: string;
    overlay: OverlayId | null;
    version: string;
    theme: Theme;
    onTogglePanel: () => void;
    onNavigate: (id: OverlayId) => void;
    onTheme: (theme: Theme) => void;
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
  <span class="brand"><Icon name="box" size={20} />{texts.title}<span class="version">{version}</span></span>
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
  <span class="divider"></span>
  <ThemeMenu {theme} onChoose={onTheme} />
</header>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    height: 3rem;
    padding: 0 0.75rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .brand {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-weight: 650;
    color: var(--text);
  }

  .brand :global(svg) {
    color: var(--accent);
  }

  .version {
    margin-left: 0.15rem;
    padding: 0 0.4rem;
    border-radius: 999px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    color: var(--text-muted);
    font-size: 0.75rem;
    font-weight: 500;
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
    color: var(--link);
    font-weight: 600;
  }

  .divider {
    width: 1px;
    height: 1.4rem;
    background: var(--border);
  }
</style>
