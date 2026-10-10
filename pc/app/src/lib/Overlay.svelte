<script lang="ts">
  // A screen that opens over the current one and closes back to it (08 › *Window frame*:
  // Categories and Settings). The body is the screen's own content.
  import type { Snippet } from "svelte";
  import { texts } from "../texts";

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

<div class="backdrop" role="presentation" onclick={onClose}></div>
<div class="panel" role="dialog" aria-label={title}>
  <header>
    <h2>{title}</h2>
    <span class="grow"></span>
    <button onclick={onClose}>{texts.categories.close}</button>
  </header>
  <div class="body">
    {@render children()}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    z-index: 10;
  }

  .panel {
    position: fixed;
    inset: 3rem 2rem 2rem;
    display: flex;
    flex-direction: column;
    background: #ffffff;
    border: 1px solid #b8b8b8;
    border-radius: 8px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.25);
    z-index: 11;
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: center;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid #d5d5d5;
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

  @media (prefers-color-scheme: dark) {
    .panel {
      background: #242424;
      border-color: #555555;
    }

    header {
      border-bottom-color: #3a3a3a;
    }
  }
</style>
