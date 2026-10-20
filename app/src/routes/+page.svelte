<script lang="ts">
  // The app shell's view (08 › *Window frame*, *Start*, *1. Day*, *2. New batch*, *3. Plan*,
  // *5. Categories*, *6. Settings*, *Search*): the state and the Tauri commands live in
  // `shell.svelte.ts` and the overlay dialogs in `Overlays.svelte`; here it only picks the screen
  // and wires the pieces. The batch flow is Day → New batch → Plan → Day; an amend is Day → Plan
  // (the batch's own files read again) → *Back* → the amend's selection page → Plan → Day. The
  // Plan screen's *Back* returns to the PDF selection page with the files still listed, for a new
  // batch and an amend alike.
  import { onMount } from "svelte";
  import { texts } from "../texts";
  import { Shell } from "../lib/shell.svelte";
  import Frame from "../lib/Frame.svelte";
  import StartPrompt from "../lib/StartPrompt.svelte";
  import DayScreen from "../lib/DayScreen.svelte";
  import NewBatchScreen from "../lib/NewBatchScreen.svelte";
  import PlanScreen from "../lib/PlanScreen.svelte";
  import Overlays from "../lib/Overlays.svelte";

  const shell = new Shell();
  onMount(shell.load);
</script>

{#if !shell.ready}
  <p class="boot">{texts.day.loading}</p>
{:else if shell.needsFolder}
  <StartPrompt
    exeDir={shell.exeDir}
    busy={shell.busy}
    onUse={shell.useProgramFolder}
    onChoose={shell.chooseFolder}
  />
{:else}
  <Frame
    day={shell.day}
    screen={shell.screen}
    overlay={shell.overlay}
    version={shell.version}
    onStep={shell.stepDay}
    onToday={shell.goToday}
    onNavigate={shell.navigate}
  />
  {#if shell.error}
    <p class="error">{shell.error}</p>
  {/if}
  {#if shell.screen === "day"}
    <DayScreen
      overview={shell.overview}
      saved={shell.savedMessage}
      reading={shell.amendReading}
      onOpen={shell.openFile}
      onOpenFolder={shell.openFolder}
      onSetPrinted={shell.setPrinted}
      onNewBatch={shell.startBatch}
      onAmend={shell.amend}
      onRevert={shell.revert}
    />
  {:else if shell.screen === "new-batch"}
    <NewBatchScreen
      batchNumber={shell.nextBatch}
      amend={shell.amendBatch}
      amendGrouping={shell.amendGrouping}
      files={shell.files}
      onFiles={(next) => (shell.files = next)}
      onPlan={shell.onPlan}
      onCancel={shell.cancelBatch}
    />
  {:else if shell.screen === "plan" && shell.readResult}
    <PlanScreen
      batchNumber={shell.nextBatch}
      read={shell.readResult}
      settings={shell.settings}
      refresh={shell.planRefresh}
      onSaved={shell.onSaved}
      onBack={shell.backToFiles}
      onCancel={shell.cancelBatch}
      onEditCategories={shell.openCategoriesFromPlan}
    />
  {/if}

  <Overlays {shell} />
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
