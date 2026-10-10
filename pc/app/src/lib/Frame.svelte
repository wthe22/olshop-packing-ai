<script lang="ts">
  // The window frame (08 › *Window frame*): title, the day switch and the navigation.
  import { texts, formatDay } from "../texts";

  // The batch flow screens (*new-batch*, *plan*) are not in the nav; none is active there.
  type Screen = "day" | "new-batch" | "plan" | "categories" | "settings";

  let {
    day,
    screen,
    onStep,
    onToday,
    onNavigate,
  }: {
    day: string;
    screen: Screen;
    onStep: (delta: number) => void;
    onToday: () => void;
    onNavigate: (screen: Screen) => void;
  } = $props();

  const screens: { id: Screen; label: string }[] = [
    { id: "day", label: texts.nav.day },
    { id: "categories", label: texts.nav.categories },
    { id: "settings", label: texts.nav.settings },
  ];
</script>

<header>
  <span class="brand">{texts.title}</span>
  <span class="dot">·</span>
  <button
    class="step"
    title={texts.nav.previousDay}
    aria-label={texts.nav.previousDay}
    onclick={() => onStep(-1)}>◀</button
  >
  <span class="day">{formatDay(day)}</span>
  <button
    class="step"
    title={texts.nav.nextDay}
    aria-label={texts.nav.nextDay}
    onclick={() => onStep(1)}>▶</button
  >
  <button class="today" onclick={onToday}>{texts.nav.today}</button>

  <span class="grow"></span>

  <nav>
    {#each screens as item (item.id)}
      <button
        class="nav"
        class:active={screen === item.id}
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

  .step {
    padding: 0.15rem 0.5rem;
  }

  .day {
    min-width: 9.5rem;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .today {
    padding: 0.2rem 0.6rem;
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
