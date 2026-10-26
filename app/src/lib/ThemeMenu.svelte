<script lang="ts">
  // The title bar's theme menu: Light, Dark or System (Windows' own choice). The button shows the
  // theme in use; the menu closes on a choice, a click elsewhere or Escape.
  import Icon from "./Icon.svelte";
  import { texts } from "../texts";
  import { themes, type Theme } from "./theme";

  let { theme, onChoose }: { theme: Theme; onChoose: (theme: Theme) => void } = $props();

  let open = $state(false);
  let root: HTMLElement | undefined = $state();

  function outside(event: MouseEvent) {
    if (open && root && !root.contains(event.target as Node)) open = false;
  }
</script>

<svelte:window onclick={outside} onkeydown={(event) => event.key === "Escape" && (open = false)} />

<div class="theme" bind:this={root}>
  <button
    class="icon"
    title={texts.theme.title}
    aria-label={texts.theme.title}
    aria-haspopup="menu"
    aria-expanded={open}
    onclick={() => (open = !open)}><Icon name={theme} /></button
  >
  {#if open}
    <div class="menu" role="menu">
      {#each themes as choice (choice)}
        <button
          role="menuitemradio"
          aria-checked={choice === theme}
          class:chosen={choice === theme}
          onclick={() => {
            open = false;
            onChoose(choice);
          }}
        >
          <Icon name={choice} size={16} />
          <span class="label">{texts.theme[choice]}</span>
          {#if choice === theme}<Icon name="check" size={16} />{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .theme {
    position: relative;
  }

  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 0.35rem);
    z-index: 20;
    min-width: 11rem;
    padding: 0.3rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    box-shadow: var(--shadow-lg);
  }

  .menu button {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    padding: 0.4rem 0.6rem;
    border: none;
    background: none;
    text-align: left;
  }

  .menu button:hover {
    background: var(--surface-hover);
  }

  .menu button.chosen {
    color: var(--link);
  }

  .label {
    flex: 1;
  }
</style>
