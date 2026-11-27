// Typed wrappers around the Tauri commands for archiving (archive.md) and Settings › Storage. The
// Rust side sends snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// The dates an archive is asked for: a range (one date is a range of one), or the dates more than
// `days` days before today.
export type ArchiveScope =
  | { kind: "range"; from: string; to: string }
  | { kind: "older_than"; days: number };

// What archiving the asked dates would delete; today, a date with no batch and an archived date
// are passed over.
export interface ArchivePreview {
  // Oldest first.
  dates: string[];
  pdfs: number;
  bytes: number;
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

// Settings › Storage › *Used*.
export interface StorageUsed {
  bytes: number;
  dates: number;
  // `null` while the search index is being built.
  archived: number | null;
}

export const archivePreview = (scope: ArchiveScope): Promise<ArchivePreview> =>
  invoke("archive_preview", { scope });

export const archiveScope = (scope: ArchiveScope): Promise<ArchiveOutcome> =>
  invoke("archive_scope", { scope });

export const onArchiveStep = (handler: (step: ArchiveStep) => void): Promise<UnlistenFn> =>
  listen<ArchiveStep>("archive-step", (event) => handler(event.payload));

export const storage = (): Promise<StorageUsed> => invoke("storage");

// *Archive automatically*: its number of days; `null`: off.
export const archiveAfterDays = (): Promise<number | null> => invoke("archive_after_days");

export const setArchiveAfterDays = (days: number | null): Promise<void> =>
  invoke("set_archive_after_days", { days });
