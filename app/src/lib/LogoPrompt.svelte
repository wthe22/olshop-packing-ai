<script lang="ts">
  // The logo prompt: one unseen logo of the batch, the
  // logo as printed, how many pages and orders carry it, and the answers. Only the view lives
  // here; the flow is in `logoPrompt.ts` and nothing is written by this component.
  import type { LogoAnswer, LogoView } from "./api/batch";
  import { texts } from "../texts";

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

<div class="backdrop" role="presentation"></div>
<div class="panel" role="dialog" aria-label={texts.logo.title}>
  <h2>{texts.logo.title}</h2>

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
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    min-width: 34rem;
    max-width: 90vw;
    padding: 1rem 1.25rem 1.25rem;
    background: var(--surface);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    box-shadow: var(--shadow-lg);
    z-index: 11;
  }

  h2 {
    margin: 0 0 0.75rem;
    font-size: 1.05rem;
  }

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
