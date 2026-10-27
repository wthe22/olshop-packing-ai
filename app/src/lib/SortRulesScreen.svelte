<script lang="ts">
  // Sort rules: edit the sort rules of `packing-workflows.toml` without typing the filter
  // language. Each label group is a code, a name and a filter built from boxes (or typed in a text
  // view), with the count it takes in the open batch. *Save* writes the file, keeping its comments
  // and every filter that was not changed; the open batch's plan then follows the file.
  import { onMount } from "svelte";
  import { confirmBox } from "./confirm.svelte";
  import * as api from "./api/rules";
  import { openingRule } from "./sortRules";
  import { texts } from "../texts";
  import GroupRow from "./GroupRow.svelte";
  import RulesFileText from "./RulesFileText.svelte";
  import RuleBar from "./RuleBar.svelte";
  import { boxesToFilter, type GroupEntry } from "./groupEntry";
  import { fromRustTree, newTree } from "./conditions";
  import { unconfigured } from "./unconfigured";

  let {
    draftBatch,
    onClose,
    onSaved,
  }: {
    // The batch being prepared, for *In batch `<b>`: n*; `null` when no draft is open.
    draftBatch: number | null;
    onClose: () => void;
    // The file was written: the open batch's plan follows it.
    onSaved: () => void;
  } = $props();

  let entries = $state<GroupEntry[]>([]);
  let rules = $state<string[]>([]);
  let current = $state("");
  let couriers = $state<api.Courier[]>([]);
  let generalError = $state("");
  let loadError = $state("");
  let busy = $state(false);
  let showHelp = $state(false);

  const noGroups = $derived(unconfigured(entries));

  // *Edit file as text*.
  let fileMode = $state(false);
  let fileText = $state("");
  let fileError = $state("");

  const canSave = $derived(
    entries.length > 0 && generalError === "" && entries.every((entry) => entry.error === ""),
  );

  onMount(() => load(null));

  // The rules of the file, then the rule to show: `rule`, else the open batch's rule, else the
  // one the next batch preselects.
  async function load(rule: string | null) {
    busy = true;
    loadError = "";
    fileMode = false;
    try {
      const [view, preselect] = await Promise.all([api.rulesList(), api.sortRules()]);
      rules = view.rules;
      couriers = view.couriers;
      fileText = view.text;
      current = rule ?? openingRule(view.rules, view.draft_rule, preselect.preselected);
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

  // A write of the rule row: the file changed, so the open batch's plan follows it.
  async function written(rule: string) {
    onSaved();
    await load(rule);
  }

  async function createRule(name: string, copy: boolean) {
    await api.ruleNew(name, copy ? current : null);
    await written(name);
  }

  async function renameRule(name: string) {
    await api.ruleRename(current, name);
    await written(name);
  }

  async function deleteRule() {
    if (!(await confirmBox.ask(texts.sortRules.askDeleteRule(current)))) return;
    await api.ruleDelete(current);
    await written(rules.find((rule) => rule !== current) ?? "");
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
  <RuleBar
    {rules}
    {current}
    {busy}
    onSelect={(rule) => load(rule)}
    onCreate={createRule}
    onRename={renameRule}
    onDelete={deleteRule}
  />

  <p class="hint">{texts.sortRules.hint}</p>

  {#if loadError}
    <p class="error">{loadError}</p>
  {/if}

  {#if fileMode}
    <RulesFileText
      text={fileText}
      error={fileError}
      {busy}
      onText={(value) => {
        fileText = value;
        checkFile();
      }}
      onCancel={() => (fileMode = false)}
      onSave={saveFile}
    />
  {:else}
    {#if noGroups}
      <p class="no-groups">{texts.noGroups}</p>
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
        onHelp={() => (showHelp = !showHelp)}
      />
    {/each}

    <div class="actions">
      <button onclick={addGroup}>{texts.sortRules.addGroup}</button>
      <span class="grow"></span>
      <button onclick={() => ((fileMode = true), (fileError = ""))}>{texts.sortRules.editFile}</button>
      <button onclick={onClose}>{texts.sortRules.cancel}</button>
      <button class="primary" disabled={!canSave || busy} onclick={save}>{texts.sortRules.save}</button>
    </div>

    {#if showHelp}
      <div class="help">
        <strong>{texts.sortRules.helpTitle}</strong>
        <ul>
          {#each texts.sortRules.examples as example}
            <li><code>{example}</code></li>
          {/each}
        </ul>
      </div>
    {/if}
  {/if}
</div>

<style>
  .rules {
    max-width: 60rem;
  }

  .hint {
    margin: 0 0 0.75rem;
    color: var(--text-muted);
  }

  .no-groups {
    margin: 0 0 0.75rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border);
  }

  .grow {
    flex: 1;
  }

  .help {
    margin-top: 0.75rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface-2);
  }

  .help ul {
    margin: 0.35rem 0 0;
    padding-left: 1.2rem;
  }

  .help code {
    font-family: var(--font-mono);
  }
</style>
