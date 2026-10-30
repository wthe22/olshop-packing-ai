<script lang="ts">
  // Sort rules: edit the sort rules of `packing-workflows.toml` without typing the filter
  // language. A tab per rule; the rule shown is one card, its head the rule's switch (offered on
  // New batch or not), **Rename…** and **Delete rule**, its rows the label groups — a code, a
  // name and a filter built from boxes (or typed as text), with the count it takes in the open
  // batch. *Save* writes the file, keeping its comments and every filter that was not changed;
  // the open batch's plan then follows the file.
  import { onMount } from "svelte";
  import { confirmBox } from "./confirm.svelte";
  import * as api from "./api/rules";
  import { openingRule } from "./sortRules";
  import { texts } from "../texts";
  import GroupRow from "./GroupRow.svelte";
  import Icon from "./Icon.svelte";
  import RulesFileText from "./RulesFileText.svelte";
  import RuleBar from "./RuleBar.svelte";
  import RuleHead from "./RuleHead.svelte";
  import { boxesToFilter, type GroupEntry } from "./groupEntry";
  import { fromRustTree, newTree } from "./conditions";
  import { unconfigured } from "./unconfigured";

  let {
    draftBatch,
    onClose,
    onSaved,
  }: {
    // The batch being prepared, for the count each group takes; `null` when no draft is open.
    draftBatch: number | null;
    onClose: () => void;
    // The file was written: the open batch's plan follows it.
    onSaved: () => void;
  } = $props();

  let entries = $state<GroupEntry[]>([]);
  let rules = $state<api.RuleSummary[]>([]);
  let lastUsed = $state<string | null>(null);
  let current = $state("");
  let couriers = $state<api.Courier[]>([]);
  let generalError = $state("");
  let loadError = $state("");
  let busy = $state(false);
  // The groups as last loaded or saved, to tell an unsaved change; none before the first load.
  let saved = $state("[]");

  const noGroups = $derived(unconfigured(entries));
  const dirty = $derived(JSON.stringify(payload()) !== saved);

  // *Edit file as text*.
  let fileMode = $state(false);
  let fileText = $state("");
  let fileError = $state("");

  const canSave = $derived(
    entries.length > 0 && generalError === "" && entries.every((entry) => entry.error === ""),
  );

  onMount(() => load(null));

  // The rules of the file with their switches, the rule used last, the file's text and the
  // couriers; the open draft's rule and the preselected one for the first rule to show.
  async function loadList() {
    const [view, preselect] = await Promise.all([api.rulesList(), api.sortRules()]);
    rules = view.rules;
    lastUsed = preselect.last_used;
    couriers = view.couriers;
    fileText = view.text;
    return { view, preselect };
  }

  // The rules, then the rule to show: `rule`, else the open batch's rule, else the one the next
  // batch preselects.
  async function load(rule: string | null) {
    busy = true;
    loadError = "";
    fileMode = false;
    try {
      const { view, preselect } = await loadList();
      const names = view.rules.map((summary) => summary.name);
      current = rule ?? openingRule(names, view.draft_rule, preselect.preselected);
      await loadRule();
    } catch (err) {
      loadError = String(err);
    }
    busy = false;
  }

  async function loadRule() {
    const groups = await api.rulesLoad(current);
    const list: GroupEntry[] = [];
    for (const group of groups) {
      const entry: GroupEntry = {
        code: group.code,
        name: group.name,
        filter: group.filter,
        origin: group.origin,
        mode: "boxes",
        tree: newTree(),
        text: group.filter ?? "",
        error: "",
        boxesError: "",
        count: null,
      };
      if (group.filter !== null) {
        try {
          entry.tree = fromRustTree(await api.filterToTree(group.filter));
        } catch (err) {
          entry.mode = "text";
          entry.error = String(err);
        }
      }
      list.push(entry);
    }
    entries = list;
    saved = JSON.stringify(payload());
    await refresh();
  }

  function payload(): api.GroupInput[] {
    return entries.map((entry) => ({
      code: entry.code,
      name: entry.name,
      filter: entry.filter,
      origin: entry.origin,
    }));
  }

  // Recount and re-check every group.
  async function refresh() {
    if (entries.length === 0) {
      generalError = "";
      return;
    }
    try {
      const check = await api.rulesCheck(current, payload());
      generalError = check.general ?? "";
      entries.forEach((entry, index) => {
        entry.error = entry.boxesError || (check.errors[index] ?? "");
        entry.count = check.counts ? (check.counts[index] ?? 0) : null;
      });
    } catch (err) {
      generalError = String(err);
    }
  }

  // Leaving the rule shown loses its unsaved changes: asked first.
  async function mayLeave(): Promise<boolean> {
    return !dirty || (await confirmBox.ask(texts.sortRules.askDiscard(current)));
  }

  async function select(rule: string) {
    if (await mayLeave()) await load(rule);
  }

  async function createRule(name: string, copy: boolean): Promise<boolean> {
    if (!(await mayLeave())) return false;
    await api.ruleNew(name, copy ? current : null);
    onSaved();
    await load(name);
    return true;
  }

  // The switch and the rename change no group, so the groups being edited stay as they are.
  async function switchRule(enabled: boolean) {
    await api.ruleEnable(current, enabled);
    await loadList();
  }

  async function renameRule(name: string) {
    await api.ruleRename(current, name);
    onSaved();
    current = name;
    await loadList();
  }

  async function deleteRule() {
    if (!(await confirmBox.ask(texts.sortRules.askDeleteRule(current)))) return;
    await api.ruleDelete(current);
    onSaved();
    await load(rules.find((rule) => rule.name !== current)?.name ?? null);
  }

  function addGroup() {
    const at = Math.max(0, entries.length - 1); // above the last group, which takes the rest
    const entry: GroupEntry = {
      code: "X",
      name: "",
      filter: "",
      origin: null,
      mode: "boxes",
      tree: newTree(),
      text: "",
      error: "",
      boxesError: "",
      count: null,
    };
    entries.splice(at, 0, entry);
    onBoxes(entry); // an empty box is an error ("This group is empty") until a row is added
  }

  async function onBoxes(entry: GroupEntry) {
    if (await boxesToFilter(entry)) await refresh();
  }

  async function removeGroup(index: number) {
    if (!(await confirmBox.ask(texts.sortRules.askDelete(entries[index].code)))) return;
    entries.splice(index, 1);
    refresh();
  }

  function move(index: number, delta: number) {
    const to = index + delta;
    if (to < 0 || to >= entries.length) return;
    const [entry] = entries.splice(index, 1);
    entries.splice(to, 0, entry);
    refresh();
  }

  // The last group with no filter must stay last.
  function canMoveUp(index: number): boolean {
    if (index === 0) return false;
    return !(index === entries.length - 1 && entries[index].filter === null);
  }

  function canMoveDown(index: number): boolean {
    if (index >= entries.length - 1) return false;
    const last = entries[entries.length - 1];
    return !(index === entries.length - 2 && last.filter === null);
  }

  async function save() {
    busy = true;
    try {
      await api.rulesSave(current, payload());
      onSaved();
      await load(current);
    } catch (err) {
      generalError = String(err);
    }
    busy = false;
  }

  async function checkFile() {
    try {
      await api.rulesCheckText(fileText);
      fileError = "";
    } catch (err) {
      fileError = String(err);
    }
  }

  async function saveFile() {
    busy = true;
    try {
      await api.rulesSaveText(fileText);
      onSaved();
      await load(current);
    } catch (err) {
      fileError = String(err);
    }
    busy = false;
  }
</script>

<div class="rules">
  <RuleBar {rules} {lastUsed} {current} {busy} onSelect={select} onCreate={createRule} />

  {#if loadError}
    <p class="error">{loadError}</p>
  {/if}

  {#if fileMode}
    <RulesFileText
      text={fileText}
      error={fileError}
      onText={(value) => {
        fileText = value;
        checkFile();
      }}
    />
  {:else}
    <section class="card">
      <RuleHead
        {rules}
        {current}
        {busy}
        onSwitch={switchRule}
        onRename={renameRule}
        onDelete={deleteRule}
      />

      <div class="columns">
        <span class="column">{texts.sortRules.groups}</span>
        <span class="hint">{texts.sortRules.hint}</span>
        <span class="grow"></span>
        {#if draftBatch !== null}
          <span class="column count">{texts.sortRules.inBatchHead(draftBatch)}</span>
          <span class="tools"></span>
        {/if}
      </div>

      {#if noGroups}
        <p class="note">{texts.noGroups}</p>
      {/if}
      {#if generalError}
        <p class="error">{generalError}</p>
      {/if}

      {#each entries as entry, index (entry)}
        <GroupRow
          {entry}
          {draftBatch}
          {couriers}
          canUp={canMoveUp(index)}
          canDown={canMoveDown(index)}
          last={index === entries.length - 1}
          onChange={refresh}
          onMove={(delta) => move(index, delta)}
          onRemove={() => removeGroup(index)}
        />
      {/each}

      <div class="card-foot">
        <button class="ghost" onclick={addGroup}
          ><Icon name="plus" size={16} />{texts.sortRules.addGroup}</button
        >
      </div>
    </section>
  {/if}

  <span class="spacer"></span>
  <footer class="foot">
    {#if fileMode}
      <span class="grow"></span>
      <button onclick={() => (fileMode = false)}>{texts.sortRules.cancel}</button>
      <button class="primary" disabled={!!fileError || busy} onclick={saveFile}
        >{texts.sortRules.save}</button
      >
    {:else}
      <button class="ghost" onclick={() => ((fileMode = true), (fileError = ""))}
        >{texts.sortRules.editFile}</button
      >
      <span class="grow"></span>
      {#if dirty}
        <span class="unsaved">{texts.sortRules.unsaved}</span>
      {/if}
      <button onclick={onClose}>{texts.sortRules.cancel}</button>
      <button class="primary" disabled={!canSave || !dirty || busy} onclick={save}
        >{texts.sortRules.save}</button
      >
    {/if}
  </footer>
</div>

<style>
  /* The foot stays at the bottom of the screen, and in view while the groups scroll. */
  .rules {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    --count-width: 6.5rem;
    --tools-width: 6.2rem;
  }

  .card {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
    overflow: hidden;
  }

  /* As the Plan screen's table head: the words in small capitals. */
  .columns {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.45rem 1rem;
  }

  .column {
    color: var(--text-muted);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .column.count {
    flex: none;
    width: var(--count-width);
    text-align: right;
  }

  .columns .tools {
    flex: none;
    width: var(--tools-width);
  }

  .hint {
    margin-left: 0.25rem;
    color: var(--text-faint);
    font-size: 0.88rem;
  }

  .grow {
    flex: 1;
  }

  .note {
    margin: 0;
    padding: 0.6rem 1rem;
    border-top: 1px solid var(--border-soft);
    color: var(--text-muted);
    font-weight: 600;
  }

  .card .error {
    margin: 0.6rem 1rem;
  }

  .card-foot {
    padding: 0.4rem 0.6rem;
    border-top: 1px solid var(--border-soft);
    background: var(--surface-2);
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
