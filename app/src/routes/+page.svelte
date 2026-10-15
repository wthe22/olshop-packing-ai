<script lang="ts">
  // The app shell (08 › *Window frame*, *Start*, *1. Day*, *2. New batch*, *3. Plan*,
  // *5. Categories*, *6. Settings*): it finds the data folder, keeps the current day and screen,
  // routes the Tauri commands, and opens Categories/Settings over the current screen (an open
  // draft is kept). The batch flow is Day → New batch → Plan → Day.
  import { onMount } from "svelte";
  // Use the plugin's own `confirm`, not `window.confirm`: the dialog plugin patches the global to
  // call `plugin:dialog|confirm`, a command that no longer exists (it is now `message`), so the
  // global rejects and the promise it returns is always truthy. The plugin's `confirm` goes through
  // `message` and must be awaited.
  import { confirm, open } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api";
  import { texts } from "../texts";
  import Frame from "../lib/Frame.svelte";
  import StartPrompt from "../lib/StartPrompt.svelte";
  import DayScreen from "../lib/DayScreen.svelte";
  import NewBatchScreen from "../lib/NewBatchScreen.svelte";
  import PlanScreen from "../lib/PlanScreen.svelte";
  import CategoriesScreen from "../lib/CategoriesScreen.svelte";
  import SettingsScreen from "../lib/SettingsScreen.svelte";
  import Overlay from "../lib/Overlay.svelte";

  type Screen = "day" | "new-batch" | "plan";
  type OverlayId = "categories" | "settings";
  type NavId = "day" | "categories" | "settings";

  let ready = $state(false);
  let needsFolder = $state(true);
  let busy = $state(false);
  let error = $state("");

  let today = $state("");
  let day = $state("");
  let exeDir = $state("");
  let dataFolder = $state<string | null>(null);
  let overview = $state<api.DayOverview | null>(null);
  let settings = $state<api.Settings | null>(null);
  let about = $state<api.About | null>(null);
  let dayList = $state<string[]>([]);
  let screen = $state<Screen>("day");
  // Categories/Settings open over the current screen and close back to it (08 › *Window frame*).
  let overlay = $state<OverlayId | null>(null);

  // The batch being prepared: the read result and whether the backend has an open draft.
  let readResult = $state<api.ReadResult | null>(null);
  let draftOpen = $state(false);
  // The green *Batch n saved* line on the Day screen, shown for a few seconds (08 › *1. Day*).
  let savedMessage = $state<string | null>(null);
  let savedTimer: ReturnType<typeof setTimeout> | undefined;
  // Bumped after Categories saves, so the open Plan screen recounts (08 › *Window frame*).
  let planRefresh = $state(0);

  // The batch number the next save will get (continuing the day).
  const nextBatch = $derived((overview?.totals.batches ?? 0) + 1);
  // The batch number being prepared, for the Categories *In batch `<b>`: n* column; `null` when no
  // draft is open.
  const draftBatch = $derived(draftOpen ? (readResult?.amend ?? nextBatch) : null);

  async function afterFolder() {
    try {
      dayList = await api.days();
    } catch {
      dayList = [];
    }
    await refreshOverview();
    try {
      settings = await api.settingsGet();
    } catch (e) {
      error = String(e);
    }
    try {
      about = await api.about();
    } catch (e) {
      error = String(e);
    }
  }

  async function refreshOverview() {
    try {
      overview = await api.dayOverview(day);
    } catch (e) {
      error = String(e);
    }
  }

  onMount(async () => {
    try {
      const info = await api.startInfo();
      today = info.today;
      day = info.today;
      exeDir = info.exe_dir;
      dataFolder = info.data_folder;
      needsFolder = info.data_folder === null;
      if (!needsFolder) {
        await afterFolder();
        await openCategoriesIfBroken();
      }
    } catch (e) {
      error = String(e);
    }
    ready = true;
  });

  /// 08 › *Start* step 3: a `categories.toml` with errors opens Categories first.
  async function openCategoriesIfBroken() {
    try {
      const rules = await api.rulesLoad();
      if (rules.error) overlay = "categories";
    } catch {
      // No categories file yet: nothing to open.
    }
  }

  async function useProgramFolder() {
    busy = true;
    try {
      dataFolder = await api.useProgramFolder();
      needsFolder = false;
      await afterFolder();
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }

  async function pickFolder(title: string): Promise<string | null> {
    const picked = await open({ directory: true, multiple: false, title });
    return typeof picked === "string" ? picked : null;
  }

  async function chooseFolder() {
    const picked = await pickFolder(texts.start.chooseFolder);
    if (!picked) return;
    busy = true;
    try {
      dataFolder = await api.chooseDataFolder(picked);
      needsFolder = false;
      await afterFolder();
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }

  async function changeFolder() {
    const picked = await pickFolder(texts.settings.change);
    if (!picked) return;
    if (!(await confirm(texts.settings.confirmChange(picked, dataFolder ?? "")))) return;
    try {
      dataFolder = await api.chooseDataFolder(picked);
      await afterFolder();
    } catch (e) {
      error = String(e);
    }
  }

  /// Ask to discard the open batch before leaving it (08 › *Window frame*: changing the day while
  /// a draft is open asks first; Categories and Settings keep it).
  async function leaveBatch(): Promise<boolean> {
    if (!draftOpen) return true;
    if (!(await confirm(texts.plan.confirmDiscard(draftBatch ?? nextBatch)))) return false;
    await api.discardDraft();
    draftOpen = false;
    readResult = null;
    return true;
  }

  function startBatch() {
    savedMessage = null;
    overlay = null;
    screen = "new-batch";
  }

  function onPlan(result: api.ReadResult) {
    readResult = result;
    draftOpen = true;
    overlay = null;
    screen = "plan";
  }

  async function onSaved(result: api.SaveResult) {
    draftOpen = false;
    readResult = null;
    overlay = null;
    savedMessage = texts.day.saved(result.batch, result.pdfs, result.packing_lists);
    screen = "day";
    await refreshOverview();
    if (savedTimer) clearTimeout(savedTimer);
    savedTimer = setTimeout(() => (savedMessage = null), 6000);
  }

  function cancelBatch() {
    // From New batch nothing was read; from Plan the screen already discarded the draft.
    draftOpen = false;
    readResult = null;
    screen = "day";
  }

  // The day switch (08 › *Window frame*). `dayList` is newest first.
  async function stepDay(delta: number) {
    if (!(await leaveBatch())) return;
    const next = dayList[dayList.indexOf(day) - delta];
    if (next) {
      day = next;
      overlay = null;
      refreshOverview();
    }
  }

  async function goToday() {
    if (!(await leaveBatch())) return;
    day = today;
    overlay = null;
    refreshOverview();
  }

  // *Day* returns to the day screen; *Categories* and *Settings* open over the current screen
  // (08 › *Window frame*: an open draft is kept).
  async function navigate(id: NavId) {
    if (id === "day") {
      if (!(await leaveBatch())) return;
      overlay = null;
      screen = "day";
    } else {
      overlay = id;
    }
  }

  function closeOverlay() {
    overlay = null;
  }

  function onCategoriesSaved() {
    overlay = null;
    planRefresh += 1; // the open Plan screen recounts with the new categories
  }

  /// *Revert* the newest batch (08 › *1. Day*): confirm, then delete it.
  async function revert(batch: api.BatchView) {
    const asked = await confirm(texts.day.confirmRevert(batch.batch, batch.pdfs.length, batch.orders), {
      title: texts.title,
      okLabel: texts.day.undoBatch(batch.batch),
    });
    if (!asked) return;
    try {
      await api.revertLast(day);
      await refreshOverview();
    } catch (e) {
      error = String(e);
    }
  }

  /// *Amend* the newest batch (08 › *1. Day*): confirm, then read its download copy into a draft
  /// and open the *Redo batch `<b>`* Plan screen.
  async function amend(batch: api.BatchView) {
    const asked = await confirm(texts.day.confirmAmend(batch.batch, batch.time), {
      title: texts.title,
      okLabel: texts.day.redoBatch(batch.batch),
    });
    if (!asked) return;
    busy = true;
    try {
      const result = await api.startAmend(day);
      onPlan(result);
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }

  async function saveSettings(value: api.Settings) {
    try {
      await api.settingsSet(value);
      settings = value;
    } catch (e) {
      error = String(e);
    }
  }

  async function openFile(path: string) {
    try {
      await api.openPath(path);
    } catch (e) {
      error = String(e);
    }
  }

  async function openFolder(path: string) {
    try {
      await api.showInFolder(path);
    } catch (e) {
      error = String(e);
    }
  }
</script>

{#if !ready}
  <p class="boot">{texts.day.loading}</p>
{:else if needsFolder}
  <StartPrompt {exeDir} {busy} onUse={useProgramFolder} onChoose={chooseFolder} />
{:else}
  <Frame {day} {screen} {overlay} onStep={stepDay} onToday={goToday} onNavigate={navigate} />
  {#if error}
    <p class="error">{error}</p>
  {/if}
  {#if screen === "day"}
    <DayScreen
      {overview}
      saved={savedMessage}
      onOpen={openFile}
      onOpenFolder={openFolder}
      onNewBatch={startBatch}
      onAmend={amend}
      onRevert={revert}
    />
  {:else if screen === "new-batch"}
    <NewBatchScreen batchNumber={nextBatch} onPlan={onPlan} onCancel={cancelBatch} />
  {:else if screen === "plan" && readResult}
    <PlanScreen
      batchNumber={nextBatch}
      read={readResult}
      {settings}
      refresh={planRefresh}
      onSaved={onSaved}
      onCancel={cancelBatch}
      onEditCategories={() => (overlay = "categories")}
    />
  {/if}

  {#if overlay === "categories"}
    <Overlay title={texts.categories.title} onClose={closeOverlay}>
      <CategoriesScreen {draftBatch} onClose={closeOverlay} onSaved={onCategoriesSaved} />
    </Overlay>
  {:else if overlay === "settings"}
    <Overlay title={texts.settings.title} onClose={closeOverlay}>
      <SettingsScreen
        {settings}
        {about}
        {dataFolder}
        onChangeFolder={changeFolder}
        onOpenFolder={() => dataFolder && openFolder(dataFolder)}
        onSave={saveSettings}
      />
    </Overlay>
  {/if}
{/if}

<style>
  .boot {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 60vh;
    color: #6a6a6a;
  }
</style>
