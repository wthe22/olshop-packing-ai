// The Settings screen's texts, part of `texts` (`texts.settings`).

import type { Question } from "./lib/confirm.svelte";

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
  change: "Change",
  open: "Open",
  // A row only when a file holds keys this version does not know; each key on its own line.
  unknownKeys: "Kept, not read",
  unknownKeysHint: "Keys this version does not know. They stay in their file.",
  askChange: (folder: string, old: string): Question => ({
    title: "Change the data folder?",
    body: `New: ${folder}. The data in ${old} stays where it is.`,
    ok: "Change",
  }),

  about: "About",
  tagline:
    "Sorts the day's label PDFs into group PDFs to print, with a summary PDF to pick and pack by.",
  version: "Version",
  built: "Built",
  pdfium: "PDFium",
  schema: "Journal schema",
} as const;
