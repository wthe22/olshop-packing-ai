// Typed wrappers around the Tauri commands for the Day screen. The Rust side sends snake_case
// fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

export interface GroupPdfView {
  // The label group's code: the **Printed** tick names it.
  group: string;
  name: string;
  file: string;
  orders: number;
  contents: number;
  printed: boolean;
  problem: FileProblem | null;
}

// What the open-time check found of an output: *Open* writes a missing one again, **Restore** a
// changed one.
export type FileProblem = "missing" | "changed";

// One copy left out of a batch: where it is, and why.
export type LeftOutView = {
  order_id: string;
  input: string;
  page_start: number;
  page_end: number;
} & (
  | { reason: "repeat"; kept_input: string; kept_page: number }
  | { reason: "saved_earlier"; batch: number }
);

export interface BatchView {
  batch: number;
  time: string;
  // Saved again by an amend.
  amended: boolean;
  sort_rule: string;
  files: number;
  orders: number;
  left_out: LeftOutView[];
  pdfs: GroupPdfView[];
  summary: string;
  summary_problem: FileProblem | null;
  warnings: string[];
  // The verifier's errors: only on a batch saved anyway.
  errors: string[];
  // `output\`, for *Open* and *Open folder*.
  folder: string;
}

export interface Totals {
  batches: number;
  orders: number;
  pdfs: number;
}

export interface DayOverview {
  day: string;
  workspace: string;
  batches: BatchView[];
  totals: Totals;
  // A damaged journal, an input missing or changed.
  problems: string[];
  // The data folder also holds files of Packing v0.1, which are not read.
  old_data: boolean;
  reprints: ReprintRow[];
}

// One reprint made on the date.
export interface ReprintRow {
  number: number;
  // `HH:MM` when it was made.
  time: string;
  file: string;
  orders: number;
  pages: number;
  path: string;
}

// What **Restore** did: the file written again, or the verifier's findings (nothing kept).
export interface RestoreResult {
  restored: boolean;
  errors: string[];
}

export const dayOverview = (day: string): Promise<DayOverview> =>
  invoke("day_overview", { day });

// The **Printed** tick of one group PDF: a note in the date's journal.
export const setPrinted = (
  day: string,
  batch: number,
  group: string,
  printed: boolean,
): Promise<void> => invoke("set_printed", { day, batch, group, printed });

export const restoreOutput = (day: string, file: string): Promise<RestoreResult> =>
  invoke("restore_output", { day, file });

// What *Open* on a group PDF or summary PDF did: opened the file, wrote it again first because it
// was not there, or kept nothing because the written file did not pass the check.
export type OpenResult =
  | { kind: "opened" }
  | { kind: "written_again" }
  | { kind: "not_kept"; errors: string[] };

export const openOutput = (day: string, file: string): Promise<OpenResult> =>
  invoke("open_output", { day, file });

// *Day summary*: write the date's summary PDF from its journal; the file's path comes back. A day
// summary open in a PDF viewer that locks it stops the write with the file named.
export const writeDaySummary = (day: string): Promise<string> => invoke("day_summary", { day });

// *Revert* the newest batch. A file open in a PDF viewer stops it with the file named.
export const revertLast = (day: string): Promise<void> =>
  invoke("revert_last", { day });
