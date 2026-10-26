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
  // Missing in `output\`, or not as its record says.
  missing: boolean;
}

export interface BatchView {
  batch: number;
  time: string;
  // Saved again by an amend.
  amended: boolean;
  sort_rule: string;
  files: number;
  orders: number;
  left_out: number;
  pdfs: GroupPdfView[];
  summary: string;
  summary_missing: boolean;
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
  namespace: string;
  batches: BatchView[];
  totals: Totals;
  // A damaged journal, an input missing or changed.
  problems: string[];
  // The data folder also holds files of Packing v0.1, which are not read.
  old_data: boolean;
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

// *Revert* the newest batch. A file open in a PDF viewer stops it with the file named.
export const revertLast = (day: string): Promise<void> =>
  invoke("revert_last", { day });
