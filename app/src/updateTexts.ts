// The words of the question at start when the data folder holds data this version keeps more of,
// of its progress, and of read-only mode (start.md, *Update the data folder?* and *Read-only*).

import { count } from "./count";

export const updateTexts = {
  title: "Update the data folder?",
  // The consequence of each answer, in plain words.
  explain: (batches: number) =>
    batches > 0
      ? `Update reads the receivers' addresses of ${count(batches, "batch", "batches")} from their label PDFs and writes the files that list each date again. No batch, label or PDF is changed or deleted.`
      : "Update writes the files that list each date again. No batch, label or PDF is changed or deleted.",
  readOnlyExplain: "Read-only shows the data as it is and changes nothing.",
  dateHead: "Date",
  changesHead: "What changes",
  // One date's row: "Addresses of 2 batches, 61 orders · the date's lists".
  changes: (batches: number, orders: number, olderViews: boolean) =>
    [
      batches > 0
        ? `Addresses of ${count(batches, "batch", "batches")}, ${count(orders, "order", "orders")}`
        : null,
      olderViews || batches > 0 ? "the date's lists" : null,
    ]
      .filter((part) => part !== null)
      .join(" · "),
  update: "Update",
  readOnly: "Open read-only",
  continue: "Continue",
  // The step running and each date done.
  reading: (date: string, batch: number, page: number, pages: number) =>
    `Reading addresses: ${date}, batch ${batch}, page ${page} of ${pages}`,
  updated: (date: string, batches: number) =>
    batches > 0
      ? `${date}: addresses of ${count(batches, "batch", "batches")} read, lists written`
      : `${date}: lists written`,
  skipped: (batch: number, reason: string) => `Batch ${batch} not read: ${reason}`,
  done: "Done. The data folder is up to date.",
  doneSkipped: "Done. A batch not read is asked about again at the next start.",
  failed: (error: string) => `The update stopped: ${error}`,
  // The title bar's mark and the Day screen's line in read-only mode.
  pill: "Read-only",
  dayLine: "Read-only: this copy changes nothing in the data folder.",
};
