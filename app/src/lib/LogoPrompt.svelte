<script lang="ts">
  // The logo prompt (08 › *2. New batch* → *The logo prompt*): one unseen logo of the batch, the
  // logo as printed, how many pages and orders carry it, and the answers. Only the view lives
  // here; the flow is in `logoPrompt.ts` and nothing is written by this component.
  import type { LogoAnswer, LogoView } from "./api";
  import { texts } from "../texts";

  let {
    logo,
    couriers,
    busy = false,
    onAnswer,
    onSkipAll,
  }: {
    logo: LogoView;
    // The courier names of `couriers.toml`, in file order: one button each (D-LABEL-1).
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
    background: rgba(0, 0, 0, 0.35);
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
    background: #ffffff;
    border: 1px solid #b8b8b8;
    border-radius: 8px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.25);
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
    border: 1px solid #ececec;
    border-radius: 4px;
  }

  .hash {
    padding: 0.5rem 0.75rem;
    font-size: 0.9rem;
    background: #f4f4f4;
    border-radius: 4px;
  }

  .counts {
    margin-left: auto;
    white-space: nowrap;
    color: #4a4a4a;
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

  @media (prefers-color-scheme: dark) {
    .panel {
      background: #242424;
      border-color: #555555;
    }

    .logo img {
      background: #ffffff;
      border-color: #3a3a3a;
    }

    .hash {
      background: #333333;
    }

    .counts {
      color: #b0b0b0;
    }
  }
</style>
