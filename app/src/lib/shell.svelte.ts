// The app shell's state and commands: it finds the data folder and the workspace, keeps the current
// day and screen, routes the Tauri commands, and decides which overlay opens over the current
// screen (an open draft is kept). The markup is `+page.svelte` and the overlay dialogs are
// `Overlays.svelte`; nothing is decided there.
//
// The batch flow is Day → New batch → Plan → Day; an amend is Day → Plan (the batch's own files
// read again) → *Back* → the amend's selection page → Plan → Day. The Plan screen's *Back* returns
// to the PDF selection page with the files still listed, for a new batch and an amend alike.

import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import * as start from "./api/start";
import * as dayData from "./api/day";
import * as panelData from "./api/panel";
import * as batch from "./api/batch";
import { togglePrinted } from "./printed";
import { texts } from "../texts";
import { amendOpens } from "./planFlow";
import { confirmBox } from "./confirm.svelte";
import type { Theme } from "./theme";

// The screens the shell routes between; the batch flow's screens are not in the nav.
export type Screen = "day" | "new-batch" | "repeats" | "plan";
export type OverlayId = "search" | "sort-rules" | "settings";

// A write the owner waits for shows its dialog only when it takes longer than this, so an *Open*
// of a file that is there does not flash one.
const WORKING_DELAY_MS = 200;

export class Shell {
  ready = $state(false);
  needsFolder = $state(true);
  // The data folder holds no workspace yet: the first workspace's name is asked.
  needsWorkspace = $state(false);
  busy = $state(false);
  error = $state("");

  today = $state("");
  day = $state("");
  exeDir = $state("");
  dataFolder = $state<string | null>(null);
  workspace = $state<string | null>(null);
  // The theme `packing-settings.toml` names; `+page.svelte` shows it.
  theme = $state<Theme>("system");
  // The program's shown version, for the title bar's brand text; the OS
  // window title is set from the same one-place text on the Rust side at start.
  version = $state("");
  overview = $state<dayData.DayOverview | null>(null);
  about = $state<start.About | null>(null);
  // The side panel: the workspaces and the open one's dates. It is shown at every start.
  panel = $state<panelData.PanelView | null>(null);
  panelShown = $state(true);
  private panelTimer: ReturnType<typeof setTimeout> | undefined;
  screen = $state<Screen>("day");
  // Settings opens over the current screen and closes back to it.
  overlay = $state<OverlayId | null>(null);

  // The batch being prepared: the chosen label files (kept here, so *Back* on the Plan screen
  // shows the file list again with the files still there), the read result and whether the
  // backend has an open draft. `amendBatch` is the batch being
  // redone, from **Amend** until the flow ends; `null` for a
  // new batch.
  files = $state<batch.FileCheck[]>([]);
  amendBatch = $state<number | null>(null);
  readResult = $state<batch.ReadResult | null>(null);
  draftOpen = $state(false);
  // Counts the plans the back end rebuilt under an open Plan screen, which then opens again.
  planVersion = $state(0);
  // The progress of *Amend*'s read of the batch's own files (the `read-progress` event), shown on
  // the Day screen until the Plan screen opens; `null` when nothing
  // reads.
  amendReading = $state<batch.ReadProgress | null>(null);
  // The green *Batch n saved* line on the Day screen, shown for a few seconds.
  savedMessage = $state<string | null>(null);
  private savedTimer: ReturnType<typeof setTimeout> | undefined;
  // The dialog line of a file being written that the owner waits for (the day summary, an output
  // written again); `null` while none is shown.
  working = $state<string | null>(null);
  // Such a write is running: a second click starts nothing.
  writing = $state(false);

  // The batch number the next save will get (continuing the day).
  nextBatch = $derived((this.overview?.totals.batches ?? 0) + 1);
  // The batch's own sort rule, for the amend's selection page's fixed line;
  // `null` for a new batch's file list.
  amendRule = $derived(
    this.amendBatch !== null && this.readResult !== null ? this.readResult.plan.sort_rule : null,
  );

  // Start the shell: the data folder, the workspace, the day and About.
  load = async () => {
    try {
      const info = await start.startInfo();
      this.today = info.today;
      this.day = info.today;
      this.exeDir = info.exe_dir;
      this.dataFolder = info.data_folder;
      this.workspace = info.workspace;
      this.version = info.version;
      this.theme = info.theme;
      this.needsFolder = info.data_folder === null;
      if (!this.needsFolder) await this.afterFolder();
    } catch (e) {
      this.error = String(e);
    }
    this.ready = true;
  };

  // The theme menu: shown at once (the window follows `theme`), then stored.
  chooseTheme = async (theme: Theme) => {
    this.theme = theme;
    try {
      await start.setTheme(theme);
    } catch (e) {
      this.error = String(e);
    }
  };

  private afterFolder = async () => {
    this.needsWorkspace = this.workspace === null;
    if (this.needsWorkspace) return;
    await this.refreshPanel();
    await this.refreshOverview();
    try {
      this.about = await start.about();
    } catch (e) {
      this.error = String(e);
    }
  };

  private refreshOverview = async () => {
    try {
      this.overview = await dayData.dayOverview(this.day);
    } catch (e) {
      this.error = String(e);
    }
  };

  // The panel's list, read again after each change and when the window regains focus. While the
  // search index is being built it has no counts yet, so it is asked again shortly.
  refreshPanel = async () => {
    clearTimeout(this.panelTimer);
    try {
      this.panel = await panelData.workspacesList();
    } catch (e) {
      this.error = String(e);
      return;
    }
    if (this.panel.loading) this.panelTimer = setTimeout(this.refreshPanel, 500);
  };

  // The data folder's workspace, after a folder is chosen: the one the start opened, if any.
  private workspaceOfFolder = async () => {
    this.workspace = (await start.startInfo()).workspace ?? null;
  };

  useProgramFolder = async () => {
    this.busy = true;
    try {
      this.dataFolder = await start.useProgramFolder();
      await this.workspaceOfFolder();
      this.needsFolder = false;
      await this.afterFolder();
    } catch (e) {
      this.error = String(e);
    }
    this.busy = false;
  };

  chooseFolder = async () => {
    const picked = await this.pickFolder(texts.start.chooseFolder);
    if (!picked) return;
    this.busy = true;
    try {
      this.dataFolder = await start.chooseDataFolder(picked);
      await this.workspaceOfFolder();
      this.needsFolder = false;
      await this.afterFolder();
    } catch (e) {
      this.error = String(e);
    }
    this.busy = false;
  };

  // The first workspace's name: its folder is made and today's date opens.
  nameWorkspace = async (name: string) => {
    this.busy = true;
    try {
      this.workspace = await start.workspaceNew(name);
      this.error = "";
      await this.afterFolder();
    } catch (e) {
      this.error = String(e);
    }
    this.busy = false;
  };

  changeFolder = async () => {
    const picked = await this.pickFolder(texts.settings.change);
    if (!picked) return;
    if (!(await confirmBox.ask(texts.settings.askChange(picked, this.dataFolder ?? "")))) return;
    try {
      this.dataFolder = await start.chooseDataFolder(picked);
      await this.workspaceOfFolder();
      await this.afterFolder();
    } catch (e) {
      this.error = String(e);
    }
  };

  private pickFolder = async (title: string): Promise<string | null> => {
    const picked = await open({ directory: true, multiple: false, title });
    return typeof picked === "string" ? picked : null;
  };

  // Ask to discard the open batch before leaving it.
  private leaveBatch = async (): Promise<boolean> => {
    if (this.draftOpen) {
      if (!(await confirmBox.ask(texts.day.askDiscardDraft))) return false;
      await batch.discardDraft();
      this.draftOpen = false;
      this.readResult = null;
    }
    this.files = [];
    this.amendBatch = null;
    return true;
  };

  startBatch = () => {
    this.savedMessage = null;
    this.overlay = null;
    this.files = [];
    this.amendBatch = null;
    this.screen = "new-batch";
  };

  // *Back* on a Plan screen: show the PDF selection page again — the batch's own
  // file list, or the amend's selection page — with the files still listed and the draft kept; a
  // new *Read labels* replaces it, so the plan changes are lost. Nothing is written and nothing
  // is asked.
  backToFiles = () => {
    this.screen = "new-batch";
  };

  // A read ended (after the logo prompt): Repeated orders first when the batch holds a repeat.
  onPlan = (result: batch.ReadResult) => {
    this.readResult = result;
    this.draftOpen = true;
    this.overlay = null;
    this.screen = result.repeats ? "repeats" : "plan";
  };

  // A choice on Repeated orders: the read as it now is.
  onRepeats = (result: batch.ReadResult) => {
    this.readResult = result;
  };

  // *Review the repeats* on the file list and *Change…* on the Plan screen.
  reviewRepeats = () => {
    this.screen = "repeats";
  };

  continueToPlan = () => {
    this.screen = "plan";
  };

  onSaved = async (result: batch.SaveResult) => {
    this.draftOpen = false;
    this.readResult = null;
    this.files = [];
    this.amendBatch = null;
    this.overlay = null;
    this.savedMessage = texts.day.saved(result.batch, result.pdfs);
    if (result.after_save.length > 0) this.error = result.after_save.join(" ");
    this.screen = "day";
    await Promise.all([this.refreshOverview(), this.refreshPanel()]);
    if (this.savedTimer) clearTimeout(this.savedTimer);
    this.savedTimer = setTimeout(() => (this.savedMessage = null), 6000);
  };

  // Leave the batch for the Day screen: *Cancel* on the PDF selection page (a new batch's file
  // list and the amend's selection page) and *Cancel amend*. Only here is the open draft
  // discarded (nothing is written, and an amend asks nothing).
  cancelBatch = async () => {
    if (this.draftOpen) await batch.discardDraft();
    this.draftOpen = false;
    this.readResult = null;
    this.files = [];
    this.amendBatch = null;
    this.screen = "day";
  };

  // A date of the panel opens on the Day screen; a draft being prepared asks first.
  openDate = async (date: string) => {
    if (!(await this.leaveBatch())) return;
    this.day = date;
    this.overlay = null;
    this.screen = "day";
    await this.refreshOverview();
  };

  // Another workspace of the panel, opened on today's date.
  openWorkspace = async (name: string) => {
    if (name === this.workspace || !(await this.leaveBatch())) return;
    try {
      await panelData.workspaceOpen(name);
    } catch (e) {
      this.error = String(e);
      return;
    }
    this.workspace = name;
    await this.openToday();
  };

  // **+ New workspace**: its folder is made and it opens on today's date. A refused name returns
  // why, for the line beside the field.
  newWorkspace = async (name: string): Promise<string | null> => {
    if (!(await this.leaveBatch())) return null;
    try {
      this.workspace = await start.workspaceNew(name);
    } catch (e) {
      return String(e);
    }
    await this.openToday();
    return null;
  };

  private openToday = async () => {
    this.day = this.today;
    this.overlay = null;
    this.screen = "day";
    await Promise.all([this.refreshPanel(), this.refreshOverview()]);
  };

  togglePanel = () => {
    this.panelShown = !this.panelShown;
  };

  // *Search* and *Settings* open over the current screen.
  navigate = (id: OverlayId) => {
    this.overlay = id;
  };

  // A sort rule was written: the open batch's plan follows the file, so the Plan screen opens
  // again on the rebuilt plan.
  afterRulesSaved = async () => {
    if (!this.draftOpen || !this.readResult) return;
    try {
      this.readResult = { ...this.readResult, plan: await batch.plan([]) };
      this.planVersion += 1;
    } catch (e) {
      this.error = String(e);
    }
  };

  // A reprint was written into today: today's Day screen lists it.
  afterReprint = async () => {
    await Promise.all([this.refreshOverview(), this.refreshPanel()]);
  };

  closeOverlay = () => {
    this.overlay = null;
  };

  // Run a write the owner waits for: its dialog shows after a moment and a second one is not
  // started while it runs. `null` when it did not run or failed (the failure is shown).
  private waitFor = async <T>(text: string, job: () => Promise<T>): Promise<T | null> => {
    if (this.writing) return null;
    this.writing = true;
    const shown = setTimeout(() => (this.working = text), WORKING_DELAY_MS);
    try {
      return await job();
    } catch (e) {
      this.error = String(e);
      return null;
    } finally {
      clearTimeout(shown);
      this.working = null;
      this.writing = false;
    }
  };

  // **Restore** an output found changed: written again from the stored inputs and checked like
  // the original; a finding keeps nothing and is shown.
  restore = async (file: string) => {
    const result = await this.waitFor(texts.day.writingAgain(file), () =>
      dayData.restoreOutput(this.day, file),
    );
    if (!result) return;
    if (!result.restored) this.error = texts.day.restoreFailed(file, result.errors);
    await this.refreshOverview();
  };

  // *Open* on a group PDF or a summary PDF of `day`: one that is not there any more is written
  // again first, checked like a **Restore**, so an old batch's summary shows today's layout.
  openOutput = async (day: string, file: string) => {
    const opened = await this.waitFor(texts.day.writingAgain(file), () =>
      dayData.openOutput(day, file),
    );
    if (!opened) return;
    if (opened.kind === "not_kept") this.error = texts.day.restoreFailed(file, opened.errors);
    if (opened.kind !== "opened" && day === this.day) await this.refreshOverview();
  };

  // *Revert* the newest batch: confirm, then revert it.
  revert = async (row: dayData.BatchView) => {
    if (!(await confirmBox.ask(texts.day.askRevert(row.batch, row.pdfs.length)))) return;
    try {
      await dayData.revertLast(this.day);
      await Promise.all([this.refreshOverview(), this.refreshPanel()]);
    } catch (e) {
      this.error = String(e);
    }
  };

  // *Amend* the newest batch: confirm, then read the batch's own
  // inputs again and open the *Redo batch `<b>`* Plan screen at once, the draft open.
  // The batch's file list is kept, so the Plan screen's *Back* shows the selection page with the
  // current selection listed. The read's
  // `read-progress` events show on the Day screen until the Plan screen opens.
  amend = async (row: dayData.BatchView) => {
    if (!(await confirmBox.ask(texts.day.askAmend(row.batch, row.sort_rule)))) return;
    this.busy = true;
    this.amendReading = null;
    const unlisten = await listen<batch.ReadProgress>("read-progress", (event) => {
      this.amendReading = event.payload;
    });
    try {
      const begin = await batch.startAmend(this.day);
      this.readResult = begin.read;
      this.files = begin.files;
      this.amendBatch = begin.batch;
      this.draftOpen = true;
      this.savedMessage = null;
      this.overlay = null;
      this.screen = amendOpens();
    } catch (e) {
      this.error = String(e);
    }
    unlisten();
    this.amendReading = null;
    this.busy = false;
  };

  openFile = async (path: string) => {
    try {
      await start.openPath(path);
    } catch (e) {
      this.error = String(e);
    }
  };

  openFolder = async (path: string) => {
    try {
      await start.showInFolder(path);
    } catch (e) {
      this.error = String(e);
    }
  };

  // *Day summary*: the open date's summary PDF, written from its journal, then opened.
  daySummary = async () => {
    await this.waitFor(texts.day.writingDaySummary, async () =>
      start.openPath(await dayData.writeDaySummary(this.day)),
    );
  };

  // The **Printed** tick of one group PDF: write it and update the batch's rows in place, without
  // reloading the whole overview. The tick rule is the pure [`togglePrinted`]; a failed write
  // shows the usual error.
  setPrinted = async (row: dayData.BatchView, pdf: dayData.GroupPdfView, printed: boolean) => {
    try {
      await dayData.setPrinted(this.day, row.batch, pdf.group, printed);
      const ticked = row.pdfs.filter((item) => item.printed).map((item) => item.group);
      const groups = togglePrinted(ticked, pdf.group, printed);
      for (const item of row.pdfs) item.printed = groups.includes(item.group);
    } catch (e) {
      this.error = String(e);
    }
  };
}
