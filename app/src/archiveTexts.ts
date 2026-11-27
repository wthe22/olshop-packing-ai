// The texts of archiving (archive.md) on the panel, the Day screen, Search and Settings › Storage,
// part of `texts` (`texts.archive`).

import type { Question } from "./lib/confirm.svelte";
import { count } from "./count";

export const archiveTexts = {
  // The panel's row menu item, the Settings button and the question's button.
  archive: "Archive",
  // The tooltip of an archived date's row on the panel.
  archivedRow: "Archived: the PDFs were removed.",
  // Under the Day screen's title.
  dayLine: "Archived: the PDFs of this date were removed.",
  // After a Search card's date.
  searchMark: "archived",
  // The dialog line while more than one date is archived.
  progress: (date: string, number: number, of: number) => `Archiving: ${date}, ${number} of ${of}`,

  // One date: "Archive Wed, 7 Oct 2026?"; several: their number, the first and the last.
  ask: (dates: string[], pdfs: number, size: string): Question =>
    dates.length === 1
      ? {
          title: `Archive ${dates[0]}?`,
          body: `Its ${count(pdfs, "PDF", "PDFs")} (${size}) are deleted. Its orders and summaries stay; it can no longer be printed, amended, reverted, reprinted or read again.`,
          ok: "Archive",
        }
      : {
          title: `Archive ${dates.length} dates, ${dates[0]} to ${dates[dates.length - 1]}?`,
          body: `Their ${count(pdfs, "PDF", "PDFs")} (${size}) are deleted. Their orders and summaries stay; they can no longer be printed, amended, reverted, reprinted or read again.`,
          ok: "Archive",
        },
  // The panel's **Archive** on a date that cannot be archived now.
  nothing: "This date is not archived now: an operation of it did not finish. Open it first.",
  // PDFs Windows refused to delete.
  left: (files: string[]) =>
    `Not deleted, open in another program: ${files.join(", ")}. Close them and archive again.`,

  storage: "Storage",
  used: "Used",
  usedHint: "The data folder: every date and its files.",
  usedValue: (size: string, dates: number, archived: number | null) =>
    `${size} in ${count(dates, "date", "dates")}${archived === null ? "" : ` · ${archived} archived`}`,
  archiveDates: "Archive dates",
  archiveDatesHint: "Their PDFs are deleted; their orders and summaries stay.",
  from: "From",
  to: "To",
  rangeValue: (dates: number, size: string) => `${count(dates, "date", "dates")} · ${size}`,
  auto: "Archive automatically",
  autoHint: "At each start, the dates older than this are archived.",
  olderThan: "Dates older than",
  days: "days",
  off: "Off",
  on: "On",
} as const;
