<script lang="ts">
  // The first owner's name: shown when the data folder holds no owner yet. The name becomes the
  // owner's folder in the data folder.
  import { texts } from "../texts";

  let {
    busy,
    error,
    onName,
  }: {
    busy: boolean;
    error: string;
    onName: (name: string) => void;
  } = $props();

  let name = $state("");
</script>

<main>
  <p class="message">{texts.owner.ask}</p>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      if (name.trim()) onName(name.trim());
    }}
  >
    <input bind:value={name} placeholder={texts.owner.placeholder} maxlength="40" />
    <button class="primary" type="submit" disabled={busy || !name.trim()}
      >{texts.owner.create}</button
    >
  </form>
  {#if error}
    <p class="error">{error}</p>
  {/if}
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

  form {
    display: flex;
    gap: 0.75rem;
  }

  input {
    width: 18rem;
  }
</style>
