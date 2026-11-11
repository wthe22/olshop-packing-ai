<script lang="ts">
  // The head of the shown account's card: its name, **Rename…** (the name edited in place, a refused
  // name told under the field) and **Delete account**.
  import { tick } from "svelte";
  import { texts } from "../texts";

  let {
    name,
    onRename,
    onDelete,
  }: {
    name: string;
    // Resolves to why the name is refused, or `null` when it was renamed.
    onRename: (name: string) => Promise<string | null>;
    onDelete: () => void;
  } = $props();

  const words = texts.accounts;

  let renaming = $state(false);
  let typed = $state("");
  let refused = $state("");
  let nameInput = $state<HTMLInputElement>();

  async function start() {
    renaming = true;
    typed = name;
    refused = "";
    await tick();
    nameInput?.select();
  }

  async function submit() {
    const wanted = typed.trim();
    if (wanted === "") return;
    refused = wanted === name ? "" : ((await onRename(wanted)) ?? "");
    if (!refused) renaming = false;
  }
</script>

<div class="head">
  {#if renaming}
    <form
      class="naming"
      onsubmit={(event) => {
        event.preventDefault();
        submit();
      }}
    >
      <input
        bind:this={nameInput}
        bind:value={typed}
        aria-label={words.renameLabel(name)}
        maxlength="40"
        onkeydown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            renaming = false;
          }
        }}
      />
      <button type="button" onclick={() => (renaming = false)}>{words.cancel}</button>
      <button class="primary" type="submit" disabled={typed.trim() === ""}>{words.renameDone}</button>
    </form>
    {#if refused}<p class="refused">{refused}</p>{/if}
    <p class="hint">{words.renameHint}</p>
  {:else}
    <h3>{name}</h3>
    <span class="grow"></span>
    <button onclick={start}>{words.rename}</button>
    <button class="danger" onclick={onDelete}>{words.delete}</button>
  {/if}
</div>

<style>
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1rem;
    border-bottom: 1px solid var(--border-soft);
    background: var(--surface-2);
  }

  h3 {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 700;
  }

  .grow {
    flex: 1;
  }

  .naming {
    display: flex;
    gap: 0.5rem;
  }

  .naming input {
    width: 16rem;
  }

  button.danger:hover:not(:disabled) {
    border-color: var(--danger-border);
    background: var(--danger-bg);
    color: var(--danger);
  }

  .refused,
  .hint {
    flex-basis: 100%;
    margin: 0;
    font-size: 0.88rem;
  }

  .refused {
    color: var(--danger);
  }

  .hint {
    color: var(--text-muted);
  }
</style>
