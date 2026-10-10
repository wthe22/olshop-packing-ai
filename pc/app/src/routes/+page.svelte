<script lang="ts">
  // The app shell (08 › *Window frame*, *Start*, *1. Day*, *2. New batch*, *3. Plan*,
  // *6. Settings*): it finds the data folder, keeps the current day and screen, and routes the
  // Tauri commands. The batch flow is Day → New batch → Plan → Day.
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api";
  import { texts } from "../texts";
  import Frame from "../lib/Frame.svelte";
  import StartPrompt from "../lib/StartPrompt.svelte";
  import DayScreen from "../lib/DayScreen.svelte";
  import NewBatchScreen from "../lib/NewBatchScreen.svelte";
  import PlanScreen from "../lib/PlanScreen.svelte";
  import SettingsScreen from "../lib/SettingsScreen.svelte";
  import Placeholder from "../lib/Placeholder.svelte";

  type Screen = "day" | "new-batch" | "plan" | "categories" | "settings";

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

  // The batch being prepared: the read result and whether the backend has an open draft.
  let readResult = $state<api.ReadResult | null>(null);
  let draftOpen = $state(false);
  // The green *Batch n saved* line on the Day screen, shown for a few seconds (08 › *1. Day*).
  let savedMessage = $state<string | null>(null);
  let savedTimer: ReturnType<typeof setTimeout> | undefined;

  // The batch number the next save will get (continuing the day).
  const nextBatch = $derived((overview?.totals.batches ?? 0) + 1);

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
      if (!needsFolder) await afterFolder();
    } catch (e) {
      error = String(e);
    }
    ready = true;
  });

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
    if (!confirm(texts.settings.confirmChange(picked, dataFolder ?? ""))) return;
    try {
      dataFolder = await api.chooseDataFolder(picked);
      await afterFolder();
    } catch (e) {
      error = String(e);
    }
  }

  /// Ask to discard the open batch before leaving it (08 › *Window frame*: changing the day
  /// while a draft is open asks first).
  async function leaveBatch(): Promise<boolean> {
    if (!draftOpen) return true;
    if (!confirm(texts.plan.confirmDiscard(nextBatch))) return false;
    await api.discardDraft();
    draftOpen = false;
    readResult = null;
    return true;
  }

  function startBatch() {
    savedMessage = null;
    screen = "new-batch";
  }

  function onPlan(result: api.ReadResult) {
    readResult = result;
    draftOpen = true;
    screen = "plan";
  }

  async function onSaved(result: api.SaveResult) {
    draftOpen = false;
    readResult = null;
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
      refreshOverview();
    }
  }

  async function goToday() {
    if (!(await leaveBatch())) return;
    day = today;
    refreshOverview();
  }

  async function navigate(next: Screen) {
    if (!(await leaveBatch())) return;
    screen = next;
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
  <Frame
    {day}
    {screen}
    onStep={stepDay}
    onToday={goToday}
    onNavigate={navigate}
  />
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
    />
  {:else if screen === "new-batch"}
    <NewBatchScreen
      batchNumber={nextBatch}
      onPlan={onPlan}
      onCancel={cancelBatch}
    />
  {:else if screen === "plan" && readResult}
    <PlanScreen
      batchNumber={nextBatch}
      read={readResult}
      {settings}
      onSaved={onSaved}
      onCancel={cancelBatch}
    />
  {:else if screen === "categories"}
    <Placeholder text={texts.comingSoon.categories} />
  {:else}
    <SettingsScreen
      {settings}
      {about}
      {dataFolder}
      onChangeFolder={changeFolder}
      onOpenFolder={() => dataFolder && openFolder(dataFolder)}
      onSave={saveSettings}
    />
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
