<script lang="ts">
  // The shown set's file as text: the whole
  // `packing-workflows.toml`, checked as it is typed and saved as it stands, comments and all.
  import { texts } from "../texts";

  let {
    text,
    error,
    busy,
    onText,
    onCancel,
    onSave,
  }: {
    text: string;
    // What the last check of the text said, in plain words; empty when the file reads.
    error: string;
    busy: boolean;
    onText: (text: string) => void;
    onCancel: () => void;
    onSave: () => void;
  } = $props();
</script>

<div class="file">
  <label for="file-text">{texts.sortRules.fileText}</label>
  <textarea
    id="file-text"
    rows="18"
    value={text}
    oninput={(event) => onText(event.currentTarget.value)}
  ></textarea>
  {#if error}
    <p class="error">{error}</p>
  {/if}
  <div class="actions">
    <span class="grow"></span>
    <button onclick={onCancel}>{texts.sortRules.cancel}</button>
    <button class="primary" disabled={!!error || busy} onclick={onSave}
      >{texts.sortRules.save}</button
    >
  </div>
</div>

<style>
  .file label {
    display: block;
    margin-bottom: 0.3rem;
    font-weight: 600;
  }

  .file textarea {
    width: 100%;
    font-family: Consolas, monospace;
    font-size: 0.9rem;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border);
  }

  .grow {
    flex: 1;
  }
</style>
