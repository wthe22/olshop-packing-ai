// The words of the question at start when the data folder holds data this version keeps more of,
// of its progress, and of read-only mode (start.md, *Update the data folder?* and *Read-only*).

import { count } from "./count";

export const updateTexts = {
  title: "Update the data folder?",
  // The consequence of each answer, in plain words.
  explain: (batches: number, v07Dates: number) => {
    const moves = v07Dates > 0;
    return [
      moves
        ? `Update moves the files of ${count(v07Dates, "date", "dates")} out of the input and output folders of Packing v0.7 into their date folders.`
        : null,
      batches > 0
        ? `${moves ? "It" : "Update"} reads the receivers' addresses and the parcels' weights of ${count(batches, "batch", "batches")} from their label PDFs and writes the files that list each date again.`
        : `${moves ? "It" : "Update"} writes the files that list each date again.`,
      moves
        ? "Each date's update folder lists every file moved and keeps a copy of each file changed."
        : "Each date's update folder keeps a copy of each file changed.",
      "No batch, label or PDF is changed or deleted.",
    ]
      .filter((part) => part !== null)
      .join(" ");
  },
  readOnlyExplain: "Read-only shows the data as it is and changes nothing.",
  dateHead: "Date",
  changesHead: "What changes",
  // One date's row: "Files of v0.7 moved in · Addresses and weights of 2 batches, 61 orders · the
  // date's lists".
  changes: (batches: number, orders: number, olderViews: boolean, v07: boolean) =>
    [
      v07 ? "Files of v0.7 moved in" : null,
      batches > 0
        ? `Addresses and weights of ${count(batches, "batch", "batches")}, ${count(orders, "order", "orders")}`
        : null,
      olderViews || batches > 0 || v07 ? "the date's lists" : null,
    ]
      .filter((part) => part !== null)
      .join(" · "),
  update: "Update",
  readOnly: "Open read-only",
  continue: "Continue",
  // The step running and each date done.
  moving: (date: string, file: number, files: number) => `Moving files: ${date}, file ${file} of ${files}`,
  reading: (date: string, batch: number, page: number, pages: number) =>
    `Reading labels: ${date}, batch ${batch}, page ${page} of ${pages}`,
  updated: (date: string, moved: boolean, batches: number) =>
    `${date}: ${[
      moved ? "files moved" : null,
      batches > 0 ? `addresses and weights of ${count(batches, "batch", "batches")} read` : null,
      "lists written",
    ]
      .filter((part) => part !== null)
      .join(", ")}`,
  skipped: (batch: number, reason: string) => `Batch ${batch} not read: ${reason}`,
  done: "Done. The data folder is up to date.",
  doneSkipped: "Done. A batch not read is asked about again at the next start.",
  failed: (error: string) => `The update stopped: ${error}`,
  // The title bar's mark and the Day screen's line in read-only mode.
  pill: "Read-only",
  dayLine: "Read-only: this copy changes nothing in the data folder.",
};
