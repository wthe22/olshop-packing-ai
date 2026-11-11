<script lang="ts">
  // The shop prompt: after the logo prompt, before the Plan screen, each sender no account lists
  // gets an account (a listed one, or a new one named after the shop), and each file whose labels
  // show no sender gets the account for this read. **Back** (also Esc, the caller's) returns to the
  // file list; **Continue** writes the answers and goes on to the plan.
  import { untrack } from "svelte";
  import type { ShopAnswer, ShopPromptView } from "./api/batch";
  import {
    canContinue,
    noSenderDrafts,
    shopAnswer,
    shopDrafts,
    type NoSenderDraft,
    type ShopDraft,
  } from "./shopPrompt";
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
  let shops = $state<ShopDraft[]>(untrack(() => shopDrafts(prompt)));
  let files = $state<NoSenderDraft[]>(untrack(() => noSenderDrafts(prompt)));
  let refused = $state("");
  let busy = $state(false);

  const ready = $derived(canContinue(shops, files));

  function pick(shop: ShopDraft, value: string) {
    shop.choice = value === "" ? { kind: "none" } : { kind: "listed", account: value };
  }

  async function go() {
    busy = true;
    refused = (await onContinue(shopAnswer(shops, files))) ?? "";
    busy = false;
  }
</script>

<Dialog title={words.title} width="52rem">
  {#if shops.length > 0}
    <p class="intro">{words.shopsIntro}</p>
    <ul>
      {#each shops as shop, index (shop.read)}
        <li>
          <label class="shop">
            <span class="label">{words.shop}</span>
            <input bind:value={shop.name} aria-label={words.shopField(shop.read)} />
          </label>
          <span class="orders">{words.shopOrders(prompt.shops[index].orders, prompt.shops[index].files)}</span>
          {#if shop.choice.kind === "new"}
            <input
              class="account"
              bind:value={shop.choice.name}
              aria-label={words.newAccountName(shop.name)}
            />
            <button onclick={() => pick(shop, "")}>{words.listedAccount}</button>
          {:else}
            <select
              value={shop.choice.kind === "listed" ? shop.choice.account : ""}
              aria-label={words.accountOf(shop.name)}
              onchange={(event) => pick(shop, event.currentTarget.value)}
            >
              <option value="">{words.choose}</option>
              {#each prompt.accounts as account (account)}
                <option value={account}>{account}</option>
              {/each}
            </select>
            <button onclick={() => (shop.choice = { kind: "new", name: shop.name.trim() })}
              >{words.newAccount}</button
            >
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
  {#if files.length > 0}
    <p class="intro">{words.noSenderIntro}</p>
    <ul>
      {#each files as file (file.row.input)}
        <li class="file-row">
          <span class="file" title={file.row.file}><Icon name="file" size={14} />{file.row.file}</span>
          <span class="orders">{words.noSenderOrders(file.row.orders, file.row.of_orders)}</span>
          <select
            value={file.account ?? ""}
            aria-label={words.accountOfFile(file.row.file)}
            onchange={(event) => (file.account = event.currentTarget.value || null)}
          >
            <option value="">{words.choose}</option>
            {#each prompt.accounts as account (account)}
              <option value={account}>{account}</option>
            {/each}
          </select>
        </li>
      {/each}
    </ul>
  {/if}
  {#if refused}<p class="refused">{refused}</p>{/if}
  <div class="answers">
    <button onclick={onBack}>{words.back}</button>
    <button class="primary" disabled={!ready || busy} onclick={go}>{words.continue}</button>
  </div>
</Dialog>

<style>
  .intro {
    margin: 0 0 0.5rem;
    color: var(--text-muted);
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin: 0 0 1rem;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 0.75rem;
  }

  .shop {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .shop input {
    width: 14rem;
  }

  .label {
    color: var(--text-muted);
  }

  .account {
    width: 12rem;
  }

  .orders {
    flex: 1;
    min-width: 8rem;
    font-variant-numeric: tabular-nums;
  }

  .file {
    max-width: 22rem;
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
