<script lang="ts">
  // The card the first start asks its questions on: the program's name and version over the
  // question, centred in the window.
  import type { Snippet } from "svelte";
  import Logo from "./Logo.svelte";
  import { texts } from "../texts";

  let { title, version, children }: { title: string; version: string; children: Snippet } =
    $props();
</script>

<main>
  <section class="card">
    <div class="brand">
      <Logo size={34} />
      <span>{texts.title}</span>
      {#if version}<span class="version">{version}</span>{/if}
    </div>
    <h1>{title}</h1>
    {@render children()}
  </section>
</main>

<style>
  main {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 2rem;
  }

  .card {
    width: min(34rem, 100%);
    padding: 1.75rem 2rem 2rem;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--surface);
    box-shadow: var(--shadow-lg);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .version {
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-faint);
  }

  h1 {
    margin: 1.25rem 0 0.5rem;
    font-size: 1.35rem;
    font-weight: 650;
  }
</style>
