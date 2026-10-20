<script lang="ts">
  // The check before saving's findings dialog: what
  // the check caught, the errors and the warnings in plain words and kept apart, with *Save
  // anyway* and *Cancel — nothing is saved*. Only the view lives here; the flow is in the Plan
  // screen and nothing is written by this component.
  import type { Findings } from "./saveCheck";
  import { texts } from "../texts";

  let {
    findings,
    busy = false,
    onSaveAnyway,
    onCancel,
  }: {
    findings: Findings;
    // *Save anyway* is on its way; the buttons wait.
    busy?: boolean;
    onSaveAnyway: () => void;
    onCancel: () => void;
  } = $props();
</script>

<div class="backdrop" role="presentation"></div>
<div class="panel" role="dialog" aria-label={texts.plan.checkTitle}>
  <h2>{texts.plan.checkTitle}</h2>

  {#if findings.errors.length > 0}
    <p class="kind">{texts.plan.checkErrors(findings.errors.length)}</p>
    <ul class="list errors">
      {#each findings.errors as error}
        <li>{error}</li>
      {/each}
    </ul>
  {/if}

  {#if findings.warnings.length > 0}
    <p class="kind">{texts.plan.checkWarnings(findings.warnings.length)}</p>
    <ul class="list">
      {#each findings.warnings as warning}
        <li>{warning}</li>
      {/each}
    </ul>
  {/if}

  <div class="buttons">
    <button class="primary" disabled={busy} onclick={onSaveAnyway}>{texts.plan.saveAnyway}</button>
    <button disabled={busy} onclick={onCancel}>{texts.plan.saveCancel}</button>
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
    width: 90vw;
    max-width: 44rem;
    max-height: 80vh;
    overflow: auto;
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

  .kind {
    margin: 0.75rem 0 0.25rem;
    font-weight: 600;
  }

  .list {
    margin: 0;
    padding-left: 1.2rem;
    color: #4a4a4a;
  }

  .list.errors {
    color: #8a1f1f;
  }

  .buttons {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  @media (prefers-color-scheme: dark) {
    .panel {
      background: #242424;
      border-color: #555555;
    }

    .list {
      color: #b0b0b0;
    }

    .list.errors {
      color: #f0b8b8;
    }
  }
</style>
