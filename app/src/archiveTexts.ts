// The texts of archiving (archive.md): the Archive page, its question and progress, and the marks
// of an archived date on the panel, the Day screen and Search; part of `texts` (`texts.archive`).

import type { Question } from "./lib/confirm.svelte";
import { count } from "./count";

export const archiveTexts = {
  // The page's title, the button of its foot and of its question.
  title: "Archive",
  archive: "Archive",
  // The tooltip of an archived date's row on the panel.
  archivedRow: "Archived: the PDFs were removed.",
  // Under the Day screen's title.
  dayLine: "Archived: the PDFs of this date were removed.",
  // After a Search card's date.
  searchMark: "archived",
  // The dialog line while more than one date is archived.
  progress: (date: string, number: number, of: number) => `Archiving: ${date}, ${number} of ${of}`,

  // "Archive 3 dates? Their PDFs (210 MB) are deleted; their orders and summaries stay."
  ask: (dates: number, size: string): Question =>
    dates === 1
      ? {
          title: "Archive 1 date?",
          body: `Its PDFs (${size}) are deleted; its orders and summaries stay.`,
          ok: "Archive",
        }
      : {
          title: `Archive ${dates} dates?`,
          body: `Their PDFs (${size}) are deleted; their orders and summaries stay.`,
          ok: "Archive",
        },
  // PDFs Windows refused to delete.
  left: (files: string[]) =>
    `Not deleted, open in another program: ${files.join(", ")}. Close them and archive again.`,

  dates: "Dates",
  // At the right of the Dates head: "3.2 GB in all".
  total: (size: string) => `${size} in all`,
  dateHead: "Date",
  ordersHead: "Orders",
  sizeHead: "Size",
  // An archived date's row.
  archived: "Archived",
  tickDate: (date: string) => `Tick ${date}`,
  tickMonth: (month: string) => `Tick the dates of ${month} that can be archived`,
  // The foot: "1 date · 72.3 MB of PDFs".
  foot: (dates: number, size: string) => `${count(dates, "date", "dates")} · ${size} of PDFs`,

  // Settings › Data: "Archive dates older than [60] days".
  olderThan: "Archive dates older than",
  days: "days",
  autoHint: "Their PDFs are deleted; their orders stay",
  off: "Off",
  on: "On",
} as const;
