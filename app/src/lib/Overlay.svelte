<script lang="ts">
  // A screen that opens over the current one and closes back to it (the close button, a click
  // beside it, or Escape). The body is the screen's own content.
  import type { Snippet } from "svelte";
  import { texts } from "../texts";
  import Icon from "./Icon.svelte";

  let {
    title,
    onClose,
    children,
  }: {
    title: string;
    onClose: () => void;
    children: Snippet;
  } = $props();
</script>

<svelte:window onkeydown={(event) => event.key === "Escape" && onClose()} />

<div class="backdrop" role="presentation" onclick={onClose}></div>
<div class="panel" role="dialog" aria-label={title}>
  <header>
    <h2>{title}</h2>
    <span class="grow"></span>
    <button class="icon" title={texts.nav.close} aria-label={texts.nav.close} onclick={onClose}
      ><Icon name="close" /></button
    >
  </header>
  <div class="body">
    {@render children()}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: var(--backdrop);
    z-index: 10;
  }

  /* Closer to the window's edges on a small screen, so the screen keeps its room. */
  .panel {
    position: fixed;
    inset: 3.6rem clamp(0.6rem, 2.5vw, 2.5rem) clamp(0.6rem, 2.5vw, 2rem);
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    z-index: 11;
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: center;
    padding: 0.45rem 0.75rem 0.45rem 1.25rem;
    border-bottom: 1px solid var(--border);
    background: var(--header-bg);
  }

  h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 700;
  }

  .grow {
    flex: 1;
  }

  .body {
    flex: 1;
    overflow: auto;
    padding: 1rem 1.25rem;
  }
</style>
