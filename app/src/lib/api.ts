// Typed wrappers around the Tauri commands (07 › *Commands between window and Rust*). The Rust
// side sends snake_case fields; the interfaces below match it.

import { invoke } from "@tauri-apps/api/core";

export interface StartInfo {
  exe_dir: string;
  program_couriers: boolean;
  data_folder: string | null;
  pdfium_ready: boolean;
  today: string;
  /// The program's shown version (`v` + major.minor); the title bar's brand text is the program's
  /// name and this (08 › *Window frame*).
  version: string;
}

export interface PdfView {
  number: number;
  code: string;
  name: string;
  file: string;
  orders: number;
  runs: number;
  /// The row's **Printed** tick, from the batch's `printed` list (07 › *`state.json`*;
  /// 08 › *1. Day*): a note only.
  printed: boolean;
}

export interface BatchView {
  batch: number;
  save_time: string;
  time: string;
  files: number;
  orders: number;
  pdfs: PdfView[];
  packing_lists: string[];
  warnings: string[];
  /// The check before saving's errors, for *Saved with `<n>` errors ▸* (08 › *1. Day*);
  /// empty unless the batch was saved anyway (07 › *Check before saving*).
  errors: string[];
  folder: string;
}

export interface Totals {
  batches: number;
  orders: number;
  pdfs: number;
}

export interface DayOverview {
  day: string;
  batches: BatchView[];
  totals: Totals;
}

export type Scope = "whole" | "per-pdf" | "none";
export type Layout = "full" | "summary" | "pick";

export interface Settings {
  scope: Scope;
  layouts: Layout[];
}

/// The packing-list choice sent to `save_batch` (07 › *Packing list*).
export interface PackingChoice {
  scope: Scope;
  layouts: Layout[];
}

/// Settings › *About* (08 › *6. Settings*).
export interface About {
  version: string;
  pdfium: string;
}

export const startInfo = (): Promise<StartInfo> => invoke("start_info");

export const useProgramFolder = (): Promise<string> =>
  invoke("use_program_folder");

export const chooseDataFolder = (path: string): Promise<string> =>
  invoke("choose_data_folder", { path });

export const dayOverview = (day: string): Promise<DayOverview> =>
  invoke("day_overview", { day });

/// The **Printed** tick of one saved-PDF row of a batch (08 › *1. Day*): writes the day's
/// `state.json`; it changes nothing else. An unknown batch or saved-PDF number changes nothing.
export const setPrinted = (
  day: string,
  batch: number,
  pdf: number,
  printed: boolean,
): Promise<void> => invoke("set_printed", { day, batch, pdf, printed });

export const days = (): Promise<string[]> => invoke("days");

export const settingsGet = (): Promise<Settings> => invoke("settings_get");

export const settingsSet = (settings: Settings): Promise<void> =>
  invoke("settings_set", { settings });

export const openPath = (path: string): Promise<void> =>
  invoke("open_path", { path });

export const showInFolder = (path: string): Promise<void> =>
  invoke("show_in_folder", { path });

// -------------------------------------------------------------------- New batch / Plan

/// One chosen label file as *New batch* shows it (08 › *2. New batch*).
export interface FileCheck {
  path: string;
  name: string;
  size: number;
  time: string;
  pages: number;
  used_in_batch: number | null;
}

/// One run inside a saved PDF, for the expanded row (08 › *3. Plan*).
export interface RunView {
  label: string;
  contents: string;
  orders: number;
}

/// One row of the Plan screen (08 › *3. Plan*).
export interface PlanRow {
  index: number | null;
  number: number | null;
  code: string;
  name: string;
  condition: string | null;
  orders: number;
  runs: RunView[];
  pages: number;
  file_name: string;
  skipped: boolean;
  generated: boolean;
  is_rest: boolean;
  duplicate_of: number | null;
  can_use: boolean;
  can_move_up: boolean;
  can_move_down: boolean;
  empty: boolean;
}

/// The Plan screen's data (08 › *3. Plan*).
export interface PlanView {
  rows: PlanRow[];
  has_duplicate_names: boolean;
  add_at: number;
  /// The plan is the one catch-all entry, so the picks are unconfigured: the screen shows the
  /// *No picks yet* line (Q-UI-4).
  no_picks: boolean;
}

/// One unseen logo as the logo prompt shows it (08 › *2. New batch* → *The logo prompt*): the
/// image's hash, how many pages and orders carry it, and the logo as printed — a PNG data URL,
/// `null` when no page could be read (the screen shows the hash then).
export interface LogoView {
  hash: string;
  pages: number;
  orders: number;
  picture: string | null;
}

/// What reading ends with (08 › *Reading*).
export interface ReadResult {
  files: number;
  pages: number;
  orders: number;
  total_orders: number;
  already_saved: number;
  warnings: string[];
  /// The batch being redone when this read is an amend (07 › *Amend*); `null` for a new batch.
  amend: number | null;
  /// The courier names the prompt's buttons show, in `couriers.toml` order (D-LABEL-1).
  couriers: string[];
  /// The header-band images the courier data does not know (D-LABEL-4); empty when every logo is
  /// known and no prompt is needed.
  logos: LogoView[];
  plan: PlanView;
}

/// What *Amend* opens with (08 › *1. Day* → *3. Plan*): the batch being redone, its `download/`
/// files (the selection page's starting list, in the `state.json` order) and the read of those
/// files as an amendment — the Plan screen shows `read` at once, the draft open.
export interface AmendStart {
  batch: number;
  files: FileCheck[];
  /// The read of the batch's own files, shaped as `readLabels` returns for an amend.
  read: ReadResult;
}

/// The owner's answer about one unseen logo (08 › *The logo prompt*), as Rust's `LogoAnswerInput`
/// reads it (`kind` is the tag). *Skip all* is the screen's alone: it sends nothing.
export type LogoAnswer =
  | { kind: "courier"; name: string }
  | { kind: "not_courier" }
  | { kind: "skip" };

/// What *Save batch* wrote (08 › *1. Day*), and the check before saving's findings when it
/// stopped or the owner chose *Save anyway* (07 › *Check before saving*).
export interface SaveResult {
  batch: number;
  pdfs: number;
  packing_lists: number;
  /// Whether the batch was written; `false` when the check found something and *Save anyway* was
  /// not chosen. The Plan screen then keeps the draft and shows the findings dialog.
  saved: boolean;
  /// The check's errors (the dialog's first list, kept apart from the warnings).
  errors: string[];
  /// The check's warnings (the dialog's second list).
  warnings: string[];
}

/// The `read-progress` event payload (07 › *Commands between window and Rust*).
export interface ReadProgress {
  page: number;
  pages: number;
}

/// The `save-progress` event payload (07 › *Check before saving*): the stage in plain words is
/// *Writing* or *Checking*, with the pages done of all pages.
export interface SaveProgress {
  stage: "Writing" | "Checking";
  page: number;
  pages: number;
}

/// A plan change from the window (08 › *3. Plan*).
export type PlanEdit =
  | { kind: "set_skipped"; index: number; skipped: boolean }
  | { kind: "move_up"; index: number }
  | { kind: "move_down"; index: number }
  | { kind: "add"; at: number; code: string; name: string; condition: string }
  | { kind: "rename"; index: number; name: string };

export const checkFiles = (paths: string[]): Promise<FileCheck[]> =>
  invoke("check_files", { paths });

/// *Read labels* (08 › *Reading*): read the chosen files and open the Plan screen. `amend` is the
/// batch being redone when the read comes from the amend's selection page
/// (08 › *2. New batch* → *The amend's selection page*); `null` for a new batch's file list. The
/// selection is read exactly as it stands: the batch's own `download/` copy in its stored order,
/// an added file where the owner put it, a file taken off the list not read at all.
export const readLabels = (
  paths: string[],
  amend: number | null = null,
): Promise<ReadResult> => invoke("read_labels", { paths, amend });

export const cancelRead = (): Promise<void> => invoke("cancel_read");

/// Answer the logo prompt about one unseen logo (08 › *The logo prompt*; D-LABEL-4): the back end
/// applies it to the open draft and returns the refreshed read result.
export const answerLogo = (hash: string, answer: LogoAnswer): Promise<ReadResult> =>
  invoke("answer_logo", { hash, answer });

export const checkCondition = (condition: string): Promise<void> =>
  invoke("check_condition", { condition });

export const plan = (edits: PlanEdit[]): Promise<PlanView> =>
  invoke("plan", { edits });

/// *Save batch* (08 › *3. Plan*): `saveAnyway` is the findings dialog's *Save anyway*, `false`
/// (the default) for a normal save (07 › *Check before saving*).
export const saveBatch = (
  packing: PackingChoice | null,
  saveAnyway = false,
): Promise<SaveResult> => invoke("save_batch", { packing, saveAnyway });

export const discardDraft = (): Promise<void> => invoke("discard_draft");

// ------------------------------------------------------------------ Revert / Amend (2.11)

/// *Revert* the last batch (08 › *1. Day*): delete its folder and forget it. A file open in a PDF
/// viewer stops with the 08 *file in use* message and changes nothing.
export const revertLast = (day: string): Promise<void> =>
  invoke("revert_last", { day });

/// *Amend* the last batch (08 › *1. Day*): read the batch's own `download/` files again and return
/// its file list with the read, so the Plan screen opens at once with the draft open. Nothing is
/// written until **Save batch `<b>` again** (`saveBatch`); the list is what the Plan screen's
/// *Back* shows on the selection page.
export const startAmend = (day: string): Promise<AmendStart> =>
  invoke("start_amend", { day });

// ------------------------------------------------------------------ Categories (2.12)

/// One category entry as the file holds it (08 › *5. Categories*): its values and the index it has
/// in the loaded file (`origin`). The one catch-all entry of a folder without `categories.toml`
/// has `origin: null`, so saving it adds the entry and writes the file.
export interface EntryView {
  code: string;
  name: string;
  when: string | null;
  origin: number | null;
}

/// The Categories screen's first load (08 › *5. Categories*): the whole file text, the entries and
/// any file-level error.
export interface RulesLoad {
  text: string;
  entries: EntryView[];
  error: string | null;
}

/// One entry the screen sends back: its values and the index it had in the loaded file (`origin`),
/// so the save can keep its comment and unchanged condition text; `null` for a new entry.
export interface EntryInput {
  code: string;
  name: string;
  when: string | null;
  origin: number | null;
}

/// The live check (08 › *5. Categories*): each entry's condition error, a file-level error and the
/// count each entry takes in the open batch.
export interface RulesCheck {
  errors: (string | null)[];
  general: string | null;
  counts: number[] | null;
}

export const rulesLoad = (): Promise<RulesLoad> => invoke("rules_load");

export const rulesCheck = (entries: EntryInput[]): Promise<RulesCheck> =>
  invoke("rules_check", { entries });

export const rulesCheckText = (text: string): Promise<void> =>
  invoke("rules_check_text", { text });

export const rulesSave = (entries: EntryInput[]): Promise<void> =>
  invoke("rules_save", { entries });

export const rulesSaveText = (text: string): Promise<void> =>
  invoke("rules_save_text", { text });

/// A condition text → boxes tree (the engine's parser); an error carries line/col.
export const conditionToTree = (text: string): Promise<unknown> =>
  invoke("condition_to_tree", { text });

/// A boxes tree → the printer's canonical condition text.
export const treeToCondition = (tree: unknown): Promise<string> =>
  invoke("tree_to_condition", { tree });

// ------------------------------------------------------------------- Search (2.21)

/// One line of a Search row (08 › *Search*): the product name and its quantity.
export interface SearchItem {
  name: string;
  quantity: number;
}

/// One matching saved order as the Search panel shows it (08 › *Search*): its IDs, courier, items,
/// the batch it was saved in, its saved PDF (name, and the full `path` *Open* uses) and whether its
/// items were stored at all (an order saved before search existed).
export interface SearchRow {
  day: string;
  order_id: string;
  tracking_id: string;
  courier: string;
  items: SearchItem[];
  batch: number;
  pdf: number;
  file: string;
  path: string;
  items_not_stored: boolean;
}

/// What a search returns (08 › *Search*): the counts shown above the rows, then one row per
/// matching saved order.
export interface SearchResults {
  rows: SearchRow[];
  orders: number;
  batches: number;
}

/// Which day folders *search_orders* reads (Q-UI-3): `day` is *This day* (the day in the title
/// bar, the default), `all` is *All days*.
export type SearchScope = "day" | "all";

/// Search the saved orders for the condition boxes (08 › *Search*): the same tree as Categories
/// (`toRustTree`), with the scope. Reads `state.json` only; *Open* then uses `openPath`.
export const searchOrders = (
  conditions: unknown,
  scope: SearchScope,
): Promise<SearchResults> => invoke("search_orders", { conditions, scope });

export const about = (): Promise<About> => invoke("about");
