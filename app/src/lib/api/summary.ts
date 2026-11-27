import { invoke } from "@tauri-apps/api/core";

// A summary as the PDF prints it, part by part (the back end's `SummaryView`).
export interface SummaryView {
  // `Summary · All accounts · 2026-10-07 · batch 2`.
  title: string;
  // `129 orders · 141 units · 2 groups`.
  counts: string;
  parts: SummaryPart[];
}

// A number and the word it counts.
export interface Count {
  n: number;
  one: string;
  many: string;
}

// One ruled row: a bold key at the left, the text, up to two counts and a short hash at the right.
export interface FactRow {
  key: string | null;
  text: string;
  bold: boolean;
  counts: [Count | null, Count | null];
  tail: string | null;
}

// One order content: its number, its items as `name ×quantity`, and its orders.
export interface ContentRow {
  number: string;
  items: string[];
  orders: number;
}

export interface Pick {
  units: number;
  item: string;
}

export type SummaryPart =
  | { kind: "facts"; heading: string; right: string | null; rows: FactRow[] }
  | { kind: "contents"; heading: string; right: string; rows: ContentRow[] }
  | { kind: "picks"; heading: string; rows: Pick[] };

// The summary of `batch`, or of the date when `null`, of every account or of `account`. Nothing is
// written, so it is read in read-only mode and on an archived date too.
export const summaryView = (
  day: string,
  batch: number | null,
  account: string | null,
): Promise<SummaryView> => invoke("summary_view", { day, batch, account });
