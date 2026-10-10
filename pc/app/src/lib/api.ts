// Typed wrappers around the Tauri commands (07 › *Commands between window and Rust*). The Rust
// side sends snake_case fields; the interfaces below match it.

import { invoke } from "@tauri-apps/api/core";

export interface StartInfo {
  exe_dir: string;
  program_categories: boolean;
  data_folder: string | null;
  pdfium_ready: boolean;
  today: string;
}

export interface PdfView {
  number: number;
  code: string;
  name: string;
  file: string;
  orders: number;
  runs: number;
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
  plan: PlanView;
}

/// What *Save batch* wrote, for the green line (08 › *1. Day*).
export interface SaveResult {
  batch: number;
  pdfs: number;
  packing_lists: number;
}

/// The `read-progress` event payload (07 › *Commands between window and Rust*).
export interface ReadProgress {
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

export const readLabels = (paths: string[]): Promise<ReadResult> =>
  invoke("read_labels", { paths });

export const cancelRead = (): Promise<void> => invoke("cancel_read");

export const checkCondition = (condition: string): Promise<void> =>
  invoke("check_condition", { condition });

export const plan = (edits: PlanEdit[]): Promise<PlanView> =>
  invoke("plan", { edits });

export const saveBatch = (packing: PackingChoice | null): Promise<SaveResult> =>
  invoke("save_batch", { packing });

export const discardDraft = (): Promise<void> => invoke("discard_draft");

// ------------------------------------------------------------------ Revert / Amend (2.11)

/// *Revert* the last batch (08 › *1. Day*): delete its folder and forget it. A file open in a PDF
/// viewer stops with the 08 *file in use* message and changes nothing.
export const revertLast = (day: string): Promise<void> =>
  invoke("revert_last", { day });

/// *Amend* the last batch (08 › *1. Day*): read its `download/` copy into a draft.
export const startAmend = (day: string): Promise<ReadResult> =>
  invoke("start_amend", { day });

// ------------------------------------------------------------------ Categories (2.12)

/// One category entry as the file holds it (08 › *5. Categories*).
export interface EntryView {
  code: string;
  name: string;
  when: string | null;
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

export const about = (): Promise<About> => invoke("about");
