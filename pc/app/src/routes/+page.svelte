<script lang="ts">
  // The app shell (08 › *Window frame*, *Start*, *1. Day*, *6. Settings*): it finds the data
  // folder, keeps the current day and screen and routes the Tauri commands.
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api";
  import { texts } from "../texts";
  import Frame from "../lib/Frame.svelte";
  import StartPrompt from "../lib/StartPrompt.svelte";
  import DayScreen from "../lib/DayScreen.svelte";
  import SettingsScreen from "../lib/SettingsScreen.svelte";
  import Placeholder from "../lib/Placeholder.svelte";

  type Screen = "day" | "categories" | "settings";

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
  let dayList = $state<string[]>([]);
  let screen = $state<Screen>("day");

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

  // The day switch (08 › *Window frame*). `dayList` is newest first.
  function stepDay(delta: number) {
    const next = dayList[dayList.indexOf(day) - delta];
    if (next) {
      day = next;
      refreshOverview();
    }
  }

  function goToday() {
    day = today;
    refreshOverview();
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
    onNavigate={(next: Screen) => (screen = next)}
  />
  {#if error}
    <p class="error">{error}</p>
  {/if}
  {#if screen === "day"}
    <DayScreen
      {overview}
      onOpen={openFile}
      onOpenFolder={openFolder}
      onNewBatch={() => {}}
    />
  {:else if screen === "categories"}
    <Placeholder text={texts.comingSoon.categories} />
  {:else}
    <SettingsScreen
      {settings}
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
