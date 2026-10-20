// Typed wrappers around the Tauri commands for the New batch selection page and the Plan screen
// (07 › *Commands between window and Rust*). The Rust side sends snake_case fields; the
// interfaces here match it.

import { invoke } from "@tauri-apps/api/core";
import type { Layout, Scope } from "./start";

// One chosen label file as *New batch* shows it (08 › *2. New batch*).
export interface FileCheck {
  path: string;
  name: string;
  size: number;
  // `HH:MM` of the file's modified time (its download time); `null` when it cannot be read.
  time: string | null;
  pages: number;
  used_in_batch: number | null;
}

// One run inside a saved PDF, for the expanded row (08 › *3. Plan*).
export interface RunView {
  label: string;
  contents: string;
  orders: number;
}

// The fields every Plan row carries (08 › *3. Plan*).
export interface PlanRowFields {
  number: number | null;
  code: string;
  name: string;
  condition: string | null;
  orders: number;
  runs: RunView[];
  pages: number;
  file_name: string;
  skipped: boolean;
  duplicate_of: number | null;
  can_move_up: boolean;
  can_move_down: boolean;
}

// One row of the Plan screen (08 › *3. Plan*): a plan entry the window edits through `index`, the
// plan's own rest entry (an entry too), or the generated `? Uncategorised` row (no index, no edits).
export type PlanRow =
  | (PlanRowFields & { kind: "entry"; index: number })
  | (PlanRowFields & { kind: "rest"; index: number })
  | (PlanRowFields & { kind: "generated" });

// The Plan screen's data (08 › *3. Plan*).
export interface PlanView {
  rows: PlanRow[];
  has_duplicate_names: boolean;
  add_at: number;
  // The plan is the one catch-all entry, so the picks are unconfigured: the screen shows the
  // *No picks yet* line (Q-UI-4).
  no_categories: boolean;
  // The grouping the plan belongs to (04 › *Sets*): the grouping chosen on the New batch
  // screen, or the batch's own grouping for an amend. The header names it and **Edit
  // categories** opens it.
  grouping: string;
}

// One unseen logo as the logo prompt shows it (08 › *2. New batch* → *The logo prompt*): the
// image's hash, how many pages and orders carry it, and the logo as printed — a PNG data URL,
// `null` when no page could be read (the screen shows the hash then).
export interface LogoView {
  hash: string;
  pages: number;
  orders: number;
  picture: string | null;
}

// What reading ends with (08 › *Reading*).
export interface ReadResult {
  files: number;
  pages: number;
  orders: number;
  total_orders: number;
  already_saved: number;
  warnings: string[];
  // The batch being redone when this read is an amend (07 › *Amend*); `null` for a new batch.
  amend: number | null;
  // The courier names the prompt's buttons show, in `couriers.toml` order (D-LABEL-1).
  couriers: string[];
  // The header-band images the courier data does not know (D-LABEL-4); empty when every logo is
  // known and no prompt is needed.
  logos: LogoView[];
  plan: PlanView;
}

// What *Amend* opens with (08 › *1. Day* → *3. Plan*): the batch being redone, its `download/`
// files (the selection page's starting list, in the `state.json` order) and the read of those
// files as an amendment — the Plan screen shows `read` at once, the draft open.
export interface AmendStart {
  batch: number;
  files: FileCheck[];
  // The read of the batch's own files, shaped as `readLabels` returns for an amend.
  read: ReadResult;
}

// The owner's answer about one unseen logo (08 › *The logo prompt*), as Rust's `LogoAnswer`
// reads it (`kind` is the tag). *Skip all* is the screen's alone: it sends nothing.
export type LogoAnswer =
  | { kind: "courier"; name: string }
  | { kind: "not_courier" }
  | { kind: "skip" };

// What *Save batch* wrote (08 › *1. Day*), and the check before saving's findings when it
// stopped or the owner chose *Save anyway* (07 › *Check before saving*).
export interface SaveResult {
  batch: number;
  pdfs: number;
  packing_lists: number;
  // Whether the batch was written; `false` when the check found something and *Save anyway* was
  // not chosen. The Plan screen then keeps the draft and shows the findings dialog.
  saved: boolean;
  // The check's errors (the dialog's first list, kept apart from the warnings).
  errors: string[];
  // The check's warnings (the dialog's second list).
  warnings: string[];
}

// The `read-progress` event payload (07 › *Commands between window and Rust*).
export interface ReadProgress {
  page: number;
  pages: number;
}

// The `save-progress` event payload (07 › *Check before saving*): the stage in plain words is
// *Writing* or *Checking*, with the pages done of all pages.
export interface SaveProgress {
  stage: "Writing" | "Checking";
  page: number;
  pages: number;
}

// A plan change from the window (08 › *3. Plan*).
export type PlanEdit =
  | { kind: "set_skipped"; index: number; skipped: boolean }
  | { kind: "move_up"; index: number }
  | { kind: "move_down"; index: number }
  | { kind: "add"; at: number; code: string; name: string; condition: string }
  | { kind: "rename"; index: number; name: string };

// The packing-list choice sent to `save_batch` (07 › *Packing list*).
export interface PackingChoice {
  scope: Scope;
  layouts: Layout[];
}

export const checkFiles = (paths: string[]): Promise<FileCheck[]> =>
  invoke("check_files", { paths });

// *Read labels* (08 › *Reading*): read the chosen files and open the Plan screen. `grouping` is
// the grouping the batch uses, chosen on the selection page (04 › *Sets*); `default` for the data
// folder's own `categories.toml`. `redo` is the batch being redone when the read comes from the
// amend's selection page (08 › *2. New batch* → *The amend's selection page*); `null` for a new
// batch's file list — an amend reads with the grouping that batch recorded, whatever `grouping`
// says. The selection is read exactly as it stands: the batch's own `download/` copy in its
// stored order, an added file where the owner put it, a file taken off the list not read at all.
export const readLabels = (
  paths: string[],
  grouping = "default",
  redo: number | null = null,
): Promise<ReadResult> => invoke("read_labels", { paths, grouping, redo });

export const cancelRead = (): Promise<void> => invoke("cancel_read");

// Answer the logo prompt about one unseen logo (08 › *The logo prompt*; D-LABEL-4): the back end
// applies it to the open draft and returns the refreshed read result.
export const answerLogo = (hash: string, answer: LogoAnswer): Promise<ReadResult> =>
  invoke("answer_logo", { hash, answer });

export const checkCondition = (condition: string): Promise<void> =>
  invoke("check_condition", { condition });

export const plan = (edits: PlanEdit[]): Promise<PlanView> =>
  invoke("plan", { edits });

// *Save batch* (08 › *3. Plan*): `saveAnyway` is the findings dialog's *Save anyway*, `false`
// (the default) for a normal save (07 › *Check before saving*).
export const saveBatch = (
  packing: PackingChoice | null,
  saveAnyway = false,
): Promise<SaveResult> => invoke("save_batch", { packing, saveAnyway });

export const discardDraft = (): Promise<void> => invoke("discard_draft");

// *Amend* the last batch (08 › *1. Day*): read the batch's own `download/` files again and return
// its file list with the read, so the Plan screen opens at once with the draft open. Nothing is
// written until **Save batch `<b>` again** (`saveBatch`); the list is what the Plan screen's
// *Back* shows on the selection page.
export const startAmend = (day: string): Promise<AmendStart> =>
  invoke("start_amend", { day });
