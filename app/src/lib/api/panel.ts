// Typed wrappers around the Tauri commands for the side panel. The Rust side sends snake_case
// fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

// One date of the data folder.
export interface DateRow {
  date: string;
  today: boolean;
  // Its saved orders of every account, from the search index; `null` while the index is being
  // built.
  orders: number | null;
  // Its journal is damaged: it opens for viewing only.
  damaged: boolean;
  // The date's folder, for *Open folder*; `null` before its first commit.
  folder: string | null;
}

export interface PanelView {
  // Newest first; today is always listed.
  dates: DateRow[];
  // Today has a group PDF of a live batch not marked Printed yet.
  today_unprinted: boolean;
  // The search index is still being built: no counts yet.
  loading: boolean;
}

export const panelDates = (): Promise<PanelView> => invoke("panel_dates");
