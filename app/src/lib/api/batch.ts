// Typed wrappers around the Tauri commands for the New batch selection page and the Plan screen.
// The Rust side sends snake_case fields; the
// interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

// One chosen label file as *New batch* shows it.
export interface FileCheck {
  path: string;
  name: string;
  // The bytes' SHA-256: a file chosen again from another folder is the same file.
  sha256: string;
  size: number;
  // `HH:MM` of the file's modified time (its download time).
  time: string;
  pages: number;
  used_in_batch: number | null;
}

// One order content of a group, for the expanded row: its items and how many orders have it.
export interface RunView {
  contents: string;
  orders: number;
}

// The fields every Plan row carries.
export interface PlanRowFields {
  code: string;
  name: string;
  condition: string | null;
  orders: number;
  runs: RunView[];
  pages: number;
  file_name: string;
  skipped: boolean;
  // The code of the earlier row whose file has the same name.
  duplicate_of: string | null;
  can_move_up: boolean;
  can_move_down: boolean;
}

// One row of the Plan screen: a plan entry the window edits through `index`, the
// plan's own rest entry (an entry too), or the generated `? Uncategorised` row (no index, no edits).
export type PlanRow =
  | (PlanRowFields & { kind: "entry"; index: number })
  | (PlanRowFields & { kind: "rest"; index: number })
  | (PlanRowFields & { kind: "generated" });

// The Plan screen's data.
export interface PlanView {
  rows: PlanRow[];
  has_duplicate_names: boolean;
  add_at: number;
  // The sort rule is one group that takes every order: one group PDF.
  no_groups: boolean;
  // The sort rule's name: the one chosen on the New batch screen, or the batch's own for an
  // amend.
  sort_rule: string;
}

// One unseen logo as the logo prompt shows it: the
// image's hash, how many pages and orders carry it, and the logo as printed — a PNG data URL,
// `null` when no page could be read (the screen shows the hash then).
export interface LogoView {
  hash: string;
  pages: number;
  orders: number;
  picture: string | null;
}

// What reading ends with.
export interface ReadResult {
  files: number;
  pages: number;
  orders: number;
  total_orders: number;
  already_saved: number;
  warnings: string[];
  // The batch being redone when this read is an amend; `null` for a new batch.
  amend: number | null;
  // The courier names the prompt's buttons show, in `couriers.toml` order.
  couriers: string[];
  // The header-band images the courier data does not know; empty when every logo is
  // known and no prompt is needed.
  logos: LogoView[];
  // The orders that appear more than once; `null` when every order has one copy.
  repeats: RepeatsView | null;
  plan: PlanView;
}

// Repeated orders: the groups of files that repeat each other, and the copy of each order that
// stays.
export interface RepeatsView {
  orders: number;
  extra_copies: number;
  extra_pages: number;
  // Files that hold a repeat, of all the batch's files.
  files: number;
  all_files: number;
  groups: RepeatGroupView[];
}

export interface RepeatGroupView {
  number: number;
  files: RepeatFileView[];
  choice: ChoiceView;
  // An order of the group has its copy chosen by hand.
  by_hand: boolean;
  // *Take file f out*, for each file whose repeated orders all have a copy in another file.
  take_out: { file: number; only_here: number }[];
  orders: RepeatOrderView[];
}

// One file of a group, by its number in the batch (from 1).
export interface RepeatFileView {
  file: number;
  name: string;
  pages: number;
  copies: number;
  kept: number;
  left_out: number;
}

export interface RepeatOrderView {
  order_id: string;
  tracking_id: string | null;
  // `null`: every page of every copy prints the same.
  difference: "tracking_id" | "line_count" | "page_count" | "pages" | null;
  copies: { copy: number; file: number; page_start: number }[];
  // The copy that stays, by its `copy`.
  kept: number;
  by_hand: boolean;
}

export type ChoiceView =
  | { kind: "keep_first" }
  | { kind: "keep_last" }
  | { kind: "take_out"; file: number };

// The owner's choices: one per group, and the orders chosen by hand.
export interface RepeatChoices {
  groups: { group: number; choice: ChoiceView }[];
  by_hand: { order_id: string; copy: number }[];
}

// What *Amend* opens with: the batch being redone, its `download/`
// files (the selection page's starting list, in the `state.json` order) and the read of those
// files as an amendment — the Plan screen shows `read` at once, the draft open.
export interface AmendStart {
  batch: number;
  files: FileCheck[];
  // The read of the batch's own files, shaped as `readLabels` returns for an amend.
  read: ReadResult;
}

// The owner's answer about one unseen logo, as Rust's `LogoAnswer`
// reads it (`kind` is the tag). *Skip all* is the screen's alone: it sends nothing.
export type LogoAnswer =
  | { kind: "courier"; name: string }
  | { kind: "not_courier" }
  | { kind: "skip" };

// What *Save batch* wrote, and the check before saving's findings when it
// stopped or the owner chose *Save anyway*.
export interface SaveResult {
  batch: number;
  pdfs: number;
  // Whether the batch was written; `false` when the check found something and *Save anyway* was
  // not chosen. The Plan screen then keeps the draft and shows the findings dialog.
  saved: boolean;
  // The check's errors (the dialog's first list, kept apart from the warnings).
  errors: string[];
  // The check's warnings (the dialog's second list).
  warnings: string[];
  // What failed after the commit: the batch is saved, and the next open settles it.
  after_save: string[];
}

// The `read-progress` event payload.
export interface ReadProgress {
  page: number;
  pages: number;
}

// The `save-progress` event payload: the stage, with the steps done of all steps.
export interface SaveProgress {
  stage: "CopyingInputs" | "Writing" | "Verifying" | "Saving";
  page: number;
  pages: number;
}

// A plan change from the window.
export type PlanEdit =
  | { kind: "set_skipped"; index: number; skipped: boolean }
  | { kind: "move_up"; index: number }
  | { kind: "move_down"; index: number }
  | { kind: "add"; at: number; code: string; name: string; condition: string }
  | { kind: "rename"; index: number; name: string };

// `redo` is the batch an amend's selection page redoes: a file is never *used in* it.
export const checkFiles = (paths: string[], redo: number | null): Promise<FileCheck[]> =>
  invoke("check_files", { paths, redo });

// *Read labels*: read the chosen files and open the Plan screen. `sortRule` is the rule chosen on
// the file list; `redo` is the batch being redone when the read comes from the amend's selection
// page, `null` for a new batch — an amend reads with that batch's rule, whatever `sortRule` says.
// The selection is read exactly as it stands: the batch's own inputs in their recorded order, an
// added file where the owner put it, a file taken off the list not read at all.
export const readLabels = (
  paths: string[],
  sortRule = "default",
  redo: number | null = null,
): Promise<ReadResult> => invoke("read_labels", { paths, sortRule, redo });

export const cancelRead = (): Promise<void> => invoke("cancel_read");

// Answer the logo prompt about one unseen logo: the back end
// applies it to the open draft and returns the refreshed read result.
export const answerLogo = (hash: string, answer: LogoAnswer): Promise<ReadResult> =>
  invoke("answer_logo", { hash, answer });

export const checkCondition = (condition: string): Promise<void> =>
  invoke("check_condition", { condition });

export const plan = (edits: PlanEdit[]): Promise<PlanView> =>
  invoke("plan", { edits });

// Apply the Repeated orders choices to the open draft; the read as it now is.
export const resolveRepeats = (choices: RepeatChoices): Promise<ReadResult> =>
  invoke("resolve_repeats", { choices });

// *Save batch*: `saveAnyway` is the findings dialog's *Save anyway*, `false`
// (the default) for a normal save.
export const saveBatch = (saveAnyway = false): Promise<SaveResult> =>
  invoke("save_batch", { saveAnyway });

export const discardDraft = (): Promise<void> => invoke("discard_draft");

// *Amend* the newest batch: read the batch's own inputs (`input\`) again and return
// its file list with the read, so the Plan screen opens at once with the draft open. Nothing is
// written until **Save batch `<b>` again** (`saveBatch`); the list is what the Plan screen's
// *Back* shows on the selection page.
export const startAmend = (day: string): Promise<AmendStart> =>
  invoke("start_amend", { day });
