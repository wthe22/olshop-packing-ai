// Typed wrappers around the Tauri commands for the start prompt, the shell's file opens and the
// Settings screen. The Rust side sends snake_case
// fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

export interface StartInfo {
  exe_dir: string;
  program_couriers: boolean;
  data_folder: string | null;
  pdfium_ready: boolean;
  today: string;
  // The program's shown version (`v` + major.minor); the title bar's brand text is the program's
  // name and this.
  version: string;
}

export type Scope = "whole" | "per-pdf" | "none";
export type Layout = "full" | "summary" | "pick";

export interface Settings {
  scope: Scope;
  layouts: Layout[];
  // The grouping used last: the New batch screen preselects it.
  // An older `settings.json` without it reads as `default`.
  grouping: string;
}

// Settings › *About*.
export interface About {
  version: string;
  pdfium: string;
}

export const startInfo = (): Promise<StartInfo> => invoke("start_info");

export const useProgramFolder = (): Promise<string> =>
  invoke("use_program_folder");

export const chooseDataFolder = (path: string): Promise<string> =>
  invoke("choose_data_folder", { path });

export const settingsGet = (): Promise<Settings> => invoke("settings_get");

export const settingsSet = (settings: Settings): Promise<void> =>
  invoke("settings_set", { settings });

export const openPath = (path: string): Promise<void> =>
  invoke("open_path", { path });

export const showInFolder = (path: string): Promise<void> =>
  invoke("show_in_folder", { path });

export const about = (): Promise<About> => invoke("about");
