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
    // 08 › *1. Day*: *Amend* and *Revert*, on the newest batch only.
    amend: "Amend",
    revert: "Revert",
    undoBatch: (n: number) => `Undo batch ${n}`,
    redoBatch: (n: number) => `Redo batch ${n}`,
    // 08 › *1. Day*: the Revert confirmation, verbatim.
    confirmRevert: (n: number, pdfs: number, orders: number) =>
      `Undo batch ${n}? Its ${count(pdfs, "saved PDF", "saved PDFs")}, packing list and download copies are deleted, and its ${count(orders, "order", "orders")} can be saved again. If you already printed them, throw the printed labels away.`,
    // 08 › *1. Day*: the Amend confirmation, verbatim.
    confirmAmend: (n: number, time: string) =>
      `Redo batch ${n} with the current categories? Batch ${n} was saved at ${time}. If you already printed it, print it again after saving.`,
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
    // 08 › *3. Plan*: the boxes editor is used for the added pick's condition, with a text toggle.
    packingList: "Packing list:",
    scopeWhole: "one for the batch",
    scopePerPdf: "one per PDF",
    scopeNone: "none",
    layoutFull: "full",
    layoutSummary: "summary",
    layoutPick: "pick",
    save: "Save batch",
    // 08 › *3. Plan*: an amend uses the same screen, titled *Redo batch 2*, with this button.
    saveAgain: (n: number) => `Save batch ${n} again`,
    amendTitle: (n: number) => `Redo batch ${n}`,
    editCategories: "Edit categories",
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

  // 08 › *5. Categories*: the boxes editor and the file editor.
  categories: {
    title: "Categories",
    close: "Close",
    hint: "Order matters: an order is taken by the first category whose condition matches.",
    code: "Code",
    name: "Name",
    // 08 › *5. Categories*: "In batch 3: 130", in order from what is left.
    inBatch: (n: number, taken: number) => `In batch ${n}: ${taken}`,
    boxes: "Boxes",
    textView: "Text view",
    takesRest: "no condition: takes the rest",
    addCategory: "+ Add category",
    editFile: "Edit file as text",
    fileText: "File text",
    cancel: "Cancel",
    save: "Save",
    delete: "Delete",
    confirmDelete: (code: string) => `Delete category ${code}?`,
    moveUp: "Move up",
    moveDown: "Move down",
    allOf: "All of these",
    anyOf: "Any of these",
    addCondition: "+ condition",
    addGroup: "+ group",
    removeGroup: "× group",
    not: "not",
    groupEmpty: "This group is empty",
    helpExamples: "Help: examples ▸",
    helpTitle: "Examples",
    // 04 › *Examples*: the condition language examples shown by *Help: examples ▸*.
    examples: [
      "name contains \"sepatu\"",
      "total_quantity = 1",
      "tracking_id starts_with \"JY\"",
      "courier starts_with \"J&T\"",
      "ship_by < \"2026-10-07 17:00\"",
      "not name contains \"sepatu\" and (name contains \"spion\" or name contains \"knalpot\")",
    ],
  },
} as const;

// 08 › *5. Categories*: the fields a condition can use (07 › *Fields the conditions can use*),
// with the plain name and the field type that decides the operators and the value input.
export type FieldType = "text" | "number" | "time";

export const conditionFields: { value: string; label: string; type: FieldType }[] = [
  { value: "name", label: "Product name", type: "text" },
  { value: "display_name", label: "Display name", type: "text" },
  { value: "variation", label: "Variation", type: "text" },
  { value: "seller_sku", label: "Seller SKU", type: "text" },
  { value: "line_quantity", label: "Line quantity", type: "number" },
  { value: "total_quantity", label: "Total quantity", type: "number" },
  { value: "distinct_items", label: "Different items", type: "number" },
  { value: "courier", label: "Courier", type: "text" },
  { value: "tracking_id", label: "Tracking ID", type: "text" },
  { value: "ship_by", label: "Ship by", type: "time" },
];

/// The operators a text field takes (04 › *Grammar*).
export const textOperators: { value: string; label: string }[] = [
  { value: "contains", label: "contains" },
  { value: "equals", label: "equals" },
  { value: "starts_with", label: "starts_with" },
];

/// The operators a number or date/time field takes (04 › *Grammar*).
export const numberOperators: { value: string; label: string }[] = [
  { value: "=", label: "=" },
  { value: "!=", label: "≠" },
  { value: "<", label: "<" },
  { value: "<=", label: "≤" },
  { value: ">", label: ">" },
  { value: ">=", label: "≥" },
];

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
