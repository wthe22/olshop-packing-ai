<script lang="ts">
  // The logo prompt: one unseen logo of the batch, the
  // logo as printed, how many pages and orders carry it, and the answers. Only the view lives
  // here; the flow is in `logoPrompt.ts` and nothing is written by this component.
  import type { LogoAnswer, LogoView } from "./api/batch";
  import { texts } from "../texts";
  import Dialog from "./Dialog.svelte";

  let {
    logo,
    couriers,
    busy = false,
    onAnswer,
    onSkipAll,
  }: {
    logo: LogoView;
    // The courier names of `couriers.toml`, in file order: one button each.
    couriers: string[];
    // An answer is on its way; the buttons wait.
    busy?: boolean;
    onAnswer: (answer: LogoAnswer) => void;
    onSkipAll: () => void;
  } = $props();
</script>

<Dialog title={texts.logo.title} width="46rem">

  <div class="logo">
    {#if logo.picture}
      <img src={logo.picture} alt="" />
    {:else}
      <!-- No page carried the image: the hash is what can still be recognised. -->
      <code class="hash">{logo.hash}</code>
    {/if}
    <span class="counts">{texts.logo.counts(logo.pages, logo.orders)}</span>
  </div>

  <div class="answers">
    {#each couriers as name (name)}
      <button disabled={busy} onclick={() => onAnswer({ kind: "courier", name })}>{name}</button>
    {/each}
    <span class="grow"></span>
    <button disabled={busy} onclick={() => onAnswer({ kind: "not_courier" })}>
      {texts.logo.notCourier}
    </button>
    <button disabled={busy} onclick={() => onAnswer({ kind: "skip" })}>
      {texts.logo.dontKnow}
    </button>
    <button disabled={busy} onclick={onSkipAll}>{texts.logo.skipAll}</button>
  </div>
</Dialog>

<style>
  .logo {
    display: flex;
    align-items: center;
    gap: 1rem;
    min-height: 4rem;
    margin-bottom: 1rem;
  }

  .logo img {
    max-width: 15rem;
    max-height: 5rem;
    /* The logo as printed, on the white it was composited onto. */
    background: #ffffff;
    border: 1px solid var(--border-soft);
    border-radius: 4px;
  }

  .hash {
    padding: 0.5rem 0.75rem;
    font-size: 0.9rem;
    background: var(--surface-2);
    border-radius: 4px;
  }

  .counts {
    margin-left: auto;
    white-space: nowrap;
    color: var(--text-muted);
  }

  .answers {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }

  .grow {
    flex: 1;
  }
</style>
