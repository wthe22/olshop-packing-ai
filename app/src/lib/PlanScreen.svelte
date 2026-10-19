<script lang="ts">
  // Plan (08 › *3. Plan*): the picks in order with their counts and runs, the per-batch changes
  // (*Use*, move, add, rename) and the packing-list options; *Save batch* writes the batch. An
  // amend uses the same screen, titled *Redo batch `<b>`*, with *Save batch `<b>` again* and a
  // bottom *Cancel amend*. The top button is *Back* on both and it never asks or discards: it
  // returns to the PDF selection page the batch came from — a new batch's file list, or the
  // amend's selection page — with the files still listed (only that page's *Cancel* and
  // *Cancel amend* discard).
  import * as api from "../lib/api";
  import { listen } from "@tauri-apps/api/event";
  import { texts, saveStageText, planButtons } from "../texts";
  import BoxesEditor from "./BoxesEditor.svelte";
  import SaveCheckDialog from "./SaveCheckDialog.svelte";
  import { fromRustTree, newTree, toRustTree, type UiTree } from "./conditions";
  import { findingsOf, type Findings } from "./saveCheck";

  let {
    batchNumber,
    read,
    settings,
    refresh,
    onSaved,
    onBack,
    onCancel,
    onEditCategories,
  }: {
    batchNumber: number;
    read: api.ReadResult;
    settings: api.Settings | null;
    /// Bumped when the Categories screen saves, so the plan is recounted (08 › *Window frame*).
    refresh: number;
    onSaved: (result: api.SaveResult) => void;
    /// *Back* (top): return to the PDF selection page — a new batch's file list, or the amend's
    /// selection page — with the files still listed and the draft kept (a new *Read labels*
    /// replaces it). Never asks and writes nothing.
    onBack: () => void;
    /// *Cancel amend* (bottom): leave the batch for the Day screen and discard its draft. Never
    /// asks, because an amend writes nothing before *Save batch `<b>` again*.
    onCancel: () => void;
    onEditCategories: () => void;
  } = $props();

  // The batch number for the header and the amend's title: the amended batch for an amend
  // (08 › *3. Plan*: *Redo batch 2*), else the next batch number.
  const batchNo = $derived(read.amend ?? batchNumber);
  const amending = $derived(read.amend !== null);
  // The top button (*Back* on both) and the amend's bottom *Cancel amend*, from the pure helper
  // (08 › *3. Plan*).
  const buttons = $derived(planButtons(read.amend));

  // The read result is fixed for the life of the screen; the view is then updated by `plan()`.
  // svelte-ignore state_referenced_locally
  let view = $state<api.PlanView>(read.plan);
  let showWarnings = $state(false);
  let expanded = $state<string[]>([]);
  let error = $state("");

  // The packing-list choice, starting from the Settings default (08 › *6. Settings*).
  // svelte-ignore state_referenced_locally
  let scope = $state<api.Scope>(settings?.scope ?? "whole");
  // svelte-ignore state_referenced_locally
  let layouts = $state<api.Layout[]>(settings?.layouts ?? ["pick"]);

  let saving = $state(false);
  let saveError = $state("");
  // The check before saving's progress (the `save-progress` event: the stage and a progress bar)
  // and the findings dialog it shows when the save stopped (07 › *Check before saving*).
  let saveProgress = $state<api.SaveProgress | null>(null);
  let findings = $state<Findings | null>(null);

  // *+ Add a pick for this batch* (08 › *3. Plan*): the condition is built in the boxes editor
  // (the same as Categories) with a text view toggle.
  let showAdd = $state(false);
  let addCode = $state("X");
  let addName = $state("");
  let addCondition = $state("");
  let addError = $state("");
  let addMode = $state<"boxes" | "text">("boxes");
  let addTree = $state<UiTree>(newTree());

  // Recount after the Categories screen saves (08 › *Window frame*: the draft is kept and the plan
  // recounted with the new categories).
  // svelte-ignore state_referenced_locally
  let lastRefresh = $state(refresh);
  $effect(() => {
    if (refresh !== lastRefresh) {
      lastRefresh = refresh;
      reload();
    }
  });

  async function reload() {
    try {
      view = await api.plan([]);
    } catch (e) {
      error = String(e);
    }
  }

  const scopes: { value: api.Scope; label: string }[] = [
    { value: "whole", label: texts.plan.scopeWhole },
    { value: "per-pdf", label: texts.plan.scopePerPdf },
    { value: "none", label: texts.plan.scopeNone },
  ];
  const layoutOptions: { value: api.Layout; label: string }[] = [
    { value: "full", label: texts.plan.layoutFull },
    { value: "summary", label: texts.plan.layoutSummary },
    { value: "pick", label: texts.plan.layoutPick },
  ];

  async function apply(edits: api.PlanEdit[]) {
    error = "";
    try {
      view = await api.plan(edits);
    } catch (e) {
      error = String(e);
    }
  }

  function toggleUse(row: api.PlanRow, checked: boolean) {
    if (row.kind === "generated") return;
    apply([{ kind: "set_skipped", index: row.index, skipped: !checked }]);
  }

  function move(row: api.PlanRow, up: boolean) {
    if (row.kind === "generated") return;
    apply([{ kind: up ? "move_up" : "move_down", index: row.index }]);
  }

  function rename(row: api.PlanRow, name: string) {
    if (row.kind === "generated") return;
    apply([{ kind: "rename", index: row.index, name }]);
  }

  function toggleExpanded(key: string) {
    expanded = expanded.includes(key)
      ? expanded.filter((item) => item !== key)
      : [...expanded, key];
  }

  function toggleLayout(layout: api.Layout, checked: boolean) {
    layouts = checked
      ? [...layouts, layout]
      : layouts.filter((item) => item !== layout);
  }

  async function checkAddCondition() {
    if (addCondition.trim() === "") {
      addError = "";
      return;
    }
    try {
      await api.checkCondition(addCondition);
      addError = "";
    } catch (e) {
      addError = String(e);
    }
  }

  /// The boxes editor changed: turn the tree into the condition text (the printer's form).
  async function addTreeChanged() {
    try {
      addCondition = await api.treeToCondition(toRustTree(addTree));
      addError = "";
    } catch (e) {
      addError = String(e);
    }
  }

  async function addToBoxes() {
    try {
      addTree = fromRustTree(await api.conditionToTree(addCondition));
      addMode = "boxes";
      addError = "";
    } catch (e) {
      addError = String(e);
    }
  }

  function toggleAdd() {
    showAdd = !showAdd;
    if (showAdd) {
      addTree = newTree();
      addMode = "boxes";
      addCondition = "";
      addError = "";
      addTreeChanged();
    }
  }

  async function addPick() {
    addError = "";
    if (addMode === "boxes") {
      await addTreeChanged();
      if (addError) return;
    } else {
      try {
        await api.checkCondition(addCondition);
      } catch (e) {
        addError = String(e);
        return;
      }
    }
    await apply([
      {
        kind: "add",
        at: view.add_at,
        code: addCode,
        name: addName,
        condition: addCondition,
      },
    ]);
    if (error === "") {
      showAdd = false;
      addCode = "X";
      addName = "";
      addCondition = "";
    }
  }

  /// *Save batch* (08 › *3. Plan*): write the batch and go to the Day screen. While it runs the
  /// stage and a progress bar are shown (the `save-progress` event). When the check before saving
  /// finds something and this is not the dialog's *Save anyway*, the findings dialog opens and the
  /// draft is kept (07 › *Check before saving*).
  async function save(saveAnyway = false) {
    saving = true;
    saveError = "";
    saveProgress = null;
    findings = null;
    const unlisten = await listen<api.SaveProgress>("save-progress", (event) => {
      saveProgress = event.payload;
    });
    try {
      const result = await api.saveBatch({ scope, layouts }, saveAnyway);
      findings = findingsOf(result);
      if (result.saved) onSaved(result);
    } catch (e) {
      saveError = String(e);
    }
    unlisten();
    saving = false;
    saveProgress = null;
  }

  // The save's stage and progress line: the event's stage in plain words, or *Saving …* before
  // the first `save-progress` arrives (07 › *Check before saving*: *Shown while it runs*).
  const savingText = $derived(
    saveProgress
      ? saveStageText(saveProgress.stage, saveProgress.page, saveProgress.pages)
      : texts.plan.saving,
  );

  /// *Back* (top, 08 › *3. Plan*): return to the PDF selection page with the files still listed —
  /// the draft is kept and a new *Read labels* replaces it, so the per-batch changes are lost.
  /// It is not a discard and it never asks (08 › *2. New batch* → *The amend's selection page*).
  function back() {
    onBack();
  }

  const rowKey = (row: api.PlanRow) => (row.kind === "generated" ? "generated" : String(row.index));
</script>

<main>
  <div class="bar">
    <h2>
      {#if amending}
        {texts.plan.amendTitle(batchNo)}
      {:else}
        {texts.plan.header(
          batchNo,
          read.plan.grouping,
          read.files,
          read.pages,
          read.orders,
          read.already_saved,
        )}
      {/if}
    </h2>
    {#if amending}
      <span class="sub">
        {texts.plan.header(
          batchNo,
          read.plan.grouping,
          read.files,
          read.pages,
          read.orders,
          read.already_saved,
        )}
      </span>
    {/if}
    <span class="grow"></span>
    <button onclick={back}>{buttons.top}</button>
  </div>

  <div class="warnings">
    {#if read.warnings.length === 0}
      <span class="muted">{texts.plan.warningsNone}</span>
    {:else}
      <button class="link" onclick={() => (showWarnings = !showWarnings)}>
        {texts.plan.warnings(read.warnings.length)}
      </button>
      {#if showWarnings}
        <ul>
          {#each read.warnings as warning}
            <li>{warning}</li>
          {/each}
        </ul>
      {/if}
    {/if}
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if view.no_picks}
    <p class="no-picks">{texts.noPicks}</p>
  {/if}

  <table>
    <thead>
      <tr>
        <th class="c-use">{texts.plan.use}</th>
        <th class="c-num">{texts.plan.hash}</th>
        <th class="c-pick">{texts.plan.pick}</th>
        <th class="c-orders">{texts.plan.orders}</th>
        <th class="c-runs">{texts.plan.runs}</th>
        <th class="c-file">{texts.plan.fileName}</th>
        <th class="c-tools"></th>
      </tr>
    </thead>
    <tbody>
      {#each view.rows as row (rowKey(row))}
        <tr class:skipped={row.skipped} class:empty={row.orders === 0 && !row.skipped} class:rest={row.kind === "generated"}>
          <td class="c-use">
            {#if row.kind !== "generated"}
              <input
                type="checkbox"
                checked={!row.skipped}
                onchange={(e) => toggleUse(row, e.currentTarget.checked)}
              />
            {/if}
          </td>
          <td class="c-num">{row.number ?? ""}</td>
          <td class="c-pick">
            <span class="code">{row.code}</span>
            <span class="name">{row.name}</span>
            {#if row.kind !== "entry"}<span class="tag">{texts.plan.rest}</span>{/if}
            {#if row.skipped}<span class="tag">{texts.plan.skipped}</span>{/if}
            {#if row.condition}
              <div class="condition">{row.condition}</div>
            {/if}
          </td>
          <td class="c-orders">{row.orders > 0 ? row.orders : ""}</td>
          <td class="c-runs">{row.orders > 0 ? row.runs.length : ""}</td>
          <td class="c-file">
            {#if row.orders === 0 && !row.skipped}
              <span class="no-pdf">{texts.plan.noPdf}</span>
            {:else}
              <input
                class="filename"
                value={row.file_name}
                onchange={(e) => rename(row, e.currentTarget.value)}
              />
              {#if row.duplicate_of !== null}
                <div class="dup">{texts.plan.duplicate(row.duplicate_of)}</div>
              {/if}
            {/if}
          </td>
          <td class="c-tools">
            {#if row.can_move_up}
              <button title={texts.plan.moveUp} onclick={() => move(row, true)}>↑</button>
            {/if}
            {#if row.can_move_down}
              <button title={texts.plan.moveDown} onclick={() => move(row, false)}>↓</button>
            {/if}
            {#if row.runs.length > 0}
              <button
                title={texts.plan.expand}
                onclick={() => toggleExpanded(rowKey(row))}>▸</button
              >
            {/if}
          </td>
        </tr>
        {#if expanded.includes(rowKey(row))}
          {#each row.runs as run}
            <tr class="run">
              <td class="c-use"></td>
              <td class="c-num"></td>
              <td class="c-pick" colspan="2">{run.label}&nbsp;&nbsp;{run.contents}</td>
              <td class="c-runs run-orders">
                {run.orders} {run.orders === 1 ? "order" : "orders"}
              </td>
              <td class="c-file"></td>
              <td class="c-tools"></td>
            </tr>
          {/each}
        {/if}
      {/each}
    </tbody>
  </table>

  <div class="add-bar">
    <button onclick={toggleAdd}>{texts.plan.addPick}</button>
    <button onclick={onEditCategories}>{texts.plan.editCategories}</button>
  </div>

  {#if showAdd}
    <div class="add-form">
      <div class="add-top">
        <label>
          {texts.plan.addCode}
          <input class="code" bind:value={addCode} />
        </label>
        <label>
          {texts.plan.addName}
          <input class="name" bind:value={addName} />
        </label>
        <span class="grow"></span>
        <button class="primary" onclick={addPick}>{texts.plan.addSubmit}</button>
        <button onclick={() => (showAdd = false)}>{texts.plan.addCancel}</button>
      </div>
      <div class="add-condition">
        <span class="label">{texts.plan.addCondition}</span>
        {#if addMode === "boxes"}
          <BoxesEditor tree={addTree} onEdit={addTreeChanged} />
          <button class="link" onclick={() => (addMode = "text")}
            >{texts.categories.textView}</button
          >
        {:else}
          <textarea
            class="condition-input"
            rows="2"
            bind:value={addCondition}
            oninput={checkAddCondition}
          ></textarea>
          <button class="link" disabled={!!addError} onclick={addToBoxes}
            >{texts.categories.boxes}</button
          >
        {/if}
        {#if addError}
          <div class="error add-error">{addError}</div>
        {/if}
      </div>
    </div>
  {/if}

  {#if saving}
    <div class="saving">
      <p class="muted">{savingText}</p>
      <progress max={saveProgress?.pages ?? 0} value={saveProgress?.page ?? 0}></progress>
    </div>
  {/if}

  <div class="foot">
    <div class="packing">
      <span class="label">{texts.plan.packingList}</span>
      {#each scopes as option (option.value)}
        <label>
          <input
            type="radio"
            name="scope"
            value={option.value}
            checked={scope === option.value}
            onchange={() => (scope = option.value)}
          />
          {option.label}
        </label>
      {/each}
      {#each layoutOptions as option (option.value)}
        <label>
          <input
            type="checkbox"
            checked={layouts.includes(option.value)}
            onchange={(e) => toggleLayout(option.value, e.currentTarget.checked)}
          />
          {option.label}
        </label>
      {/each}
    </div>
    <span class="grow"></span>
    {#if buttons.discard}
      <button onclick={onCancel}>{buttons.discard}</button>
    {/if}
    <button
      class="primary"
      disabled={view.has_duplicate_names || saving}
      onclick={() => save()}>{amending ? texts.plan.saveAgain(batchNo) : texts.plan.save}</button
    >
  </div>

  {#if saveError}
    <p class="error">{saveError}</p>
  {/if}

  {#if findings}
    <SaveCheckDialog
      {findings}
      busy={saving}
      onSaveAnyway={() => save(true)}
      onCancel={() => (findings = null)}
    />
  {/if}
</main>

<style>
  main {
    padding: 1rem 1.5rem;
  }

  .bar {
    display: flex;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  h2 {
    margin: 0;
    font-size: 1.05rem;
  }

  .grow {
    flex: 1;
  }

  .warnings {
    margin-bottom: 0.75rem;
    font-size: 0.95rem;
  }

  .warnings ul {
    margin: 0.35rem 0 0;
    padding-left: 1.2rem;
  }

  /* Q-UI-4: the plan is the one catch-all entry, so every order goes into one PDF set. */
  .no-picks {
    margin: 0 0 0.75rem;
    color: #4a4a4a;
    font-weight: 600;
  }

  .link {
    padding: 0;
    border: none;
    background: none;
    color: #1f6feb;
    cursor: pointer;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 0.75rem;
  }

  th {
    text-align: left;
    font-weight: 600;
    font-size: 0.9rem;
    color: #4a4a4a;
    padding: 0.3rem 0.4rem;
    border-bottom: 1px solid #d5d5d5;
  }

  td {
    padding: 0.35rem 0.4rem;
    border-bottom: 1px solid #ececec;
    vertical-align: top;
  }

  .c-use {
    width: 2.5rem;
  }

  .c-num {
    width: 2.5rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .c-orders,
  .c-runs {
    width: 4.5rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .c-tools {
    width: 7rem;
    white-space: nowrap;
  }

  .c-tools button {
    padding: 0.05rem 0.4rem;
    margin-right: 0.15rem;
  }

  .code {
    display: inline-block;
    min-width: 1.4rem;
    font-weight: 600;
  }

  .tag {
    margin-left: 0.4rem;
    color: #6a6a6a;
    font-size: 0.85rem;
  }

  .condition {
    color: #6a6a6a;
    font-size: 0.85rem;
    white-space: pre-wrap;
  }

  .filename {
    width: 100%;
    min-width: 12rem;
  }

  .dup {
    color: #8a1f1f;
    font-size: 0.85rem;
  }

  .no-pdf {
    color: #8a8a8a;
  }

  tr.skipped td,
  tr.empty td {
    color: #9a9a9a;
  }

  tr.run td {
    border-bottom: none;
    color: #4a4a4a;
    font-family: Consolas, monospace;
    font-size: 0.9rem;
  }

  .run-orders {
    font-family: "Segoe UI", sans-serif;
  }

  .add-bar {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }

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

  .sub {
    margin-left: 0.75rem;
    color: #6a6a6a;
    font-size: 0.9rem;
    font-weight: 400;
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding-top: 0.5rem;
    border-top: 1px solid #d5d5d5;
  }

  /* 08 › *3. Plan*: the check before saving's stage and progress bar. */
  .saving {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem 0 0.25rem;
  }

  .saving progress {
    width: 100%;
    height: 1.1rem;
  }

  .saving .muted {
    margin: 0;
    color: #4a4a4a;
  }

  .packing {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    flex-wrap: wrap;
  }

  .packing label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .packing .label {
    font-weight: 600;
  }

  @media (prefers-color-scheme: dark) {
    .add-form {
      background: #2b2b2b;
      border-color: #444444;
    }

    th {
      color: #b0b0b0;
      border-color: #444444;
    }

    td {
      border-color: #3a3a3a;
    }

    .foot {
      border-color: #444444;
    }

    .condition,
    .tag,
    .muted,
    tr.run td {
      color: #b0b0b0;
    }

    .saving .muted {
      color: #b0b0b0;
    }

    .no-pdf {
      color: #8a8a8a;
    }
  }
</style>
