<script lang="ts">
  // A dialog over the window: a backdrop that takes the clicks, and a panel in the middle with a
  // title and the caller's content. What its buttons and keys do is the caller's.
  import type { Snippet } from "svelte";

  let {
    title,
    width = "30rem",
    children,
  }: {
    title: string;
    // The panel's width; a narrow window takes 90% of its own.
    width?: string;
    children: Snippet;
  } = $props();
</script>

<div class="backdrop" role="presentation"></div>
<div class="panel" role="dialog" aria-modal="true" aria-label={title} style:--width={width}>
  <h2>{title}</h2>
  {@render children()}
</div>

<style>
  /* Above the screens that open over the current one, so a question asked on Search shows. */
  .backdrop {
    position: fixed;
    inset: 0;
    background: var(--backdrop);
    z-index: 20;
  }

  .panel {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(90vw, var(--width));
    max-height: 80vh;
    overflow: auto;
    padding: 1rem 1.25rem 1.25rem;
    background: var(--surface);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    z-index: 21;
  }

  h2 {
    margin: 0 0 0.75rem;
    font-size: 1.05rem;
  }
</style>
