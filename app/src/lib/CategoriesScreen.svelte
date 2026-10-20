<script lang="ts">
  // Categories: edit `categories.toml` without typing the condition language.
  // Each category is a code, a name and a condition built from boxes (or typed in a text view),
  // with the count it takes in the open batch. *Save* writes the file through `RulesDocument`,
  // keeping comments and unchanged condition texts.
  import { onMount } from "svelte";
  // The plugin's `confirm` (not `window.confirm`, which the dialog plugin patches to a command that
  // no longer exists) and it must be awaited.
  import { confirm } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api/categories";
  import { categoriesGrouping, groupingFile } from "../lib/groupings";
  import { texts } from "../texts";
  import CategoryRow from "./CategoryRow.svelte";
  import CategoryText from "./CategoryText.svelte";
  import GroupingBar from "./GroupingBar.svelte";
  import { boxesToCondition, type CatEntry } from "./categoryEntry";
  import { fromRustTree, newTree } from "./conditions";
  import { unconfigured } from "./unconfigured";

  let {
    draftBatch,
    openGrouping,
    onClose,
    onSaved,
  }: {
    // The batch number being prepared, for *In batch `<b>`: n*; `null` when no draft is open.
    draftBatch: number | null;
    // The set to open on: the batch's own set when the screen is opened
    // from the Plan screen's *Edit categories*, so its entries and its counts *In batch `<b>`*
    // are that set's; `null` from the top bar, where the set used last is shown.
    openGrouping: string | null;
    onClose: () => void;
    onSaved: () => void;
  } = $props();

  let entries = $state<CatEntry[]>([]);
  let generalError = $state("");
  let loadError = $state("");
  let busy = $state(false);
  let showHelp = $state(false);

  // The set row: the sets of the data folder, the set being edited (and
  // its file), and the **+ New set** form. A batch is packed with one set, chosen on *New batch*.
  //
  let groupings = $state<api.GroupingView[]>([]);
  let currentGrouping = $state(api.DEFAULT_GROUPING);

  const currentFile = $derived(groupingFile(groupings, currentGrouping));
  const canDeleteGrouping = $derived(currentGrouping !== api.DEFAULT_GROUPING);

  // The one catch-all entry a folder without `categories.toml` starts with, and nothing
  // else, is the unconfigured state; the line says what that means for a batch.
  const noCategories = $derived(unconfigured(entries));

  // *Edit file as text*.
  let fileMode = $state(false);
  let fileText = $state("");
  let fileError = $state("");

  const canSave = $derived(
    entries.length > 0 && generalError === "" && entries.every((entry) => entry.error === ""),
  );

  onMount(load);

  // Open the screen: the sets of the data folder, then the set to open on — the batch's own set
  // when the screen came from the Plan screen's *Edit categories*, else the set used last.
  //
  async function load() {
    busy = true;
    try {
      const list = await api.groupingsList();
      groupings = list.groupings;
      currentGrouping = categoriesGrouping(openGrouping, list.groupings, list.last_grouping);
      await loadGrouping();
    } catch (err) {
      loadError = String(err);
    }
    busy = false;
  }

  // Load the set being shown: its file's entries, or the one catch-all entry when it has no file,
  // then the live check and counts.
  async function loadGrouping() {
    const data = await api.rulesLoad(currentGrouping);
    fileText = data.text;
    loadError = data.error ?? "";
    const list: CatEntry[] = [];
    for (let index = 0; index < data.entries.length; index += 1) {
      const view = data.entries[index];
      const entry: CatEntry = {
        code: view.code,
        name: view.name,
        when: view.when,
        // `null` for the one catch-all entry of a set that has no file: *Save* adds it, so the
        // file is written.
        origin: view.origin,
        mode: "boxes",
        tree: newTree(),
        text: view.when ?? "",
        error: "",
        boxesError: "",
        count: null,
      };
      if (view.when !== null) {
        try {
          entry.tree = fromRustTree(await api.conditionToTree(view.when));
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

  // The set chooser: show another set.
  async function changeGrouping(name: string) {
    if (name === currentGrouping) return;
    currentGrouping = name;
    loadError = "";
    fileMode = false;
    try {
      await loadGrouping();
    } catch (err) {
      loadError = String(err);
    }
  }

  // **+ New set**: **Create** writes `categories/<name>.toml` (the set's
  // file) and shows it. The caller shows the refusal.
  async function createGrouping(name: string) {
    busy = true;
    try {
      await api.newGrouping(name, null);
      const list = await api.groupingsList();
      groupings = list.groupings;
      currentGrouping = name;
      loadError = "";
      fileMode = false;
      await loadGrouping();
    } finally {
      busy = false;
    }
  }

  // **Delete set**: asks first, then deletes that set's file, whose
  // categories fall back to the one catch-all entry. The set `default` is the data folder's own
  // `categories.toml` and cannot be deleted.
  async function removeGrouping() {
    if (!canDeleteGrouping) return;
    if (!(await confirm(texts.categories.confirmDeleteGrouping(currentGrouping)))) return;
    busy = true;
    try {
      await api.deleteGrouping(currentGrouping);
      const list = await api.groupingsList();
      groupings = list.groupings;
      currentGrouping = api.DEFAULT_GROUPING;
      loadError = "";
      fileMode = false;
      await loadGrouping();
    } finally {
      busy = false;
    }
  }

  function payload(): api.EntryInput[] {
    return entries.map((entry) => ({
      code: entry.code,
      name: entry.name,
      when: entry.when,
      origin: entry.origin,
    }));
  }

  // Recount and re-check every entry.
  async function refresh() {
    if (entries.length === 0) {
      generalError = "";
      return;
    }
    try {
      const check = await api.rulesCheck(payload(), currentGrouping);
      generalError = check.general ?? "";
      entries.forEach((entry, index) => {
        entry.error = entry.boxesError || (check.errors[index] ?? "");
        entry.count = check.counts ? (check.counts[index] ?? 0) : null;
      });
    } catch (err) {
      generalError = String(err);
    }
  }

  function addCategory() {
    const at = Math.max(0, entries.length - 1); // above the last (rest) entry
    const entry: CatEntry = {
      code: "X",
      name: "",
      when: "",
      origin: null,
      mode: "boxes",
      tree: newTree(),
      text: "",
      error: "",
      boxesError: "",
      count: null,
    };
    entries.splice(at, 0, entry);
    onBoxes(entry); // an empty group is an error ("This group is empty") until a row is added
  }

  async function onBoxes(entry: CatEntry) {
    if (await boxesToCondition(entry)) await refresh();
  }

  async function removeCategory(index: number) {
    if (!(await confirm(texts.categories.confirmDelete(entries[index].code)))) return;
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

  // The last entry with no condition must stay last.
  function canMoveUp(index: number): boolean {
    if (index === 0) return false;
    return !(index === entries.length - 1 && entries[index].when === null);
  }

  function canMoveDown(index: number): boolean {
    if (index >= entries.length - 1) return false;
    const last = entries[entries.length - 1];
    return !(index === entries.length - 2 && last.when === null);
  }

  async function save() {
    busy = true;
    try {
      await api.rulesSave(payload(), currentGrouping);
      onSaved();
    } catch (err) {
      generalError = String(err);
    }
    busy = false;
  }

  function openFileMode() {
    fileMode = true;
    fileError = "";
  }

  async function checkFile() {
    try {
      await api.rulesCheckText(fileText, currentGrouping);
      fileError = "";
    } catch (err) {
      fileError = String(err);
    }
  }

  async function saveFile() {
    busy = true;
    try {
      await api.rulesSaveText(fileText, currentGrouping);
      fileMode = false;
      onSaved();
    } catch (err) {
      fileError = String(err);
    }
    busy = false;
  }
</script>

<div class="categories">
  <GroupingBar
    {groupings}
    current={currentGrouping}
    file={currentFile}
    canDelete={canDeleteGrouping}
    {busy}
    onSelect={changeGrouping}
    onCreate={createGrouping}
    onDelete={removeGrouping}
  />

  <p class="hint">{texts.categories.hint}</p>

  {#if loadError}
    <p class="error">{loadError}</p>
  {/if}

  {#if fileMode}
    <CategoryText
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
    {#if noCategories}
      <p class="no-categories">{texts.noCategories}</p>
    {/if}

    {#if generalError}
      <p class="error">{generalError}</p>
    {/if}

    {#each entries as entry, index (entry)}
      <CategoryRow
        {entry}
        {draftBatch}
        canUp={canMoveUp(index)}
        canDown={canMoveDown(index)}
        last={index === entries.length - 1}
        onChange={refresh}
        onMove={(delta) => move(index, delta)}
        onRemove={() => removeCategory(index)}
        onHelp={() => (showHelp = !showHelp)}
      />
    {/each}

    <div class="actions">
      <button onclick={addCategory}>{texts.categories.addCategory}</button>
      <span class="grow"></span>
      <button onclick={openFileMode}>{texts.categories.editFile}</button>
      <button onclick={onClose}>{texts.categories.cancel}</button>
      <button class="primary" disabled={!canSave || busy} onclick={save}
        >{texts.categories.save}</button
      >
    </div>

    {#if showHelp}
      <div class="help">
        <strong>{texts.categories.helpTitle}</strong>
        <ul>
          {#each texts.categories.examples as example}
            <li><code>{example}</code></li>
          {/each}
        </ul>
      </div>
    {/if}
  {/if}
</div>

<style>
  .categories {
    max-width: 60rem;
  }

  .hint {
    margin: 0 0 0.75rem;
    color: #4a4a4a;
  }

  /*: the picks are unconfigured (the one catch-all entry, which takes every order). */
  .no-categories {
    margin: 0 0 0.75rem;
    color: #4a4a4a;
    font-weight: 600;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid #d5d5d5;
  }

  .grow {
    flex: 1;
  }

  .help {
    margin-top: 0.75rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid #d5d5d5;
    border-radius: 6px;
    background: #fafafa;
  }

  .help ul {
    margin: 0.35rem 0 0;
    padding-left: 1.2rem;
  }

  .help code {
    font-family: Consolas, monospace;
  }

  @media (prefers-color-scheme: dark) {
    .hint {
      color: #b0b0b0;
    }

    .actions {
      border-color: #444444;
    }

    .help {
      background: #2b2b2b;
      border-color: #444444;
    }
  }
</style>
