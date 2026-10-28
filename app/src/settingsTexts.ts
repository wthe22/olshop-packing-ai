// The Settings screen's texts, part of `texts` (`texts.settings`).

import type { Question } from "./lib/confirm.svelte";
import { count } from "./count";

export const settingsTexts = {
  title: "Settings",

  appearance: "Appearance",
  theme: "Theme",
  themeHint: "Light, dark, or the same as Windows.",
  themes: {
    light: "Light",
    dark: "Dark",
    system: "System",
  },

  data: "Data",
  dataFolder: "Data folder",
  change: "Change…",
  open: "Open",
  askChange: (folder: string, old: string): Question => ({
    title: "Change the data folder?",
    body: `New: ${folder}. The data in ${old} stays where it is.`,
    ok: "Change",
  }),

  workspaces: "Workspaces",
  workspacesHint:
    "Each workspace keeps its own dates and batches. A new one is made with the + in the panel.",
  openNow: "Open now",
  dates: (n: number) => (n === 0 ? "No dates yet" : count(n, "date", "dates")),
  rename: "Rename…",
  renameLabel: (name: string) => `New name for ${name}`,
  save: "Rename",
  cancel: "Cancel",
  delete: "Delete…",
  askDelete: (name: string, dates: number): Question => ({
    title: `Delete workspace ${name}?`,
    body:
      dates === 0
        ? "Its folder goes to the Recycle Bin."
        : `Its ${count(dates, "date", "dates")}, with their labels and files, go to the Recycle Bin.`,
    ok: "Delete workspace",
  }),

  about: "About",
  tagline:
    "Sorts the day's label PDFs into group PDFs to print, with a summary PDF to pick and pack by.",
  version: "Version",
  pdfium: "PDFium",
  schema: "Journal schema",
} as const;
