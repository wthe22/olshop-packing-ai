<script lang="ts">
  // The Start prompt (08 › *Start*): shown at every start until `categories.toml` is found next
  // to the program.
  import { texts } from "../texts";

  let {
    exeDir,
    busy,
    onUse,
    onChoose,
  }: {
    exeDir: string;
    busy: boolean;
    onUse: () => void;
    onChoose: () => void;
  } = $props();
</script>

<main>
  <p class="message">{texts.start.noData(exeDir)}</p>
  <div class="buttons">
    <button class="primary" disabled={busy} onclick={onUse}
      >{texts.start.useThisFolder}</button
    >
    <button disabled={busy} onclick={onChoose}>{texts.start.chooseFolder}</button>
  </div>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.25rem;
    height: 100vh;
    padding: 2rem;
    text-align: center;
  }

  .message {
    max-width: 40rem;
    margin: 0;
    font-size: 1.05rem;
  }

  .buttons {
    display: flex;
    gap: 0.75rem;
  }
</style>
