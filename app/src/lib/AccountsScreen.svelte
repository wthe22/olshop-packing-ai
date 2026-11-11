<script lang="ts">
  // Accounts: one account at a time as a tab, its shops, its dates and its name. The state and the
  // commands are `accountsState.svelte.ts`; here the pieces are wired and the foot kept in view.
  import { onDestroy, onMount } from "svelte";
  import { texts } from "../texts";
  import { AccountsState } from "./accountsState.svelte";
  import AccountTabs from "./AccountTabs.svelte";
  import AccountHead from "./AccountHead.svelte";
  import AccountShops from "./AccountShops.svelte";
  import AccountDates from "./AccountDates.svelte";

  let {
    today,
    onOpenDate,
  }: {
    today: string;
    onOpenDate: (date: string) => void;
  } = $props();

  const screen = new AccountsState();
  const words = texts.accounts;

  onMount(screen.load);
  onDestroy(screen.stop);
</script>

<div class="accounts">
  {#if screen.error}
    <p class="error">{screen.error}</p>
  {/if}
  <AccountTabs
    accounts={screen.accounts}
    current={screen.current}
    onSelect={screen.select}
    onStartNew={screen.startNew}
    onCreate={screen.create}
  />

  {#if screen.account}
    <section class="card">
      <AccountHead name={screen.account.name} onRename={screen.rename} onDelete={screen.remove} />
      <AccountShops
        bind:names={screen.names}
        problems={screen.problems}
        refused={screen.refused}
        onAdd={screen.addShop}
        onRemove={screen.removeShop}
      />
      <AccountDates dates={screen.dates} {today} {onOpenDate} />
    </section>
  {/if}

  <span class="spacer"></span>
  <footer class="foot">
    <span class="grow"></span>
    {#if screen.dirty}
      <span class="unsaved">{words.unsaved}</span>
    {/if}
    <button disabled={!screen.dirty} onclick={screen.cancelShops}>{words.cancel}</button>
    <button
      class="primary"
      title={screen.blocked}
      disabled={!screen.dirty || screen.saving || screen.blocked !== ""}
      onclick={screen.saveShops}>{words.save}</button
    >
  </footer>
</div>

<style>
  /* The foot stays at the bottom of the screen, and in view while the dates scroll. */
  .accounts {
    display: flex;
    flex-direction: column;
    min-height: 100%;
  }

  .card {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
    overflow: hidden;
  }

  .grow {
    flex: 1;
  }

  .foot {
    position: sticky;
    bottom: -1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0 -1.25rem -1rem;
    padding: 0.65rem 1.25rem;
    border-top: 1px solid var(--border);
    background: var(--header-bg);
  }

  .spacer {
    flex: 1 0 1rem;
  }

  .unsaved {
    margin-right: 0.5rem;
    color: var(--warn);
    font-size: 0.9rem;
  }
</style>
