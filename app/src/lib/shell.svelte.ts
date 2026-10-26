// The app shell's state and commands: it finds the data folder and the owner, keeps the current
// day and screen, routes the Tauri commands, and decides which overlay opens over the current
// screen (an open draft is kept). The markup is `+page.svelte` and the overlay dialogs are
// `Overlays.svelte`; nothing is decided there.
//
// The batch flow is Day → New batch → Plan → Day; an amend is Day → Plan (the batch's own files
// read again) → *Back* → the amend's selection page → Plan → Day. The Plan screen's *Back* returns
// to the PDF selection page with the files still listed, for a new batch and an amend alike.

import { confirm, open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import * as start from "./api/start";
import * as dayData from "./api/day";
import * as batch from "./api/batch";
import { togglePrinted } from "./printed";
import { texts, amendOpens } from "../texts";

// The screens the shell routes between; the batch flow's screens are not in the nav.
export type Screen = "day" | "new-batch" | "plan";
export type OverlayId = "settings";
export type NavId = "day" | "settings";

export class Shell {
  ready = $state(false);
  needsFolder = $state(true);
  // The data folder holds no owner yet: the first owner's name is asked.
  needsOwner = $state(false);
  busy = $state(false);
  error = $state("");

  today = $state("");
  day = $state("");
  exeDir = $state("");
  dataFolder = $state<string | null>(null);
  owner = $state<string | null>(null);
  // The program's shown version, for the title bar's brand text; the OS
  // window title is set from the same one-place text on the Rust side at start.
  version = $state("");
  overview = $state<dayData.DayOverview | null>(null);
  about = $state<start.About | null>(null);
  dayList = $state<string[]>([]);
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
  // The progress of *Amend*'s read of the batch's own files (the `read-progress` event), shown on
  // the Day screen until the Plan screen opens; `null` when nothing
  // reads.
  amendReading = $state<batch.ReadProgress | null>(null);
  // The green *Batch n saved* line on the Day screen, shown for a few seconds.
  savedMessage = $state<string | null>(null);
  private savedTimer: ReturnType<typeof setTimeout> | undefined;

  // The batch number the next save will get (continuing the day).
  nextBatch = $derived((this.overview?.totals.batches ?? 0) + 1);
  // The batch's own sort rule, for the amend's selection page's fixed line;
  // `null` for a new batch's file list.
  amendGrouping = $derived(
    this.amendBatch !== null && this.readResult !== null ? this.readResult.plan.grouping : null,
  );

  // Start the shell: the data folder, the owner, the day and About.
  load = async () => {
    try {
      const info = await start.startInfo();
      this.today = info.today;
      this.day = info.today;
      this.exeDir = info.exe_dir;
      this.dataFolder = info.data_folder;
      this.owner = info.owner;
      this.version = info.version;
      this.needsFolder = info.data_folder === null;
      if (!this.needsFolder) await this.afterFolder();
    } catch (e) {
      this.error = String(e);
    }
    this.ready = true;
  };

  private afterFolder = async () => {
    this.needsOwner = this.owner === null;
    if (this.needsOwner) return;
    try {
      this.dayList = await dayData.days();
    } catch {
      this.dayList = [];
    }
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

  // The data folder's owner, after a folder is chosen: the one the start opened, if any.
  private ownerOfFolder = async () => {
    this.owner = (await start.startInfo()).owner ?? null;
  };

  useProgramFolder = async () => {
    this.busy = true;
    try {
      this.dataFolder = await start.useProgramFolder();
      await this.ownerOfFolder();
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
      await this.ownerOfFolder();
      this.needsFolder = false;
      await this.afterFolder();
    } catch (e) {
      this.error = String(e);
    }
    this.busy = false;
  };

  // The first owner's name: the owner's folder is made and today's date opens.
  nameOwner = async (name: string) => {
    this.busy = true;
    try {
      this.owner = await start.ownerNew(name);
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
    if (!(await confirm(texts.settings.confirmChange(picked, this.dataFolder ?? "")))) return;
    try {
      this.dataFolder = await start.chooseDataFolder(picked);
      await this.ownerOfFolder();
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
      if (!(await confirm(texts.day.confirmDiscardDraft))) return false;
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

  onPlan = (result: batch.ReadResult) => {
    this.readResult = result;
    this.draftOpen = true;
    this.overlay = null;
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
    await this.refreshOverview();
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

  // The day switch. `dayList` is newest first.
  stepDay = async (delta: number) => {
    if (!(await this.leaveBatch())) return;
    const next = this.dayList[this.dayList.indexOf(this.day) - delta];
    if (next) {
      this.day = next;
      this.overlay = null;
      this.refreshOverview();
    }
  };

  goToday = async () => {
    if (!(await this.leaveBatch())) return;
    this.day = this.today;
    this.overlay = null;
    this.refreshOverview();
  };

  // *Day* returns to the day screen; *Settings* opens over the current screen.
  navigate = async (id: NavId) => {
    if (id === "day") {
      if (!(await this.leaveBatch())) return;
      this.overlay = null;
      this.screen = "day";
    } else {
      this.overlay = id;
    }
  };

  closeOverlay = () => {
    this.overlay = null;
  };

  // *Revert* the newest batch: confirm, then revert it.
  revert = async (row: dayData.BatchView) => {
    const asked = await confirm(texts.day.confirmRevert(row.batch, row.pdfs.length, row.orders), {
      title: texts.title,
      okLabel: texts.day.undoBatch(row.batch),
    });
    if (!asked) return;
    try {
      await dayData.revertLast(this.day);
      await this.refreshOverview();
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
    const asked = await confirm(texts.day.confirmAmend(row.batch, row.sort_rule, row.time), {
      title: texts.title,
      okLabel: texts.day.redoBatch(row.batch),
    });
    if (!asked) return;
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
