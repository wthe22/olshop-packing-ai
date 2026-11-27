// Typed wrappers around the Tauri commands for the start, the shell's file
// opens and *About*. The Rust side sends snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Theme } from "../theme";

// A date the migration would change (start.md, *Update the data folder?*).
export interface MigrationView {
  date: string;
  // Its files are still in the input and output folders of v0.7 and are moved into the date folder.
  v0_7: boolean;
  // Its lists are of an older version and are written again.
  older_views: boolean;
  // The batches whose receivers' addresses are read, and their orders.
  batches: number;
  orders: number;
}

// A batch the Update could not read, and why.
export interface SkippedView {
  batch: number;
  reason: { kind: "label_changed" } | { kind: "failed"; error: string };
}

// A date the Update finished: whether its files of v0.7 were moved, its batches whose addresses were
// read, and those not read.
export interface DateUpdated {
  date: string;
  moved: boolean;
  batches: number;
  skipped: SkippedView[];
}

// What the Update sends as it runs (the `migration-step` event): the date it starts (`number` of
// `of`, from 1), the file of v0.7 being moved and the label page being read.
export type MigrationStep =
  | { kind: "starting"; date: string; number: number; of: number }
  | { kind: "moving"; date: string; file: number; files: number }
  | { kind: "reading"; date: string; batch: number; page: number; pages: number };

// The data folder chosen, and what the migration would change in it: while there is something,
// the folder is not opened until the owner answers.
export interface FolderChoice {
  data_folder: string;
  migration: MigrationView[];
  // The folder in use was chosen again and stays read-only.
  read_only: boolean;
}

export interface StartInfo {
  exe_dir: string;
  // The data folder `packing-app.toml` names; null until one is chosen.
  data_folder: string | null;
  // What the migration would change in it; no date opens until the owner answers.
  migration: MigrationView[];
  // The owner chose to look at the data folder without changing it.
  read_only: boolean;
  pdfium_ready: boolean;
  today: string;
  // The program's shown version (`v` + the version); the title bar's brand text is the program's
  // name and this.
  version: string;
  // The theme `packing-app.toml` names.
  theme: Theme;
}

// Settings › *About*.
export interface About {
  version: string;
  built: string;
  pdfium: string;
  schema: string;
}

export const startInfo = (): Promise<StartInfo> => invoke("start_info");

export const useProgramFolder = (): Promise<FolderChoice> => invoke("use_program_folder");

export const chooseDataFolder = (path: string): Promise<FolderChoice> =>
  invoke("choose_data_folder", { path });

// **Update**: the dates it finished; the folder is then open.
export const updateData = (): Promise<DateUpdated[]> => invoke("update_data");

// **Open read-only**.
export const openReadOnly = (): Promise<void> => invoke("read_only");

export const onMigrationStep = (handler: (step: MigrationStep) => void): Promise<UnlistenFn> =>
  listen<MigrationStep>("migration-step", (event) => handler(event.payload));

export const openPath = (path: string): Promise<void> =>
  invoke("open_path", { path });

export const showInFolder = (path: string): Promise<void> =>
  invoke("show_in_folder", { path });

// The theme menu: stored in `packing-app.toml`; the OS title bar follows it.
export const setTheme = (theme: Theme): Promise<void> => invoke("set_theme", { theme });

export const about = (): Promise<About> => invoke("about");

// Settings › *Data*: the keys of the program's files this version does not know, kept and not used.
export const unknownKeys = (): Promise<string[]> => invoke("unknown_keys");
