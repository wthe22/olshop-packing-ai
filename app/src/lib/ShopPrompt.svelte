<script lang="ts">
  // The account question: after the logo prompt, before the Plan screen, one row per shop name no
  // account has and per file with labels that have no shop name, each with one list
  // (`shopPrompt.ts`). *New account* shows a box for its name. **Back** (also Esc, the caller's)
  // returns to the file list; **Continue** writes the answers and goes on to the plan.
  import { untrack } from "svelte";
  import type { ShopAnswer, ShopPromptView } from "./api/batch";
  import { accountRows, canContinue, choiceOf, listOf, shopAnswer, valueOf, type AccountRow } from "./shopPrompt";
  import { texts } from "../texts";
  import Dialog from "./Dialog.svelte";
  import Icon from "./Icon.svelte";

  let {
    prompt,
    onBack,
    onContinue,
  }: {
    prompt: ShopPromptView;
    onBack: () => void;
    // Resolves to why the answers were refused, or `null` when the read went on.
    onContinue: (answer: ShopAnswer) => Promise<string | null>;
  } = $props();

  const words = texts.shops;

  // The rows start from the prompt as it is when the dialog opens.
  let rows = $state<AccountRow[]>(untrack(() => accountRows(prompt)));
  let refused = $state("");
  let busy = $state(false);

  const ready = $derived(canContinue(rows));

  const who = (row: AccountRow) =>
    row.subject.kind === "shop" ? row.subject.name : `${row.subject.file}, ${words.noShopName}`;

  async function go() {
    busy = true;
    refused = (await onContinue(shopAnswer(rows))) ?? "";
    busy = false;
  }
</script>

<Dialog title={words.title} width="46rem">
  <p class="intro">{words.intro}</p>
  <ul>
    {#each rows as row (row.key)}
      <li>
        <span class="who">
          {#if row.subject.kind === "shop"}
            <strong>{row.subject.name}</strong>
          {:else}
            <span class="muted">{words.noShopName}</span>
            <span class="file" title={row.subject.file}><Icon name="file" size={14} />{row.subject.file}</span>
          {/if}
        </span>
        <span class="orders">{words.orders(row.orders)}</span>
        <span class="account">
          <select
            value={valueOf(rows, row)}
            aria-label={words.accountOf(who(row))}
            onchange={(event) => (row.choice = choiceOf(row, event.currentTarget.value))}
          >
            <option value="" disabled>{words.choose}</option>
            {#each listOf(rows, row, prompt.accounts) as entry (entry.value)}
              <option value={entry.value}>{entry.text}</option>
            {/each}
          </select>
          {#if row.choice.kind === "new"}
            <input bind:value={row.choice.name} aria-label={words.newName} placeholder={words.newName} />
          {/if}
        </span>
      </li>
    {/each}
  </ul>
  {#if refused}<p class="refused">{refused}</p>{/if}
  <div class="answers">
    <button onclick={onBack}>{words.back}</button>
    <button class="primary" disabled={!ready || busy} onclick={go}>{words.continue}</button>
  </div>
</Dialog>

<style>
  .intro {
    margin: 0 0 0.75rem;
    color: var(--text-muted);
  }

  ul {
    display: flex;
    flex-direction: column;
    margin: 0 0 1rem;
    padding: 0;
    list-style: none;
  }

  li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 6rem 16rem;
    align-items: start;
    gap: 0.75rem;
    padding: 0.5rem 0;
    border-top: 1px solid var(--border);
  }

  li:last-child {
    border-bottom: 1px solid var(--border);
  }

  .who {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem 0.5rem;
    min-width: 0;
    min-height: 2rem;
    overflow-wrap: anywhere;
  }

  .muted {
    color: var(--text-muted);
  }

  .orders {
    display: flex;
    align-items: center;
    min-height: 2rem;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .account {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .account select,
  .account input {
    width: 100%;
    box-sizing: border-box;
  }

  .file {
    max-width: 100%;
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    min-width: 0;
    padding: 0.05rem 0.45rem;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
    font-family: var(--font-mono);
    font-size: 0.8rem;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .file :global(svg) {
    flex: none;
  }

  .refused {
    margin: 0 0 0.75rem;
    color: var(--danger);
  }

  .answers {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>
