// Typed wrappers around the Tauri commands for the side panel. The Rust side sends snake_case
// fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

// One date of the open workspace.
export interface DateRow {
  date: string;
  today: boolean;
  // Its saved orders from the search index; `null` while the index is being built.
  orders: number | null;
  // Its journal is damaged: it opens for viewing only.
  damaged: boolean;
  // The date's folder, for *Open folder*; `null` before its first commit.
  folder: string | null;
}

export interface PanelView {
  // Every workspace of the data folder, in name order.
  workspaces: string[];
  open: string | null;
  // Newest first; today is always listed.
  dates: DateRow[];
  // The search index is still being built: no counts yet.
  loading: boolean;
}

export const workspacesList = (): Promise<PanelView> => invoke("workspaces_list");

// Open another workspace on today's date; an open draft is dropped.
export const workspaceOpen = (name: string): Promise<void> => invoke("workspace_open", { name });
