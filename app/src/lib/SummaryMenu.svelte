<script lang="ts">
  // A summary's button (*Day summary*, a batch's *Summary PDF*): one button, or, when its batches
  // hold more than one account, a menu of *All accounts* and each account.
  import { texts } from "../texts";

  let {
    label,
    title,
    accounts,
    disabled,
    onWrite,
  }: {
    label: string;
    title: string;
    // The accounts of the summary's batches; the button is a menu when there are several.
    accounts: string[];
    disabled: boolean;
    // `null`: every account.
    onWrite: (account: string | null) => void;
  } = $props();

  let menuOpen = $state(false);

  function choose(account: string | null) {
    menuOpen = false;
    onWrite(account);
  }
</script>

{#if accounts.length > 1}
  <span class="menu">
    <button
      {title}
      {disabled}
      aria-haspopup="menu"
      aria-expanded={menuOpen}
      onclick={() => (menuOpen = !menuOpen)}>{label} ▾</button
    >
    {#if menuOpen}
      <span class="items" role="menu">
        <button role="menuitem" onclick={() => choose(null)}>{texts.day.allAccounts}</button>
        {#each accounts as account (account)}
          <button role="menuitem" onclick={() => choose(account)}>{account}</button>
        {/each}
      </span>
    {/if}
  </span>
{:else}
  <button {title} {disabled} onclick={() => onWrite(null)}>{label}</button>
{/if}

<style>
  .menu {
    position: relative;
  }

  .items {
    position: absolute;
    right: 0;
    top: 100%;
    z-index: 5;
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-top: 2px;
    padding: 0.3rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    box-shadow: var(--shadow-lg);
  }

  .items button {
    border: none;
    background: none;
    text-align: left;
    white-space: nowrap;
  }

  .items button:hover {
    background: var(--surface-hover);
  }
</style>
