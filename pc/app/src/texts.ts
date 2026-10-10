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

  // 08 › *Window frame*: Categories (2.12) and New batch (2.9) are not built yet.
  comingSoon: {
    categories: "Categories: coming in 2.12.",
    newBatch: "New batch: coming in 2.9.",
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
    loading: "Loading…",
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
