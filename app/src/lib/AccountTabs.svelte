<script lang="ts">
  // The account row of the Accounts screen: one tab per account, in the order they were made, and
  // **+ New account**, which asks the name in a dialog. The screen does the making and the asking; a
  // refused name is shown under the field.
  import { tick } from "svelte";
  import type { Account } from "./api/accounts";
  import { texts } from "../texts";
  import Dialog from "./Dialog.svelte";
  import Icon from "./Icon.svelte";

  let {
    accounts,
    current,
    onSelect,
    onStartNew,
    onCreate,
  }: {
    accounts: Account[];
    current: string | null;
    onSelect: (name: string) => void;
    // Says whether the name field may open: the owner may keep the shown account's unsaved shops.
    onStartNew: () => Promise<boolean>;
    // Resolves to why the name is refused, or `null` when the account was made.
    onCreate: (name: string) => Promise<string | null>;
  } = $props();

  const words = texts.accounts;

  let creating = $state(false);
  let name = $state("");
  let refused = $state("");
  let nameInput = $state<HTMLInputElement>();

  async function open() {
    if (!(await onStartNew())) return;
    creating = true;
    name = "";
    refused = "";
    await tick();
    nameInput?.focus();
  }

  async function submit() {
    if (name.trim() === "") return;
    refused = (await onCreate(name.trim())) ?? "";
    if (!refused) creating = false;
  }
</script>

<div class="bar">
  {#if accounts.length > 0}
    <div class="tabs" role="tablist" aria-label={words.title}>
      {#each accounts as account (account.name)}
        <button
          role="tab"
          aria-selected={account.name === current}
          class:chosen={account.name === current}
          onclick={() => onSelect(account.name)}>{account.name}</button
        >
      {/each}
    </div>
  {/if}
  <button class="ghost" onclick={open}><Icon name="plus" size={16} />{words.newAccount}</button>
</div>

{#if accounts.length === 0 && !creating}
  <p class="none">{words.none}</p>
{/if}

{#if creating}
  <Dialog title={words.newAccount}>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        submit();
      }}
    >
      <input
        bind:this={nameInput}
        bind:value={name}
        aria-label={words.nameLabel}
        placeholder={words.namePlaceholder}
        maxlength="40"
        onkeydown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            creating = false;
          }
        }}
      />
      {#if refused}<p class="refused">{refused}</p>{/if}
      <div class="buttons">
        <button class="primary" type="submit" disabled={name.trim() === ""}>{words.create}</button>
        <button type="button" onclick={() => (creating = false)}>{words.cancel}</button>
      </div>
    </form>
  </Dialog>
{/if}

<style>
  .bar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  /* The accounts side by side, as Settings' theme choice: the one shown is raised. */
  .tabs {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
  }

  .tabs button {
    padding: 0.3rem 0.85rem;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-muted);
  }

  .tabs button:hover:not(.chosen) {
    border-color: transparent;
    background: var(--surface-hover);
    color: var(--text);
  }

  .tabs button.chosen {
    border-color: var(--border);
    background: var(--surface);
    color: var(--text);
    font-weight: 600;
    box-shadow: var(--shadow);
    cursor: default;
  }

  .none {
    margin: 0 0 0.75rem;
    color: var(--text-muted);
  }

  input {
    box-sizing: border-box;
    width: 100%;
  }

  .refused {
    margin: 0.4rem 0 0;
    color: var(--danger);
    font-size: 0.88rem;
  }

  .buttons {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }
</style>
