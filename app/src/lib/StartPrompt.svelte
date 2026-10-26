<script lang="ts">
  // The Start prompt: shown at every start until `packing-settings.toml` beside the program names
  // the data folder.
  import { texts } from "../texts";
  import Icon from "./Icon.svelte";
  import StartCard from "./StartCard.svelte";

  let {
    exeDir,
    version,
    busy,
    onUse,
    onChoose,
  }: {
    exeDir: string;
    version: string;
    busy: boolean;
    onUse: () => void;
    onChoose: () => void;
  } = $props();
</script>

<StartCard title={texts.start.ask} {version}>
  <p class="explain">{texts.start.explain}</p>
  <div class="folder">
    <Icon name="folder" />
    <div>
      <div class="label">{texts.start.nextToProgram}</div>
      <div class="path">{texts.start.folder(exeDir)}</div>
    </div>
  </div>
  <div class="buttons">
    <button disabled={busy} onclick={onChoose}>{texts.start.chooseFolder}</button>
    <button class="primary" disabled={busy} onclick={onUse}>{texts.start.useThisFolder}</button>
  </div>
</StartCard>

<style>
  .explain {
    margin: 0 0 1.25rem;
    color: var(--text-muted);
  }

  .folder {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem 0.9rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
    color: var(--text-muted);
  }

  .label {
    color: var(--text);
    font-weight: 600;
  }

  .path {
    font-family: Consolas, monospace;
    font-size: 0.9rem;
    overflow-wrap: anywhere;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    margin-top: 1.5rem;
  }
</style>
