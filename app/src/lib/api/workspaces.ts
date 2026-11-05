// Typed wrappers around the Tauri commands for Settings › Workspaces. The Rust side sends
// snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceView {
  name: string;
  // Its dates that have a folder.
  dates: number;
  open: boolean;
  // The shops whose labels belong in it; empty when nothing is checked.
  shops: string[];
}

// Every workspace of the data folder, in name order.
export const workspacesOverview = (): Promise<WorkspaceView[]> => invoke("workspaces_overview");

// The open workspace stays open under its new name; an open draft is dropped.
export const workspaceRename = (name: string, newName: string): Promise<void> =>
  invoke("workspace_rename", { name, newName });

// Writes the workspace's shops; resolves to the names as written (trimmed).
export const workspaceSetShops = (name: string, shops: string[]): Promise<string[]> =>
  invoke("workspace_set_shops", { name, shops });

// Moves the workspace to the Recycle Bin. Resolves to the workspace open afterwards; `null` when
// none is left.
export const workspaceDelete = (name: string): Promise<string | null> =>
  invoke("workspace_delete", { name });
