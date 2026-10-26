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

  .panel {
    position: fixed;
    inset: 3.5rem 2rem 2rem;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: var(--shadow-lg);
    z-index: 11;
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: center;
    padding: 0.6rem 0.75rem 0.6rem 1.25rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  h2 {
    margin: 0;
    font-size: 1.05rem;
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
