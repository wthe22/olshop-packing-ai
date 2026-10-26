// Typed wrappers around the Tauri commands for the start, the first namespace's name, the shell's file
// opens and *About*. The Rust side sends snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";
import type { Theme } from "../theme";

export interface StartInfo {
  exe_dir: string;
  // The data folder `packing-settings.toml` names; null until one is chosen.
  data_folder: string | null;
  // The namespace open; null until the first one is named.
  namespace: string | null;
  pdfium_ready: boolean;
  today: string;
  // The program's shown version (`v` + major.minor); the title bar's brand text is the program's
  // name and this.
  version: string;
  // The theme `packing-settings.toml` names.
  theme: Theme;
}

// Settings › *About*.
export interface About {
  version: string;
  pdfium: string;
  schema: string;
}

export const startInfo = (): Promise<StartInfo> => invoke("start_info");

export const useProgramFolder = (): Promise<string> =>
  invoke("use_program_folder");

export const chooseDataFolder = (path: string): Promise<string> =>
  invoke("choose_data_folder", { path });

// The first namespace's name: its folder is made at once and opened.
export const namespaceNew = (name: string): Promise<string> =>
  invoke("namespace_new", { name });

export const openPath = (path: string): Promise<void> =>
  invoke("open_path", { path });

export const showInFolder = (path: string): Promise<void> =>
  invoke("show_in_folder", { path });

// The theme menu: stored in `packing-settings.toml`; the OS title bar follows it.
export const setTheme = (theme: Theme): Promise<void> => invoke("set_theme", { theme });

export const about = (): Promise<About> => invoke("about");
