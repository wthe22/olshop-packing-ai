// Typed wrappers around the Tauri commands of the Archive page (ui/archive.md) and of archiving at
// start. The Rust side sends snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// The dates an archive is asked for: the ticked dates, or the dates more than `days` days before
// today.
export type ArchiveScope = { kind: "dates"; dates: string[] } | { kind: "older_than"; days: number };

// What archiving the asked dates would delete; today, a date with no batch and an archived date
// are passed over.
export interface ArchivePreview {
  // Oldest first.
  dates: string[];
  pdfs: number;
  bytes: number;
}

// What archiving one date would delete.
export interface Pdfs {
  pdfs: number;
  bytes: number;
}

// One date of the Archive page.
export interface ArchiveRow {
  date: string;
  today: boolean;
  // Its saved orders; `null` while the search index is being built.
  orders: number | null;
  // Every file of its folder together, in bytes.
  size_bytes: number;
  archived: boolean;
  // What archiving it would delete; `null`: it cannot be archived, and has no tick.
  archive: Pdfs | null;
}

export interface ArchivePage {
  // The search index is still being built: no orders yet.
  loading: boolean;
  // The data folder's size.
  total_bytes: number;
  // Newest first.
  dates: ArchiveRow[];
}

// The date being archived (the `archive-step` event).
export interface ArchiveStep {
  date: string;
  number: number;
  of: number;
}

// A PDF Windows refused to delete, most often open in a viewer.
export interface LeftPdf {
  date: string;
  file: string;
}

export interface ArchiveOutcome {
  // The dates archived now.
  archived: string[];
  left: LeftPdf[];
}

export const archivePage = (): Promise<ArchivePage> => invoke("archive_page");

export const archivePreview = (scope: ArchiveScope): Promise<ArchivePreview> =>
  invoke("archive_preview", { scope });

export const archiveScope = (scope: ArchiveScope): Promise<ArchiveOutcome> =>
  invoke("archive_scope", { scope });

export const onArchiveStep = (handler: (step: ArchiveStep) => void): Promise<UnlistenFn> =>
  listen<ArchiveStep>("archive-step", (event) => handler(event.payload));

// *Automatic*: its number of days; `null`: off.
export const archiveAfterDays = (): Promise<number | null> => invoke("archive_after_days");

export const setArchiveAfterDays = (days: number | null): Promise<void> =>
  invoke("set_archive_after_days", { days });
