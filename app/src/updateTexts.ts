// The words of the question at start when the data folder holds data this version keeps more of,
// of its progress, and of read-only mode (start.md, *Update the data folder?* and *Read-only*).

import { count } from "./count";

// A batch the Update did not read, as the line after it names it.
export interface SkippedBatch {
  date: string;
  batch: number;
  // `null`: its label PDF has changed; otherwise the error that stopped it.
  error: string | null;
}

export const updateTexts = {
  title: "Update the data folder?",
  // "29 dates were saved by an older Packing. ..."
  explain: (dates: number) =>
    `${count(dates, "date", "dates")} ${dates === 1 ? "was" : "were"} saved by an older Packing. Updating takes a few minutes and keeps a copy of every file it changes.`,
  update: "Update",
  readOnly: "Open read-only",
  continue: "Continue",
  // The line over the bar, and the step under it in grey.
  updating: (date: string, number: number, of: number) => `Updating ${date}, ${number} of ${of}`,
  moving: (file: number, files: number) => `Moving files: ${file} of ${files}`,
  reading: (page: number, pages: number) => `Reading labels: page ${page} of ${pages}`,
  // "1 batch not read: 2026-11-24 batch 2, its label PDF has changed."
  skipped: (batches: SkippedBatch[]) =>
    `${count(batches.length, "batch", "batches")} not read: ${batches
      .map(
        (one) =>
          `${one.date} batch ${one.batch}, ${one.error === null ? "its label PDF has changed" : one.error.replace(/\.$/, "")}`,
      )
      .join("; ")}.`,
  failed: (error: string) => `The update stopped: ${error}`,
  // The title bar's mark and the Day screen's line in read-only mode.
  pill: "Read-only",
  dayLine: "Read-only: this copy changes nothing in the data folder.",
};
