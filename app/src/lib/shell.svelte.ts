// The app shell's state and commands: it finds the data folder, keeps the current
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
import * as categories from "./api/categories";
import { togglePrinted } from "./printed";
import { texts, amendOpens } from "../texts";

// The screens the shell routes between; the batch flow's screens are not in the nav.
export type Screen = "day" | "new-batch" | "plan";
export type OverlayId = "search" | "categories" | "settings";
export type NavId = "day" | "search" | "categories" | "settings";

export class Shell {
  ready = $state(false);
  needsFolder = $state(true);
  busy = $state(false);
  error = $state("");

  today = $state("");
  day = $state("");
  exeDir = $state("");
  dataFolder = $state<string | null>(null);
  // The program's shown version, for the title bar's brand text; the OS
  // window title is set from the same one-place text on the Rust side at start.
  version = $state("");
  overview = $state<dayData.DayOverview | null>(null);
  settings = $state<start.Settings | null>(null);
  about = $state<start.About | null>(null);
  dayList = $state<string[]>([]);
  screen = $state<Screen>("day");
  // Categories/Settings/Search open over the current screen and close back to it.
  overlay = $state<OverlayId | null>(null);
  // The set the Categories overlay opens on: the batch's own set when the
  // screen comes from the Plan screen's *Edit categories*; `null` from the top bar, where the
  // set used last is shown.
  categoriesGrouping = $state<string | null>(null);

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
  // Bumped after Categories saves, so the open Plan screen recounts.
  planRefresh = $state(0);

  // The batch number the next save will get (continuing the day).
  nextBatch = $derived((this.overview?.totals.batches ?? 0) + 1);
  // The batch number being prepared, for the Categories *In batch `<b>`: n* column; `null` when no
  // draft is open.
  draftBatch = $derived(this.draftOpen ? (this.readResult?.amend ?? this.nextBatch) : null);
  // The batch's own picks set, for the amend's selection page's fixed *Set* line;
  // `null` for a new batch's file list.
  amendGrouping = $derived(
    this.amendBatch !== null && this.readResult !== null ? this.readResult.plan.grouping : null,
  );

  // Start the shell: the data folder, the day, the settings and About.
  load = async () => {
    try {
      const info = await start.startInfo();
      this.today = info.today;
      this.day = info.today;
      this.exeDir = info.exe_dir;
      this.dataFolder = info.data_folder;
      this.version = info.version;
      this.needsFolder = info.data_folder === null;
      if (!this.needsFolder) {
        await this.afterFolder();
        await this.openCategoriesIfBroken();
      }
    } catch (e) {
      this.error = String(e);
    }
    this.ready = true;
  };

  private afterFolder = async () => {
    try {
      this.dayList = await dayData.days();
    } catch {
      this.dayList = [];
    }
    await this.refreshOverview();
    try {
      this.settings = await start.settingsGet();
    } catch (e) {
      this.error = String(e);
    }
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

  // A `categories.toml` with errors opens Categories first.
  private openCategoriesIfBroken = async () => {
    try {
      const rules = await categories.rulesLoad();
      if (rules.error) {
        this.categoriesGrouping = null;
        this.overlay = "categories";
      }
    } catch {
      // No categories file yet: nothing to open.
    }
  };

  useProgramFolder = async () => {
    this.busy = true;
    try {
      this.dataFolder = await start.useProgramFolder();
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
      this.needsFolder = false;
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
    this.savedMessage = texts.day.saved(result.batch, result.pdfs, result.packing_lists);
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

  // *Day* returns to the day screen; *Categories* and *Settings* open over the current screen.
  //
  navigate = async (id: NavId) => {
    if (id === "day") {
      if (!(await this.leaveBatch())) return;
      this.overlay = null;
      this.screen = "day";
    } else {
      // From the top bar: the Categories screen shows the set used last.
      if (id === "categories") this.categoriesGrouping = null;
      this.overlay = id;
    }
  };

  closeOverlay = () => {
    this.overlay = null;
  };

  // The Plan screen's *Edit categories*: open Categories on the set the batch
  // uses, so a change is written into that set's file.
  openCategoriesFromPlan = () => {
    this.categoriesGrouping = this.readResult?.plan.grouping ?? null;
    this.overlay = "categories";
  };

  onCategoriesSaved = () => {
    this.overlay = null;
    this.planRefresh += 1; // the open Plan screen recounts with the new categories
  };

  // *Revert* the newest batch: confirm, then delete it.
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
  // `download/` files again and open the *Redo batch `<b>`* Plan screen at once, the draft open.
  // The batch's file list is kept, so the Plan screen's *Back* shows the selection page with the
  // current selection listed. The read's
  // `read-progress` events show on the Day screen until the Plan screen opens.
  amend = async (row: dayData.BatchView) => {
    const asked = await confirm(texts.day.confirmAmend(row.batch, row.grouping, row.time), {
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

  saveSettings = async (value: start.Settings) => {
    try {
      await start.settingsSet(value);
      this.settings = value;
    } catch (e) {
      this.error = String(e);
    }
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

  // The **Printed** tick of one saved-PDF row: write it and
  // update the batch's rows in place, without reloading the whole overview. The tick rule is the
  // pure [`togglePrinted`]; a failed save shows the usual error.
  setPrinted = async (row: dayData.BatchView, pdf: dayData.PdfView, printed: boolean) => {
    try {
      await dayData.setPrinted(this.day, row.batch, pdf.number, printed);
      const ticked = row.pdfs.filter((item) => item.printed).map((item) => item.number);
      const numbers = togglePrinted(ticked, pdf.number, printed);
      for (const item of row.pdfs) item.printed = numbers.includes(item.number);
    } catch (e) {
      this.error = String(e);
    }
  };
}
