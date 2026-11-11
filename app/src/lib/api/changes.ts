// Typed wrapper around the Tauri command that looks for what other copies of the app wrote to the
// data folder. The Rust side sends snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

export interface DataChanges {
  // The open date changed: the Day screen reads it again.
  day: boolean;
  // A date came or went, or a journal changed: the panel reads its list again.
  panel: boolean;
}

export const dataChanges = (): Promise<DataChanges> => invoke("data_changes");
