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
