<script lang="ts">
  // The window frame: the title, the panel's switch, the open namespace and date, and the screens
  // that open over the current one.
  import { texts, formatDay, brandText } from "../texts";
  import type { OverlayId } from "./shell.svelte";

  let {
    namespace,
    day,
    overlay,
    // The program's shown version: the title bar's brand text is the
    // program's name and this, the same line as the OS window title (from `StartInfo.version`).
    version,
    onTogglePanel,
    onNavigate,
  }: {
    namespace: string | null;
    day: string;
    overlay: OverlayId | null;
    version: string;
    onTogglePanel: () => void;
    onNavigate: (id: OverlayId) => void;
  } = $props();

  const items: { id: OverlayId; label: string }[] = [
    { id: "search", label: texts.nav.search },
    { id: "sort-rules", label: texts.nav.sortRules },
    { id: "settings", label: texts.nav.settings },
  ];
</script>

<header>
  <span class="brand">{brandText(version)}</span>
  <span class="dot">·</span>
  <button
    class="panel"
    title={texts.nav.showPanel}
    aria-label={texts.nav.showPanel}
    onclick={onTogglePanel}>☰</button
  >
  <span class="where">{namespace ?? ""} · {formatDay(day)}</span>

  <span class="grow"></span>

  <nav>
    {#each items as item (item.id)}
      <button
        class="nav"
        class:active={overlay === item.id}
        onclick={() => onNavigate(item.id)}>{item.label}</button
      >
    {/each}
  </nav>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid #d5d5d5;
    background: #ffffff;
  }

  .brand {
    font-weight: 600;
  }

  .dot {
    color: #9a9a9a;
  }

  .panel {
    padding: 0.15rem 0.5rem;
  }

  .where {
    font-variant-numeric: tabular-nums;
  }

  .grow {
    flex: 1;
  }

  nav {
    display: flex;
    gap: 0.35rem;
  }

  .nav.active {
    background: #e7f0fc;
    border-color: #7aa7d6;
    font-weight: 600;
  }

  @media (prefers-color-scheme: dark) {
    header {
      background: #2b2b2b;
      border-bottom-color: #444444;
    }

    .nav.active {
      background: #34455a;
      border-color: #6f8fb5;
    }
  }
</style>
