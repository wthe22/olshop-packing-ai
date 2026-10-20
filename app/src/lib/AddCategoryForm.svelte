<script lang="ts">
  // *+ Add a pick for this batch* (08 › *3. Plan*): the condition is built in the boxes editor
  // (the same as Categories) with a text view toggle.
  import { onMount } from "svelte";
  import * as api from "./api";
  import { texts } from "../texts";
  import BoxesEditor from "./BoxesEditor.svelte";
  import { fromRustTree, newTree, toRustTree, type UiTree } from "./conditions";

  let {
    onAdd,
    onClose,
  }: {
    /// *Add*: the code, name and condition of the new category; `true` when the plan took it and
    /// this form can close.
    onAdd: (code: string, name: string, condition: string) => Promise<boolean>;
    /// *Cancel*, and after a successful *Add*.
    onClose: () => void;
  } = $props();

  let code = $state("X");
  let name = $state("");
  let condition = $state("");
  let error = $state("");
  let mode = $state<"boxes" | "text">("boxes");
  let tree = $state<UiTree>(newTree());

  // The form opens on the boxes editor: its condition is what the empty tree reads as.
  onMount(treeChanged);

  /// The boxes editor changed: turn the tree into the condition text (the printer's form).
  async function treeChanged() {
    try {
      condition = await api.treeToCondition(toRustTree(tree));
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function checkCondition() {
    if (condition.trim() === "") {
      error = "";
      return;
    }
    try {
      await api.checkCondition(condition);
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function toBoxes() {
    try {
      tree = fromRustTree(await api.conditionToTree(condition));
      mode = "boxes";
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function submit() {
    error = "";
    if (mode === "boxes") {
      await treeChanged();
      if (error) return;
    } else {
      try {
        await api.checkCondition(condition);
      } catch (e) {
        error = String(e);
        return;
      }
    }
    if (await onAdd(code, name, condition)) onClose();
  }
</script>

<div class="add-form">
  <div class="add-top">
    <label>
      {texts.plan.addCode}
      <input class="code" bind:value={code} />
    </label>
    <label>
      {texts.plan.addName}
      <input class="name" bind:value={name} />
    </label>
    <span class="grow"></span>
    <button class="primary" onclick={submit}>{texts.plan.addSubmit}</button>
    <button onclick={onClose}>{texts.plan.addCancel}</button>
  </div>
  <div class="add-condition">
    <span class="label">{texts.plan.addCondition}</span>
    {#if mode === "boxes"}
      <BoxesEditor tree={tree} onEdit={treeChanged} />
      <button class="link" onclick={() => (mode = "text")}
        >{texts.categories.textView}</button
      >
    {:else}
      <textarea
        class="condition-input"
        rows="2"
        bind:value={condition}
        oninput={checkCondition}
      ></textarea>
      <button class="link" disabled={!!error} onclick={toBoxes}
        >{texts.categories.boxes}</button
      >
    {/if}
    {#if error}
      <div class="error add-error">{error}</div>
    {/if}
  </div>
</div>

<style>
  .add-form {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding: 0.75rem;
    margin-bottom: 0.75rem;
    border: 1px solid #d5d5d5;
    border-radius: 6px;
    background: #ffffff;
  }

  .add-top {
    display: flex;
    align-items: flex-end;
    gap: 0.6rem;
  }

  .add-form label {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.85rem;
  }

  .add-top .grow {
    flex: 1;
  }

  .add-form .code {
    width: 4rem;
  }

  .add-form .name {
    width: 12rem;
  }

  .add-condition {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .add-condition .label {
    font-size: 0.85rem;
    color: #4a4a4a;
  }

  .add-form .condition-input {
    width: 100%;
    min-width: 16rem;
    font-family: Consolas, monospace;
  }

  .link {
    align-self: flex-start;
    padding: 0;
    border: none;
    background: none;
    color: #1f6feb;
    cursor: pointer;
  }

  .add-error {
    width: 100%;
    margin: 0;
  }

  @media (prefers-color-scheme: dark) {
    .add-form {
      background: #2b2b2b;
      border-color: #444444;
    }
  }
</style>
