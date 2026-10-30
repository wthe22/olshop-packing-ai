<script lang="ts">
  // The rule row of the Sort rules screen: one tab per sort rule — the one the newest batch was
  // sorted by marked *last used*, one that is switched off marked *off* — and **+ New rule** with
  // its name form (empty, or a copy of the rule shown). The screen does the writing and the
  // asking; a refusal of its call is shown under the row.
  import { tick } from "svelte";
  import type { RuleSummary } from "./api/rules";
  import { ruleNameProblem } from "./sortRules";
  import { texts } from "../texts";
  import Icon from "./Icon.svelte";

  let {
    rules,
    lastUsed,
    current,
    busy,
    onSelect,
    onCreate,
  }: {
    rules: RuleSummary[];
    // The rule of the workspace's newest batch; `null` before its first batch.
    lastUsed: string | null;
    current: string;
    busy: boolean;
    onSelect: (name: string) => void;
    // Says whether the rule was made: the owner may keep the shown rule's unsaved changes.
    onCreate: (name: string, copy: boolean) => Promise<boolean>;
  } = $props();

  let creating = $state(false);
  let name = $state("");
  let copy = $state(false);
  let error = $state("");
  let nameInput = $state<HTMLInputElement>();

  async function open() {
    creating = true;
    name = "";
    copy = false;
    error = "";
    await tick();
    nameInput?.focus();
  }

  async function submit() {
    const problem = ruleNameProblem(name, rules.map((rule) => rule.name));
    if (problem !== null) {
      error = problem === "empty" ? texts.sortRules.ruleNeedsName : texts.sortRules.ruleNameTaken;
      return;
    }
    error = "";
    try {
      if (await onCreate(name.trim(), copy)) creating = false;
    } catch (err) {
      error = String(err);
    }
  }
</script>

<div class="bar">
  <div class="tabs" role="tablist" aria-label={texts.sortRules.title}>
    {#each rules as rule (rule.name)}
      <button
        role="tab"
        aria-selected={rule.name === current}
        class:chosen={rule.name === current}
        class:off={!rule.enabled}
        onclick={() => !busy && rule.name !== current && onSelect(rule.name)}
      >
        <span class="name">{rule.name}</span>
        {#if rule.name === lastUsed}<span class="tag used">{texts.sortRules.lastUsed}</span>{/if}
        {#if !rule.enabled}<span class="tag">{texts.sortRules.off}</span>{/if}
      </button>
    {/each}
  </div>
  <button class="ghost" disabled={busy} onclick={open}
    ><Icon name="plus" size={16} />{texts.sortRules.newRule}</button
  >
</div>

{#if creating}
  <div class="form">
    <input
      bind:this={nameInput}
      bind:value={name}
      placeholder={texts.sortRules.ruleName}
      onkeydown={(event) => {
        if (event.key === "Enter") submit();
        if (event.key === "Escape") {
          event.stopPropagation();
          creating = false;
        }
      }}
    />
    <label>
      <input type="checkbox" bind:checked={copy} />
      {texts.sortRules.copyOf(current)}
    </label>
    <button class="primary" disabled={busy} onclick={submit}>{texts.sortRules.create}</button>
    <button onclick={() => (creating = false)}>{texts.sortRules.cancel}</button>
  </div>
{/if}
{#if error}
  <p class="error">{error}</p>
{/if}

<style>
  .bar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  /* The rules side by side, as Settings' theme choice: the one shown is raised. */
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
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
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
    box-shadow: var(--shadow);
    cursor: default;
  }

  .tabs button.chosen .name {
    font-weight: 600;
  }

  .tabs button.off:not(.chosen) .name {
    color: var(--text-faint);
  }

  .tag {
    padding: 0 0.45rem;
    border-radius: 999px;
    background: var(--surface-hover);
    color: var(--text-faint);
    font-size: 0.75rem;
    font-weight: 600;
    line-height: 1.5;
  }

  .tag.used {
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  .form {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .form input:not([type="checkbox"]) {
    width: 16rem;
  }

  .form label {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    color: var(--text-muted);
  }

  .error {
    margin: 0 0 0.75rem;
  }
</style>
