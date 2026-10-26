<script lang="ts">
  // The first namespace's name: shown when the data folder holds no namespace yet. The name becomes the
  // namespace's folder in the data folder.
  import { texts } from "../texts";
  import StartCard from "./StartCard.svelte";

  let {
    version,
    busy,
    error,
    onName,
  }: {
    version: string;
    busy: boolean;
    error: string;
    onName: (name: string) => void;
  } = $props();

  let name = $state("");
  let field: HTMLInputElement | undefined = $state();
  $effect(() => field?.focus());
</script>

<StartCard title={texts.namespace.title} {version}>
  <p class="explain">{texts.namespace.ask}</p>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      if (name.trim()) onName(name.trim());
    }}
  >
    <input bind:this={field} bind:value={name} placeholder={texts.namespace.placeholder} maxlength="40" />
    <button class="primary" type="submit" disabled={busy || !name.trim()}
      >{texts.namespace.create}</button
    >
  </form>
  {#if error}
    <p class="refused">{error}</p>
  {/if}
</StartCard>

<style>
  .explain {
    margin: 0 0 1.25rem;
    color: var(--text-muted);
  }

  form {
    display: flex;
    gap: 0.6rem;
  }

  input {
    flex: 1;
    min-width: 0;
  }

  .refused {
    margin: 0.6rem 0 0;
    color: var(--danger);
  }
</style>
