<script lang="ts">
  // The question at start when the data folder holds data this version keeps more of (start.md,
  // *Update the data folder?*): how many dates and that a copy is kept, then **Update** with one
  // line and a bar for the whole run, or **Open read-only**. No date opens before the answer; when
  // all are done the Day screen opens, after one line when a batch was not read.
  import type { MigrationView } from "./api/start";
  import { updateTexts as words } from "../updateTexts";
  import StartCard from "./StartCard.svelte";
  import { Update } from "./update.svelte";

  let {
    version,
    dates,
    // **Open read-only** that failed.
    error,
    onDone,
    onReadOnly,
  }: {
    version: string;
    dates: MigrationView[];
    error: string;
    onDone: () => void;
    onReadOnly: () => void;
  } = $props();

  const update = new Update();

  async function run() {
    if (await update.run()) onDone();
  }
</script>

<StartCard title={words.title} {version}>
  <p class="explain">{words.explain(dates.length)}</p>

  {#if update.skipped.length > 0}
    <p class="skipped">{words.skipped(update.skipped)}</p>
    <div class="buttons">
      <button class="primary" onclick={onDone}>{words.continue}</button>
    </div>
  {:else if update.running && update.line}
    <div class="progress">
      <p class="line">{update.line}</p>
      <progress max={update.of} value={update.done}></progress>
      <p class="step">{update.step ?? ""}</p>
    </div>
  {:else}
    {#if update.error}
      <p class="error">{words.failed(update.error)}</p>
    {:else if error}
      <p class="error">{error}</p>
    {/if}
    <div class="buttons">
      <button disabled={update.running} onclick={onReadOnly}>{words.readOnly}</button>
      <button class="primary" disabled={update.running} onclick={run}>{words.update}</button>
    </div>
  {/if}
</StartCard>

<style>
  .explain {
    margin: 0;
    color: var(--text-muted);
  }

  .progress {
    margin-top: 1.5rem;
  }

  .progress p {
    margin: 0;
    font-variant-numeric: tabular-nums;
  }

  progress {
    display: block;
    width: 100%;
    height: 1.1rem;
    margin: 0.5rem 0;
  }

  /* The step: room kept, so the card does not move between steps. */
  .step {
    min-height: 1.4em;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .skipped {
    margin: 1rem 0 0;
    color: var(--danger);
  }

  .error {
    margin: 0.75rem 0 0;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    margin-top: 1.5rem;
  }
</style>
