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
  // When **Print** opened its preview, "09:14"; `null` once the mark is taken back.
  sent: string | null;
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

// An account of a batch and its orders there.
export interface AccountOrders {
  account: string;
  orders: number;
}

export interface BatchView {
  batch: number;
  time: string;
  // Saved again by an amend.
  amended: boolean;
  sort_rule: string;
  files: number;
  orders: number;
  // Its accounts, most orders first.
  accounts: AccountOrders[];
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
  batches: BatchView[];
  totals: Totals;
  // A damaged journal, an input missing or changed.
  problems: string[];
  // Data of older versions in the data folder, which is not read.
  older: OlderData[];
  reprints: ReprintRow[];
}

// One older version's data as the Day screen's note names it.
export interface OlderData {
  // `v0.1` or `v0.4`.
  version: string;
  // The files and folders that show it; a folder's name ends in `\`.
  names: string[];
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

// **Print**: the output made ready like *Open* (written again when it is not there); the preview
// is the front end's, over `outputPdf`.
export const printOutput = (day: string, file: string): Promise<OpenResult> =>
  invoke("print_output", { day, file });

// The bytes of an output that is there.
export const outputPdf = (day: string, file: string): Promise<ArrayBuffer> =>
  invoke("output_pdf", { day, file });

// The *Sent* mark of a group PDF: set as its preview opens, taken back by its ✕.
export const setSent = (day: string, batch: number, group: string, sent: boolean): Promise<void> =>
  invoke("set_sent", { day, batch, group, sent });

// *Day summary*: write the date's summary PDF from its journal, of every account or of one; the
// file's path comes back. A day summary open in a PDF viewer that locks it stops the write with the
// file named.
export const writeDaySummary = (day: string, account: string | null): Promise<string> =>
  invoke("day_summary", { day, account });

// An account's summary of a batch: its path when kept, else the verifier's findings (nothing kept).
export interface AccountSummary {
  path: string | null;
  errors: string[];
}

export const accountSummaryPdf = (
  day: string,
  batch: number,
  account: string,
): Promise<AccountSummary> => invoke("account_summary_pdf", { day, batch, account });

// *Revert* `batch`, the newest; one that is no longer the newest (another copy of the app saved
// since) is refused. A file open in a PDF viewer stops it with the file named.
export const revertLast = (day: string, batch: number): Promise<void> =>
  invoke("revert_last", { day, batch });
