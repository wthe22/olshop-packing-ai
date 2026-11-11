<script lang="ts">
  // The Shops section of an account's card: one field per shop name with its ✕, and **+ Add shop**.
  // A name listed twice, or by another account, is told under its field. Saving is the screen's.
  import { untrack } from "svelte";
  import { texts } from "../texts";
  import type { ShopProblem } from "./shops";
  import Icon from "./Icon.svelte";

  let {
    names = $bindable(),
    problems,
    refused,
    onAdd,
    onRemove,
  }: {
    names: string[];
    problems: (ShopProblem | null)[];
    // Why the back end refused the list on Save.
    refused: string;
    onAdd: () => void;
    onRemove: (index: number) => void;
  } = $props();

  const words = texts.accounts;
</script>

<div class="shops">
  <h4>{words.shops}<span class="hint">{words.shopsHint}</span></h4>
  {#if names.length === 0}
    <p class="none">{words.noShops}</p>
  {/if}
  {#each names as _, index (index)}
    {@const problem = problems[index]}
    <div class="field">
      <input
        bind:value={names[index]}
        aria-label={words.shopLabel(index + 1)}
        {@attach (input) =>
          untrack(() => {
            if (index === names.length - 1 && names[index] === "") input.focus();
          })}
      />
      <button
        type="button"
        class="icon"
        title={words.removeShop(names[index])}
        aria-label={words.removeShop(names[index])}
        onclick={() => onRemove(index)}><Icon name="close" size={16} /></button
      >
      {#if problem && problem.kind !== "empty"}
        <p class="refused">{words.problem(problem)}</p>
      {/if}
    </div>
  {/each}
  {#if refused}<p class="refused">{refused}</p>{/if}
  <div><button type="button" class="ghost" onclick={onAdd}>{words.addShop}</button></div>
</div>

<style>
  .shops {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.6rem 1rem 0.8rem;
  }

  h4 {
    margin: 0;
    color: var(--text-muted);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .hint {
    margin-left: 0.75rem;
    color: var(--text-faint);
    font-size: 0.88rem;
    font-weight: 400;
    letter-spacing: 0;
    text-transform: none;
  }

  .none {
    margin: 0;
    color: var(--text-muted);
  }

  .field {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
  }

  .field input {
    flex: 1;
    max-width: 24rem;
  }

  .refused {
    flex-basis: 100%;
    margin: 0;
    color: var(--danger);
    font-size: 0.88rem;
  }
</style>
