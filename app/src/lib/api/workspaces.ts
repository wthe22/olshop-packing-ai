// Typed wrappers around the Tauri commands for Settings › Workspaces. The Rust side sends
// snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceView {
  name: string;
  // Its dates that have a folder.
  dates: number;
  open: boolean;
}

// Every workspace of the data folder, in name order.
export const workspacesOverview = (): Promise<WorkspaceView[]> => invoke("workspaces_overview");

// The open workspace stays open under its new name; an open draft is dropped.
export const workspaceRename = (name: string, newName: string): Promise<void> =>
  invoke("workspace_rename", { name, newName });

// Moves the workspace to the Recycle Bin. Resolves to the workspace open afterwards; `null` when
// none is left.
export const workspaceDelete = (name: string): Promise<string | null> =>
  invoke("workspace_delete", { name });
