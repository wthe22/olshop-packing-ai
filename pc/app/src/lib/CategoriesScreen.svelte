<script lang="ts">
  // Categories (08 › *5. Categories*): edit `categories.toml` without typing the condition language.
  // Each category is a code, a name and a condition built from boxes (or typed in a text view),
  // with the count it takes in the open batch. *Save* writes the file through `RulesDocument`,
  // keeping comments and unchanged condition texts.
  import { onMount } from "svelte";
  // The plugin's `confirm` (not `window.confirm`, which the dialog plugin patches to a command that
  // no longer exists) and it must be awaited.
  import { confirm } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api";
  import { texts } from "../texts";
  import BoxesEditor from "./BoxesEditor.svelte";
  import { fromRustTree, newTree, toRustTree, type UiTree } from "./conditions";

  let {
    draftBatch,
    onClose,
    onSaved,
  }: {
    /// The batch number being prepared, for *In batch `<b>`: n*; `null` when no draft is open.
    draftBatch: number | null;
    onClose: () => void;
    onSaved: () => void;
  } = $props();

  interface CatEntry {
    code: string;
    name: string;
    /// The condition text; `null` for the last entry with no condition (it takes the rest).
    when: string | null;
    /// The index the entry had in the loaded file, so the save keeps its comment and text.
    origin: number | null;
    mode: "boxes" | "text";
    tree: UiTree;
    text: string;
    error: string;
    /// The boxes editor's own error (e.g. an empty group), which a recount must not clear.
    boxesError: string;
    count: number | null;
  }

  let entries = $state<CatEntry[]>([]);
  let generalError = $state("");
  let loadError = $state("");
  let busy = $state(false);
  let showHelp = $state(false);

  // *Edit file as text* (08 › *5. Categories*).
  let fileMode = $state(false);
  let fileText = $state("");
  let fileError = $state("");

  const canSave = $derived(
    entries.length > 0 && generalError === "" && entries.every((entry) => entry.error === ""),
  );

  onMount(load);

  async function load() {
    busy = true;
    try {
      const data = await api.rulesLoad();
      fileText = data.text;
      loadError = data.error ?? "";
      const list: CatEntry[] = [];
      for (let index = 0; index < data.entries.length; index += 1) {
        const view = data.entries[index];
        const entry: CatEntry = {
          code: view.code,
          name: view.name,
          when: view.when,
          origin: index,
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
    } catch (err) {
      loadError = String(err);
    }
    busy = false;
  }

  function payload(): api.EntryInput[] {
    return entries.map((entry) => ({
      code: entry.code,
      name: entry.name,
      when: entry.when,
      origin: entry.origin,
    }));
  }

  /// Recount and re-check every entry (08 › *5. Categories*: "every change recounts all rows").
  async function refresh() {
    if (entries.length === 0) {
      generalError = "";
      return;
    }
    try {
      const check = await api.rulesCheck(payload());
      generalError = check.general ?? "";
      entries.forEach((entry, index) => {
        entry.error = entry.boxesError || (check.errors[index] ?? "");
        entry.count = check.counts ? (check.counts[index] ?? 0) : null;
      });
    } catch (err) {
      generalError = String(err);
    }
  }

  async function onBoxes(entry: CatEntry) {
    try {
      entry.when = await api.treeToCondition(toRustTree(entry.tree));
      entry.boxesError = "";
    } catch (err) {
      entry.boxesError = String(err);
      entry.error = entry.boxesError;
      return;
    }
    await refresh();
  }

  function onText(entry: CatEntry) {
    entry.when = entry.text;
    entry.boxesError = "";
    refresh();
  }

  async function toText(entry: CatEntry) {
    entry.text = entry.when ?? "";
    entry.mode = "text";
    entry.boxesError = "";
    await refresh();
  }

  async function toBoxes(entry: CatEntry) {
    try {
      entry.tree = fromRustTree(await api.conditionToTree(entry.text));
      entry.when = entry.text;
      entry.mode = "boxes";
      entry.boxesError = "";
      await refresh();
    } catch (err) {
      entry.boxesError = String(err);
      entry.error = entry.boxesError;
    }
  }

  function setRest(entry: CatEntry, checked: boolean) {
    if (checked) {
      entry.when = null;
      entry.boxesError = "";
      entry.error = "";
      refresh();
    } else {
      entry.when = "";
      entry.text = "";
      entry.tree = newTree();
      entry.mode = "boxes";
      entry.boxesError = "";
      onBoxes(entry);
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

  // The last entry with no condition must stay last (04 › *File format*).
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
      await api.rulesSave(payload());
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
      fileMode = false;
      onSaved();
    } catch (err) {
      fileError = String(err);
    }
    busy = false;
  }
</script>

<div class="categories">
  <p class="hint">{texts.categories.hint}</p>

  {#if loadError}
    <p class="error">{loadError}</p>
  {/if}

  {#if fileMode}
    <div class="file">
      <label for="file-text">{texts.categories.fileText}</label>
      <textarea
        id="file-text"
        rows="18"
        bind:value={fileText}
        oninput={() => checkFile()}
      ></textarea>
      {#if fileError}
        <p class="error">{fileError}</p>
      {/if}
      <div class="actions">
        <span class="grow"></span>
        <button onclick={() => (fileMode = false)}>{texts.categories.cancel}</button>
        <button class="primary" disabled={!!fileError || busy} onclick={saveFile}
          >{texts.categories.save}</button
        >
      </div>
    </div>
  {:else}
    {#if generalError}
      <p class="error">{generalError}</p>
    {/if}

    {#each entries as entry, index (entry)}
      <section class="category">
        <div class="cat-head">
          <input
            class="code"
            aria-label={texts.categories.code}
            bind:value={entry.code}
            oninput={() => refresh()}
          />
          <input
            class="name"
            aria-label={texts.categories.name}
            bind:value={entry.name}
            oninput={() => refresh()}
          />
          <span class="grow"></span>
          {#if entry.error}
            <span class="count mark">✗</span>
          {:else if draftBatch !== null && entry.count !== null}
            <span class="count">{texts.categories.inBatch(draftBatch, entry.count)}</span>
          {/if}
          <button
            title={texts.categories.moveUp}
            disabled={!canMoveUp(index)}
            onclick={() => move(index, -1)}>↑</button
          >
          <button
            title={texts.categories.moveDown}
            disabled={!canMoveDown(index)}
            onclick={() => move(index, 1)}>↓</button
          >
          <button title={texts.categories.delete} onclick={() => removeCategory(index)}>🗑</button>
        </div>

        {#if entry.when === null}
          <p class="rest">{texts.categories.takesRest}</p>
        {:else if entry.mode === "boxes"}
          <BoxesEditor tree={entry.tree} onEdit={() => onBoxes(entry)} />
        {:else}
          <textarea
            class="condition-text"
            rows="3"
            bind:value={entry.text}
            oninput={() => onText(entry)}
          ></textarea>
        {/if}

        {#if entry.error}
          <p class="error">{entry.error}</p>
        {/if}

        <div class="cat-foot">
          {#if index === entries.length - 1}
            <label class="rest-toggle">
              <input
                type="checkbox"
                checked={entry.when === null}
                onchange={(event) => setRest(entry, event.currentTarget.checked)}
              />
              {texts.categories.takesRest}
            </label>
          {/if}
          {#if entry.when !== null}
            {#if entry.mode === "boxes"}
              <button onclick={() => toText(entry)}>{texts.categories.textView}</button>
            {:else}
              <button disabled={!!entry.error} onclick={() => toBoxes(entry)}
                >{texts.categories.boxes}</button
              >
              <button class="link" onclick={() => (showHelp = !showHelp)}
                >{texts.categories.helpExamples}</button
              >
            {/if}
          {/if}
        </div>
      </section>
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

  .category {
    border: 1px solid #d5d5d5;
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.75rem;
    background: #ffffff;
  }

  .cat-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .cat-head .code {
    width: 3.5rem;
  }

  .cat-head .name {
    width: 14rem;
  }

  .grow {
    flex: 1;
  }

  .count {
    color: #4a4a4a;
    font-variant-numeric: tabular-nums;
  }

  .count.mark {
    color: #8a1f1f;
    font-weight: 600;
  }

  .rest {
    margin: 0.3rem 0;
    color: #6a6a6a;
    font-style: italic;
  }

  .condition-text {
    width: 100%;
    font-family: Consolas, monospace;
  }

  .cat-foot {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 0.35rem;
  }

  .rest-toggle {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.9rem;
    color: #4a4a4a;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid #d5d5d5;
  }

  .file label {
    display: block;
    margin-bottom: 0.3rem;
    font-weight: 600;
  }

  .file textarea {
    width: 100%;
    font-family: Consolas, monospace;
    font-size: 0.9rem;
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

  .link {
    padding: 0;
    border: none;
    background: none;
    color: #1f6feb;
    cursor: pointer;
  }

  @media (prefers-color-scheme: dark) {
    .category {
      background: #2b2b2b;
      border-color: #444444;
    }

    .hint,
    .count,
    .rest,
    .rest-toggle {
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
