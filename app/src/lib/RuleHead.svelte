<script lang="ts">
  // The head of the shown rule's card: its switch (offered on New batch or not), **Rename…**
  // (the name edited in place) and **Delete rule**. The only rule that is on keeps its switch on
  // and cannot be deleted; the last rule cannot be deleted either. The screen does the writing
  // and the asking; a refusal of its call is shown in the head.
  import { tick } from "svelte";
  import type { RuleSummary } from "./api/rules";
  import { deleteProblem, ruleNameProblem, switchOffProblem, type RuleKept } from "./sortRules";
  import { texts } from "../texts";

  let {
    rules,
    current,
    busy,
    onSwitch,
    onRename,
    onDelete,
  }: {
    rules: RuleSummary[];
    current: string;
    busy: boolean;
    onSwitch: (enabled: boolean) => Promise<void>;
    onRename: (name: string) => Promise<void>;
    onDelete: () => Promise<void>;
  } = $props();

  const on = $derived(rules.find((rule) => rule.name === current)?.enabled ?? true);
  const switchKept = $derived(on ? switchOffProblem(rules, current) : null);
  const deleteKept = $derived(deleteProblem(rules, current));

  let renaming = $state(false);
  let name = $state("");
  let error = $state("");
  let nameInput = $state<HTMLInputElement>();

  const keptText = (kept: RuleKept | null) =>
    kept === "only-on" ? texts.sortRules.keptOnlyOn : kept === "last" ? texts.sortRules.keptLast : "";

  async function startRename() {
    renaming = true;
    name = current;
    error = "";
    await tick();
    nameInput?.select();
  }

  async function rename() {
    const others = rules.map((rule) => rule.name).filter((rule) => rule !== current);
    const problem = ruleNameProblem(name, others);
    if (problem !== null) {
      error = problem === "empty" ? texts.sortRules.ruleNeedsName : texts.sortRules.ruleNameTaken;
      return;
    }
    await run(async () => {
      if (name.trim() !== current) await onRename(name.trim());
      renaming = false;
    });
  }

  async function run(action: () => Promise<void>) {
    error = "";
    try {
      await action();
    } catch (err) {
      error = String(err);
    }
  }
</script>

<div class="head">
  {#if renaming}
    <input
      class="rename"
      aria-label={texts.sortRules.ruleName}
      bind:this={nameInput}
      bind:value={name}
      onkeydown={(event) => {
        if (event.key === "Enter") rename();
        if (event.key === "Escape") {
          event.stopPropagation();
          renaming = false;
        }
      }}
    />
    <button class="primary" disabled={busy} onclick={rename}>{texts.sortRules.renameTo}</button>
    <button onclick={() => (renaming = false)}>{texts.sortRules.cancel}</button>
  {:else}
    <button
      class="switch"
      role="switch"
      aria-checked={on}
      aria-label={texts.sortRules.offer}
      title={keptText(switchKept) || texts.sortRules.offer}
      disabled={busy || switchKept !== null}
      onclick={() => run(() => onSwitch(!on))}><span class="knob"></span></button
    >
    <span class="state">
      <strong>{on ? texts.sortRules.on : texts.sortRules.offWord}</strong>
      {on ? texts.sortRules.offered : texts.sortRules.notOffered}
    </span>
    <span class="grow"></span>
    <button disabled={busy} onclick={startRename}>{texts.sortRules.rename}</button>
    <button
      disabled={busy || deleteKept !== null}
      title={keptText(deleteKept)}
      onclick={() => run(onDelete)}>{texts.sortRules.deleteRule}</button
    >
  {/if}
</div>
{#if error}
  <p class="error">{error}</p>
{/if}

<style>
  .head {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.6rem;
    padding: 0.55rem 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
  }

  .state {
    color: var(--text-muted);
  }

  .state strong {
    margin-right: 0.3rem;
    color: var(--text);
  }

  .grow {
    flex: 1;
  }

  .rename {
    width: 16rem;
  }

  /* A switch: a pill with a knob, the accent when on. */
  .switch,
  .switch:hover:not(:disabled) {
    position: relative;
    flex: none;
    width: 2.4rem;
    height: 1.35rem;
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: 999px;
    background: var(--border-strong);
  }

  .switch[aria-checked="true"],
  .switch[aria-checked="true"]:hover:not(:disabled) {
    border-color: var(--accent);
    background: var(--accent);
  }

  .switch:hover:not(:disabled) {
    filter: brightness(1.08);
  }

  .switch:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .knob {
    position: absolute;
    top: 50%;
    left: 0.15rem;
    width: 1rem;
    height: 1rem;
    border-radius: 50%;
    background: #ffffff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    transform: translateY(-50%);
    transition: left 0.15s;
  }

  .switch[aria-checked="true"] .knob {
    left: calc(100% - 1.15rem);
  }

  .error {
    margin: 0.5rem 1rem;
  }
</style>
