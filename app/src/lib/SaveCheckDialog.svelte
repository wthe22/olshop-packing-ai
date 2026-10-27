<script lang="ts">
  // The check before saving's findings dialog: what
  // the check caught, the errors and the warnings in plain words and kept apart, with *Save
  // anyway* and *Cancel — nothing is saved*. Only the view lives here; the flow is in the Plan
  // screen and nothing is written by this component.
  import type { Findings } from "./saveCheck";
  import { texts } from "../texts";
  import Dialog from "./Dialog.svelte";

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

<Dialog title={texts.plan.checkTitle} width="44rem">

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
</Dialog>

<style>
  .kind {
    margin: 0.75rem 0 0.25rem;
    font-weight: 600;
  }

  .list {
    margin: 0;
    padding-left: 1.2rem;
    color: var(--text-muted);
  }

  .list.errors {
    color: var(--danger);
  }

  .buttons {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }
</style>
