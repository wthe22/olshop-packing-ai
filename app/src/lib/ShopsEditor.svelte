<script lang="ts">
  // Settings › Workspaces › **Shops**: a workspace's shop names in place of its row, one field each
  // with a ✕, **+ Add shop**, **Cancel** and **Save**. A name listed twice is told under its field;
  // it, or an empty name, keeps **Save** off with the reason as its tooltip. What the back end
  // refuses is told under the list.
  import { untrack } from "svelte";
  import { texts } from "../texts";
  import Icon from "./Icon.svelte";
  import { shopProblems } from "./shops";

  let {
    workspace,
    shops,
    onCancel,
    onSave,
  }: {
    workspace: string;
    shops: string[];
    onCancel: () => void;
    // Resolves to why the list was refused, or `null` when it was written.
    onSave: (shops: string[]) => Promise<string | null>;
  } = $props();

  const words = texts.settings;

  // The list as it is when the editor opens; a workspace with no shop starts with one empty field.
  const opened = () => (shops.length > 0 ? [...shops] : [""]);
  let names = $state(opened());
  let refused = $state("");
  let saving = $state(false);

  const problems = $derived(shopProblems(names));
  const blocked = $derived(
    problems.includes("empty") ? words.shopEmpty : problems.includes("twice") ? words.shopTwice : "",
  );

  function add() {
    names = [...names, ""];
  }

  function remove(index: number) {
    names = names.filter((_, at) => at !== index);
  }

  async function save() {
    saving = true;
    refused = (await onSave(names.map((name) => name.trim()))) ?? "";
    saving = false;
  }
</script>

<form
  class="shops"
  aria-label={`${words.shops}: ${workspace}`}
  onsubmit={(event) => {
    event.preventDefault();
    if (!blocked) save();
  }}
>
  <span class="name">{workspace}</span>
  {#each names as _, index (index)}
    <div class="field">
      <input
        bind:value={names[index]}
        aria-label={words.shopLabel(index + 1)}
        onkeydown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            onCancel();
          }
        }}
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
        onclick={() => remove(index)}><Icon name="close" size={16} /></button
      >
      {#if problems[index] === "twice"}
        <p class="refused">{words.shopTwice}</p>
      {/if}
    </div>
  {/each}
  {#if refused}<p class="refused">{refused}</p>{/if}
  <div class="controls">
    <button type="button" onclick={add}>{words.addShop}</button>
    <span class="grow"></span>
    <button type="button" onclick={onCancel}>{words.cancel}</button>
    <button
      class="primary"
      type="submit"
      title={blocked}
      disabled={saving || blocked !== ""}>{words.saveShops}</button
    >
  </div>
</form>

<style>
  .shops {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .name {
    font-weight: 600;
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

  .controls {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.2rem;
  }

  .grow {
    flex: 1;
  }

  .refused {
    flex-basis: 100%;
    margin: 0;
    color: var(--danger);
    font-size: 0.88rem;
  }
</style>
