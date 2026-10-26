<script lang="ts">
  // The rule row of the Sort rules screen: the chooser, **+ New rule** (empty, or a copy of the
  // rule shown), **Rename…** and **Delete rule**. The screen does the writing and the asking; a
  // refusal of its call is shown under the row.
  import { ruleNameProblem, type RuleNameProblem } from "./sortRules";
  import { texts } from "../texts";

  let {
    rules,
    current,
    busy,
    onSelect,
    onCreate,
    onRename,
    onDelete,
  }: {
    rules: string[];
    current: string;
    busy: boolean;
    onSelect: (name: string) => void;
    onCreate: (name: string, copy: boolean) => Promise<void>;
    onRename: (name: string) => Promise<void>;
    onDelete: () => Promise<void>;
  } = $props();

  // The name form: for **+ New rule** or **Rename…**.
  let form = $state<"new" | "rename" | null>(null);
  let name = $state("");
  let copy = $state(false);
  let error = $state("");

  function open(kind: "new" | "rename") {
    form = kind;
    name = kind === "rename" ? current : "";
    copy = false;
    error = "";
  }

  function problemText(problem: RuleNameProblem): string {
    return problem === "empty" ? texts.sortRules.ruleNeedsName : texts.sortRules.ruleNameTaken;
  }

  async function submit() {
    error = "";
    const others = form === "rename" ? rules.filter((rule) => rule !== current) : rules;
    const problem = ruleNameProblem(name, others);
    if (problem !== null) {
      error = problemText(problem);
      return;
    }
    try {
      if (form === "new") await onCreate(name.trim(), copy);
      else await onRename(name.trim());
      form = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function remove() {
    error = "";
    try {
      await onDelete();
    } catch (err) {
      error = String(err);
    }
  }
</script>

<div class="bar">
  <label>
    {texts.sortRules.rule}
    <select
      value={current}
      disabled={busy}
      onchange={(event) => {
        form = null;
        error = "";
        onSelect(event.currentTarget.value);
      }}
    >
      {#each rules as rule (rule)}
        <option value={rule}>{rule}</option>
      {/each}
    </select>
  </label>
  <button disabled={busy} onclick={() => open("new")}>{texts.sortRules.newRule}</button>
  <button disabled={busy} onclick={() => open("rename")}>{texts.sortRules.rename}</button>
  <button disabled={busy || rules.length < 2} onclick={remove}>{texts.sortRules.deleteRule}</button>
</div>

{#if form}
  <div class="form">
    <input
      bind:value={name}
      placeholder={texts.sortRules.ruleName}
      onkeydown={(event) => event.key === "Enter" && submit()}
    />
    {#if form === "new"}
      <label>
        <input type="checkbox" bind:checked={copy} />
        {texts.sortRules.copyOf(current)}
      </label>
    {/if}
    <button disabled={busy} onclick={submit}
      >{form === "new" ? texts.sortRules.create : texts.sortRules.renameTo}</button
    >
    <button onclick={() => (form = null)}>{texts.sortRules.cancel}</button>
  </div>
{/if}
{#if error}
  <p class="error">{error}</p>
{/if}

<style>
  .bar,
  .form {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .bar label,
  .form label {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }
</style>
