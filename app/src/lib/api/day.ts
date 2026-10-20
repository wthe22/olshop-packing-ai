// Typed wrappers around the Tauri commands for the Day screen (07 › *Commands between window and
// Rust*). The Rust side sends snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

export interface PdfView {
  number: number;
  code: string;
  name: string;
  file: string;
  orders: number;
  runs: number;
  // The row's **Printed** tick, from the batch's `printed` list (07 › *`state.json`*;
  // 08 › *1. Day*): a note only.
  printed: boolean;
}

export interface BatchView {
  batch: number;
  save_time: string;
  time: string;
  // The grouping the batch was packed with (04 › *Sets*; 07 › *`state.json`*), recorded at
  // save; **Amend** reports it in its confirmation and reads that grouping again. An older
  // `state.json` without it reads as `default`.
  grouping: string;
  files: number;
  orders: number;
  pdfs: PdfView[];
  packing_lists: string[];
  warnings: string[];
  // The check before saving's errors, for *Saved with `<n>` errors ▸* (08 › *1. Day*);
  // empty unless the batch was saved anyway (07 › *Check before saving*).
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

export const dayOverview = (day: string): Promise<DayOverview> =>
  invoke("day_overview", { day });

// The **Printed** tick of one saved-PDF row of a batch (08 › *1. Day*): writes the day's
// `state.json`; it changes nothing else. An unknown batch or saved-PDF number changes nothing.
export const setPrinted = (
  day: string,
  batch: number,
  pdf: number,
  printed: boolean,
): Promise<void> => invoke("set_printed", { day, batch, pdf, printed });

export const days = (): Promise<string[]> => invoke("days");

// *Revert* the last batch (08 › *1. Day*): delete its folder and forget it. A file open in a PDF
// viewer stops with the 08 *file in use* message and changes nothing.
export const revertLast = (day: string): Promise<void> =>
  invoke("revert_last", { day });
