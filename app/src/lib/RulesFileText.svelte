<script lang="ts">
  // The whole `packing-workflows.toml` as text, comments and all, checked as it is typed. The
  // screen's foot holds *Cancel* and *Save*.
  import { texts } from "../texts";

  let {
    text,
    error,
    onText,
  }: {
    text: string;
    // What the last check of the text said, in plain words; empty when the file reads.
    error: string;
    onText: (text: string) => void;
  } = $props();
</script>

<div class="file">
  <label for="file-text">{texts.sortRules.fileText}</label>
  <textarea
    id="file-text"
    rows="22"
    spellcheck="false"
    value={text}
    oninput={(event) => onText(event.currentTarget.value)}
  ></textarea>
  {#if error}
    <p class="error">{error}</p>
  {/if}
</div>

<style>
  .file label {
    display: block;
    margin-bottom: 0.3rem;
    font-weight: 600;
  }

  .file textarea {
    width: 100%;
    font-family: var(--font-mono);
    font-size: 0.9rem;
  }

  .error {
    margin: 0.5rem 0 0;
  }
</style>
