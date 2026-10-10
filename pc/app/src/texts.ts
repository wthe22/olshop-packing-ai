// Every screen text of the PC app, in English (07 › *Settings*: one file so another language
// can be added later). Components import from here; no screen text is written in a component.

function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

export const texts = {
  title: "Packing",

  nav: {
    day: "Day",
    categories: "Categories",
    settings: "Settings",
    today: "Today",
    previousDay: "Previous day",
    nextDay: "Next day",
  },

  // 08 › *Window frame*: Categories (2.12) is not built yet.
  comingSoon: {
    categories: "Categories: coming in 2.12.",
  },

  start: {
    // 08 › *Start* step 2: "No packing data next to the program (`D:\Packing\`)."
    noData: (folder: string) => `No packing data next to the program (${folder}).`,
    useThisFolder: "Use this folder",
    chooseFolder: "Choose a folder…",
  },

  day: {
    // 08 › *1. Day* header: "Today: 2 batches · 729 orders · 5 saved PDFs".
    summary: (batches: number, orders: number, pdfs: number) =>
      `Today: ${count(batches, "batch", "batches")} · ${count(orders, "order", "orders")} · ${count(pdfs, "saved PDF", "saved PDFs")}`,
    nothing: "Nothing saved on this day yet.",
    newBatch: "+ New batch",
    // 08 › *1. Day* card: "Batch 2 · saved 09:12 · 2 files · 129 orders".
    batch: (n: number, time: string, files: number, orders: number) =>
      `Batch ${n} · saved ${time} · ${count(files, "file", "files")} · ${count(orders, "order", "orders")}`,
    open: "Open",
    openFolder: "Open folder",
    packingList: "Packing list",
    // 08 › *1. Day* PDF row: "88 orders   3 runs".
    counts: (orders: number, runs: number) =>
      `${count(orders, "order", "orders")}   ${count(runs, "run", "runs")}`,
    // 08 › *1. Day* card: the batch's stored warnings, expanding to the texts.
    warnings: (n: number) => `Warnings: ${n} ▸`,
    warningsNone: "Warnings: none",
    // 08 › *1. Day*: the green line after *Save batch*, shown for a few seconds.
    saved: (n: number, pdfs: number, lists: number) =>
      `Batch ${n} saved: ${count(pdfs, "PDF", "PDFs")}, ${count(lists, "packing list", "packing lists")}.`,
    loading: "Loading…",
  },

  // 08 › *2. New batch*.
  newBatch: {
    title: (n: number) => `New batch ${n}`,
    cancel: "Cancel",
    dropHint: "Drop label PDFs here, or",
    addFiles: "Add files…",
    pages: (n: number) => `${count(n, "page", "pages")}`,
    usedInBatch: (n: number) => `used in batch ${n}`,
    remove: "×",
    removeTitle: "Remove this file",
    selected: (files: number, pages: number) =>
      `Selected: ${count(files, "file", "files")} · ${count(pages, "page", "pages")}`,
    read: "Read labels",
    // 08 › *Reading*: "Reading labels …  page 143 of 257 (file 1 of 2)".
    reading: (page: number, pages: number, file: number, files: number) =>
      `Reading labels …  page ${page} of ${pages} (file ${file} of ${files})`,
    back: "Back",
  },

  // 08 › *3. Plan*.
  plan: {
    // 08 › *3. Plan* header: "Batch 3 · 2 files · 257 pages · 256 orders · 0 already saved".
    header: (
      batch: number,
      files: number,
      pages: number,
      orders: number,
      alreadySaved: number,
    ) =>
      `Batch ${batch} · ${count(files, "file", "files")} · ${count(pages, "page", "pages")} · ${count(orders, "order", "orders")} · ${alreadySaved} already saved`,
    warnings: (n: number) => `Warnings: ${n} ▸`,
    warningsNone: "Warnings: none",
    use: "Use",
    hash: "#",
    pick: "Pick",
    orders: "Orders",
    runs: "Runs",
    fileName: "File name",
    rest: "(the rest)",
    // 08 › *3. Plan*: a pick that takes 0 orders.
    noPdf: "0 orders — no PDF",
    skipped: "skipped for this batch",
    moveUp: "Move up",
    moveDown: "Move down",
    expand: "Runs",
    collapse: "Hide runs",
    addPick: "+ Add a pick for this batch",
    addCode: "Code",
    addName: "Name",
    addCondition: "Condition",
    addSubmit: "Add",
    addCancel: "Cancel",
    // 08 › *3. Plan*: the condition is typed for now (the boxes editor is 2.12).
    conditionNote:
      "Type the condition (the boxes editor comes in 2.12). Example: name contains \"sepatu\"",
    packingList: "Packing list:",
    scopeWhole: "one for the batch",
    scopePerPdf: "one per PDF",
    scopeNone: "none",
    layoutFull: "full",
    layoutSummary: "summary",
    layoutPick: "pick",
    save: "Save batch",
    // 08 › *3. Plan*: the second of two PDFs with the same name.
    duplicate: (n: number) => `Same name as PDF ${n}`,
    cancel: "Cancel",
    confirmDiscard: (n: number) =>
      `Discard batch ${n}? Nothing has been saved.`,
  },

  settings: {
    title: "Settings",
    dataFolder: "Data folder",
    change: "Change…",
    open: "Open",
    // 08 › *6. Settings*: "Use `<folder>` until the app closes? Existing days stay in `<old>`."
    confirmChange: (folder: string, old: string) =>
      `Use ${folder} until the app closes? Existing days stay in ${old}.`,
    packingList: "Packing list default",
    scope: "Scope",
    scopeWhole: "one for the batch",
    scopePerPdf: "one per PDF",
    scopeNone: "none",
    layouts: "Layouts",
    layoutFull: "full",
    layoutSummary: "summary",
    layoutPick: "pick",
    save: "Save",
    about: "About",
    version: (version: string) => `Version ${version}`,
    // 08 › *6. Settings*: the PDFium version (D14: version + PDFium version only).
    pdfium: (version: string) => `PDFium ${version}`,
  },
} as const;

// 08 › *Window frame*: the day switch shows e.g. "Wed 7 Oct 2026". Derived from the date, so it
// is not a fixed screen text; kept here with the other texts.
const dayFormat = new Intl.DateTimeFormat("en-GB", {
  weekday: "short",
  day: "numeric",
  month: "short",
  year: "numeric",
});

export function formatDay(day: string): string {
  return dayFormat.format(new Date(`${day}T00:00:00`));
}
