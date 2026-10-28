<script lang="ts">
  // The app shell's view: the state and the Tauri commands live in
  // `shell.svelte.ts` and the overlay dialogs in `Overlays.svelte`; here it only picks the screen
  // and wires the pieces. The batch flow is Day → New batch → Plan → Day; an amend is Day → Plan
  // (the batch's own files read again) → *Back* → the amend's selection page → Plan → Day. The
  // Plan screen's *Back* returns to the PDF selection page with the files still listed, for a new
  // batch and an amend alike.
  import { onMount } from "svelte";
  import { texts } from "../texts";
  import { Shell } from "../lib/shell.svelte";
  import Frame from "../lib/Frame.svelte";
  import Panel from "../lib/Panel.svelte";
  import StartPrompt from "../lib/StartPrompt.svelte";
  import WorkspacePrompt from "../lib/WorkspacePrompt.svelte";
  import DayScreen from "../lib/DayScreen.svelte";
  import NewBatchScreen from "../lib/NewBatchScreen.svelte";
  import PlanScreen from "../lib/PlanScreen.svelte";
  import RepeatsScreen from "../lib/RepeatsScreen.svelte";
  import Overlays from "../lib/Overlays.svelte";
  import ConfirmDialog from "../lib/ConfirmDialog.svelte";
  import WorkingDialog from "../lib/WorkingDialog.svelte";
  import { applyTheme } from "../lib/theme";

  // Below the smallest window width the panel hides on its own.
  const SMALLEST_WIDTH = 1024;

  const shell = new Shell();
  $effect(() => applyTheme(shell.theme));
  onMount(() => {
    shell.load();
    const onKey = (event: KeyboardEvent) => {
      if (event.ctrlKey && event.key.toLowerCase() === "b") {
        event.preventDefault();
        shell.togglePanel();
      }
    };
    // A folder added in Explorer shows up when the window regains focus.
    const onFocus = () => shell.workspace && shell.refreshPanel();
    const onResize = () => {
      if (window.innerWidth < SMALLEST_WIDTH) shell.panelShown = false;
    };
    onResize();
    window.addEventListener("keydown", onKey);
    window.addEventListener("focus", onFocus);
    window.addEventListener("resize", onResize);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("focus", onFocus);
      window.removeEventListener("resize", onResize);
    };
  });
</script>

{#if !shell.ready}
  <p class="boot">{texts.day.loading}</p>
{:else if shell.needsFolder}
  <StartPrompt
    exeDir={shell.exeDir}
    version={shell.version}
    busy={shell.busy}
    onUse={shell.useProgramFolder}
    onChoose={shell.chooseFolder}
  />
{:else if shell.needsWorkspace}
  <WorkspacePrompt version={shell.version} busy={shell.busy} error={shell.error} onName={shell.nameWorkspace} />
{:else}
  <div class="window">
    <Frame
      workspace={shell.workspace}
      day={shell.day}
      overlay={shell.overlay}
      version={shell.version}
      onTogglePanel={shell.togglePanel}
      onNavigate={shell.navigate}
    />
    <div class="body">
      {#if shell.panelShown && shell.panel}
        <Panel
          panel={shell.panel}
          day={shell.day}
          today={shell.today}
          onOpenDate={shell.openDate}
          onOpenWorkspace={shell.openWorkspace}
          onNewWorkspace={shell.newWorkspace}
          onOpenFolder={shell.openFolder}
        />
      {/if}
      <div class="screen">
        {#if shell.error}
          <p class="error">{shell.error}</p>
        {/if}
        {#if shell.screen === "day"}
          <DayScreen
            overview={shell.overview}
            saved={shell.savedMessage}
            reading={shell.amendReading}
            writing={shell.writing}
            onOpen={shell.openFile}
            onOpenOutput={(file) => shell.openOutput(shell.day, file)}
            onOpenFolder={shell.openFolder}
            onSetPrinted={shell.setPrinted}
            onNewBatch={shell.startBatch}
            onDaySummary={shell.daySummary}
            onAmend={shell.amend}
            onRevert={shell.revert}
            onRestore={shell.restore}
          />
        {:else if shell.screen === "new-batch"}
          <NewBatchScreen
            batchNumber={shell.nextBatch}
            amend={shell.amendBatch}
            amendRule={shell.amendRule}
            files={shell.files}
            onFiles={(next) => (shell.files = next)}
            repeats={shell.draftOpen ? (shell.readResult?.repeats ?? null) : null}
            onPlan={shell.onPlan}
            onReviewRepeats={shell.reviewRepeats}
            onCancel={shell.cancelBatch}
          />
        {:else if shell.screen === "repeats" && shell.readResult}
          <RepeatsScreen
            read={shell.readResult}
            onResult={shell.onRepeats}
            onContinue={shell.continueToPlan}
            onBack={shell.backToFiles}
          />
        {:else if shell.screen === "plan" && shell.readResult}
          {#key shell.planVersion}
            <PlanScreen
              batchNumber={shell.nextBatch}
              read={shell.readResult}
              onSaved={shell.onSaved}
              onBack={shell.backToFiles}
              onChangeRepeats={shell.reviewRepeats}
              onEditRule={() => shell.navigate("sort-rules")}
              onCancel={shell.cancelBatch}
            />
          {/key}
        {/if}
      </div>
    </div>
  </div>

  <Overlays {shell} />
  <WorkingDialog text={shell.working} />
{/if}

<ConfirmDialog />

<style>
  .window {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .body {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .screen {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }

  .boot {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 60vh;
    color: var(--text-muted);
  }
</style>
