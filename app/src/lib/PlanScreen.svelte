<script lang="ts">
  // Plan (08 › *3. Plan*): the picks in order with their counts and runs, the per-batch changes
  // (*Use*, move, add, rename) and the packing-list options; *Save batch* writes the batch. An
  // amend uses the same screen, titled *Redo batch `<b>`*, with *Save batch `<b>` again* and a
  // bottom *Cancel amend*. The top button is *Back* on both and it never asks or discards: it
  // returns to the PDF selection page the batch came from — a new batch's file list, or the
  // amend's selection page — with the files still listed (only that page's *Cancel* and
  // *Cancel amend* discard).
  import { untrack } from "svelte";
  import * as api from "../lib/api";
  import { listen } from "@tauri-apps/api/event";
  import { texts, planButtons } from "../texts";
  import AddCategoryForm from "./AddCategoryForm.svelte";
  import PackingListOptions from "./PackingListOptions.svelte";
  import PlanRow from "./PlanRow.svelte";
  import SaveProgress from "./SaveProgress.svelte";
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
  // (`untrack`: the screen starts from it once; it is not a value the read keeps in step.)
  let view = $state<api.PlanView>(untrack(() => read.plan));
  let showWarnings = $state(false);
  let expanded = $state<string[]>([]);
  let error = $state("");

  // The packing-list choice, starting from the Settings default (08 › *6. Settings*).
  let scope = $state<api.Scope>(untrack(() => settings?.scope ?? "whole"));
  let layouts = $state<api.Layout[]>(untrack(() => settings?.layouts ?? ["pick"]));

  let saving = $state(false);
  let saveError = $state("");
  // The check before saving's progress (the `save-progress` event: the stage and a progress bar)
  // and the findings dialog it shows when the save stopped (07 › *Check before saving*).
  let saveProgress = $state<api.SaveProgress | null>(null);
  let findings = $state<Findings | null>(null);

  let showAdd = $state(false);

  // Recount after the Categories screen saves (08 › *Window frame*: the draft is kept and the plan
  // recounted with the new categories).
  let lastRefresh = $state(untrack(() => refresh));
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

  /// *Add* on the add form: the new entry goes at the plan's own add position. `false` leaves the
  /// form open; the error line above the table says why.
  async function addCategory(code: string, name: string, condition: string): Promise<boolean> {
    await apply([
      {
        kind: "add",
        at: view.add_at,
        code,
        name,
        condition,
      },
    ]);
    return error === "";
  }

  function toggleAdd() {
    showAdd = !showAdd;
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

  {#if view.no_categories}
    <p class="no-categories">{texts.noCategories}</p>
  {/if}

  <table>
    <thead>
      <tr>
        <th class="c-use">{texts.plan.use}</th>
        <th class="c-num">{texts.plan.hash}</th>
        <th class="c-category">{texts.plan.category}</th>
        <th class="c-orders">{texts.plan.orders}</th>
        <th class="c-runs">{texts.plan.runs}</th>
        <th class="c-file">{texts.plan.fileName}</th>
        <th class="c-tools"></th>
      </tr>
    </thead>
    <tbody>
      {#each view.rows as row (rowKey(row))}
        <PlanRow
          {row}
          open={expanded.includes(rowKey(row))}
          onToggleUse={(checked) => toggleUse(row, checked)}
          onMove={(up) => move(row, up)}
          onRename={(name) => rename(row, name)}
          onToggleOpen={() => toggleExpanded(rowKey(row))}
        />
      {/each}
    </tbody>
  </table>

  <div class="add-bar">
    <button onclick={toggleAdd}>{texts.plan.addCategory}</button>
    <button onclick={onEditCategories}>{texts.plan.editCategories}</button>
  </div>

  {#if showAdd}
    <AddCategoryForm onAdd={addCategory} onClose={() => (showAdd = false)} />
  {/if}

  <SaveProgress
    {saving}
    progress={saveProgress}
    {findings}
    onSaveAnyway={() => save(true)}
    onCancel={() => (findings = null)}
  />

  <div class="foot">
    <PackingListOptions
      {scope}
      {layouts}
      onScope={(value) => (scope = value)}
      onLayout={toggleLayout}
    />
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
  .no-categories {
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

  /* The column widths sit on the header cells; a row's cells line up under them. */
  th.c-use {
    width: 2.5rem;
  }

  th.c-num {
    width: 2.5rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  th.c-orders,
  th.c-runs {
    width: 4.5rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  th.c-tools {
    width: 7rem;
  }

  .add-bar {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
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

  @media (prefers-color-scheme: dark) {
    th {
      color: #b0b0b0;
      border-color: #444444;
    }

    .foot {
      border-color: #444444;
    }

    .muted {
      color: #b0b0b0;
    }
  }
</style>
