<script lang="ts">
  // The save in flight: the stage and a progress bar
  // while *Save batch* runs, and the findings dialog the check before saving ends with. The dialog's
  // *Save anyway* is handed back to the screen, which runs the save again. Nothing is written here.
  import type { SaveProgress } from "./api/batch";
  import { saveStageText, texts } from "../texts";
  import type { Findings } from "./saveCheck";
  import SaveCheckDialog from "./SaveCheckDialog.svelte";

  let {
    saving,
    progress,
    findings,
    onSaveAnyway,
    onCancel,
  }: {
    saving: boolean;
    // The last `save-progress` event; `null` before the first one arrives.
    progress: SaveProgress | null;
    // What the check before saving caught; `null` while there is nothing to show.
    findings: Findings | null;
    onSaveAnyway: () => void;
    onCancel: () => void;
  } = $props();

  // The save's stage and progress line: the event's stage in plain words, or *Saving …* before
  // the first `save-progress` arrives.
  const savingText = $derived(
    progress ? saveStageText(progress.stage, progress.page, progress.pages) : texts.plan.saving,
  );
</script>

{#if saving}
  <div class="saving">
    <p class="muted">{savingText}</p>
    <progress max={progress?.pages ?? 0} value={progress?.page ?? 0}></progress>
  </div>
{/if}

{#if findings}
  <SaveCheckDialog {findings} busy={saving} {onSaveAnyway} {onCancel} />
{/if}

<style>
  /* The check before saving's stage and progress bar. */
  .saving {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem 0 0.25rem;
  }

  .saving progress {
    width: 100%;
    height: 1.1rem;
  }

  .saving .muted {
    margin: 0;
    color: var(--text-muted);
  }
</style>
