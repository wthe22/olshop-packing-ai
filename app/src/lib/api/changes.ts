// Typed wrapper around the Tauri command that looks for what other copies of the app wrote to the
// data folder. The Rust side sends snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

// The open workspace is no longer in the data folder: another copy renamed or deleted it.
export interface WorkspaceGone {
  workspace: string;
  // The workspace opened instead, as at start; `null` when no workspace is left.
  now_open: string | null;
}

export interface DataChanges {
  // The open date changed: the Day screen reads it again.
  day: boolean;
  // A workspace or a date came or went, or a journal changed: the panel reads its list again.
  panel: boolean;
  gone: WorkspaceGone | null;
}

export const dataChanges = (): Promise<DataChanges> => invoke("data_changes");
