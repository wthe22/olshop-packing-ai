// Every screen text of the PC app, in English. Components import from here; no screen text is written in a component.

function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

export const texts = {
  title: "Packing",

  // The line the Categories screen and the Plan screen show while the picks are
  // unconfigured (the one catch-all entry a folder without `categories.toml` starts with, so
  // every order goes into one PDF set). The Day screen has no line.
  noCategories: "No picks yet: all orders go into one PDF set",

  nav: {
    day: "Day",
    search: "Search",
    categories: "Categories",
    settings: "Settings",
    today: "Today",
    previousDay: "Previous day",
    nextDay: "Next day",
  },

  start: {
    // "Where should the data be kept? `D:\Packing\data` is next to the program."
    noData: (folder: string) =>
      `Where should the data be kept? The data folder next to the program is ${folder}\\data.`,
    useThisFolder: "Use the folder next to the program",
    chooseFolder: "Choose a folder…",
  },

  day: {
    // "Today: 2 batches · 729 orders · 5 group PDFs".
    summary: (batches: number, orders: number, pdfs: number) =>
      `Today: ${count(batches, "batch", "batches")} · ${count(orders, "order", "orders")} · ${count(pdfs, "group PDF", "group PDFs")}`,
    nothing: "Nothing saved on this day yet.",
    newBatch: "+ New batch",
    // "Batch 2 · saved 09:12 · 2 files · 129 orders".
    batch: (n: number, time: string, files: number, orders: number) =>
      `Batch ${n} · saved ${time} · ${count(files, "file", "files")} · ${count(orders, "order", "orders")}`,
    open: "Open",
    // The **Printed** tick beside *Open* on each group PDF. The tick carries no word, so this is
    // its accessible name and tooltip; it names the file, so a screen reader reads one row's tick
    // apart from another's.
    printedLabel: (file: string) => `Printed ${file}`,
    openFolder: "Open folder",
    summaryPdf: "Summary PDF",
    // "88 orders   3 order contents".
    counts: (orders: number, contents: number) =>
      `${count(orders, "order", "orders")}   ${count(contents, "order content", "order contents")}`,
    // A batch's copies left out (repeats, orders saved earlier).
    leftOut: (n: number) => `Left out: ${count(n, "copy", "copies")}`,
    // An output found missing or changed when the date opened.
    missing: "missing",
    amended: " · amended",
    // The data folder also holds files of Packing v0.1.
    oldData: "This folder also holds data of Packing v0.1 (labels\\). It is not read.",
    // The batch's stored warnings, expanding to the texts.
    warnings: (n: number) => `Warnings: ${n} ▸`,
    warningsNone: "Warnings: none",
    // The verifier's errors of a batch saved anyway, expanding to the texts. Shown until the batch is reverted or amended.
    savedWithErrors: (n: number) => `Saved with ${count(n, "error", "errors")} ▸`,
    // The green line after *Save batch*, shown for a few seconds.
    saved: (n: number, pdfs: number) =>
      `Batch ${n} saved: ${count(pdfs, "group PDF", "group PDFs")} and the summary PDF.`,
    loading: "Loading…",
    // *Amend* and *Revert*, on the newest batch only.
    amend: "Amend",
    revert: "Revert",
    undoBatch: (n: number) => `Undo batch ${n}`,
    redoBatch: (n: number) => `Redo batch ${n}`,
    // The Revert confirmation, verbatim.
    confirmRevert: (n: number, pdfs: number, orders: number) =>
      `Undo batch ${n}? Its ${count(pdfs, "group PDF", "group PDFs")} and its summary PDF are deleted, and its ${count(orders, "order", "orders")} can be saved again. If you already printed them, throw the printed labels away.`,
    // The Amend confirmation, verbatim. An amend redoes the batch with its sort rule, so the
    // confirmation names it.
    confirmAmend: (n: number, rule: string, time: string) =>
      `Redo batch ${n} with its sort rule (${rule})? Batch ${n} was saved at ${time}. If you already printed it, print it again after saving.`,
    // While *Amend* reads the batch's own label files again (the
    // `read-progress` event), shown with a progress bar before the Plan screen opens.
    amendReading: (page: number, pages: number) =>
      `Reading the batch's labels …  page ${page} of ${pages}`,
    // Changing the day while a draft is open asks first; the batch being
    // prepared is thrown away. (The Plan screen's top *Back* never asks.)
    confirmDiscardDraft: "Discard the batch being prepared?",
  },

  newBatch: {
    title: (n: number) => `New batch ${n}`,
    // The amendment's PDF selection page is
    // the same screen, titled *Redo batch `<b>`*.
    amendTitle: (n: number) => `Redo batch ${n}`,
    cancel: "Cancel",
    dropHint: "Drop label PDFs here, or",
    addFiles: "Add files…",
    // The Set control before *Read labels*. On the amend's
    // selection page the batch's own set is shown and not chosen, so the name is followed by
    // *(fixed)*.
    grouping: "Grouping",
    groupingFixed: (name: string) => `${name} (fixed)`,
    pages: (n: number) => `${count(n, "page", "pages")}`,
    usedInBatch: (n: number) => `used in batch ${n}`,
    remove: "×",
    removeTitle: "Remove this file",
    selected: (files: number, pages: number) =>
      `Selected: ${count(files, "file", "files")} · ${count(pages, "page", "pages")}`,
    read: "Read labels",
    // "Reading labels …  page 143 of 257 (file 1 of 2)".
    reading: (page: number, pages: number, file: number, files: number) =>
      `Reading labels …  page ${page} of ${pages} (file ${file} of ${files})`,
    back: "Back",
  },

  // One unseen logo of the batch, asked about before the
  // Plan screen. The buttons are the courier names, then *Not a courier*, *I don't know* and
  // *Skip all*; Esc skips only the one logo shown.
  logo: {
    title: "Which courier is this logo?",
    // The doc's counts line: "4 pages · 4 orders".
    counts: (pages: number, orders: number) =>
      `${count(pages, "page", "pages")} · ${count(orders, "order", "orders")}`,
    notCourier: "Not a courier",
    dontKnow: "I don't know",
    skipAll: "Skip all",
  },

  plan: {
    // The batch names the set it is packed with:
    // "Batch 3 · set default · 2 files · 257 pages · 256 orders · 0 already saved".
    header: (
      batch: number,
      grouping: string,
      files: number,
      pages: number,
      orders: number,
      alreadySaved: number,
    ) =>
      `Batch ${batch} · sort rule ${grouping} · ${count(files, "file", "files")} · ${count(pages, "page", "pages")} · ${count(orders, "order", "orders")} · ${alreadySaved} already saved`,
    warnings: (n: number) => `Warnings: ${n} ▸`,
    warningsNone: "Warnings: none",
    use: "Use",
    category: "Group",
    orders: "Orders",
    runs: "Contents",
    fileName: "File name",
    rest: "(the rest)",
    // A group that takes 0 orders.
    noPdf: "0 orders — no PDF",
    skipped: "skipped for this batch",
    moveUp: "Move up",
    moveDown: "Move down",
    expand: "Order contents",
    collapse: "Hide order contents",
    addCategory: "+ Add a group for this batch",
    addCode: "Code",
    addName: "Name",
    addCondition: "Condition",
    addSubmit: "Add",
    addCancel: "Cancel",
    save: "Save batch",
    // An amend uses the same screen, titled *Redo batch 2*, with this button.
    saveAgain: (n: number) => `Save batch ${n} again`,
    // While *Save batch* runs — the stage and a progress bar. `saving` shows before the first `save-progress`.
    saving: "Saving …",
    saveCopying: "Copying inputs …",
    saveWriting: (done: number, all: number) => `Writing …  file ${done + 1} of ${all}`,
    saveVerifying: "Verifying the outputs …",
    saveCommitting: "Saving …",
    // The check before saving found something — the two lists, kept apart, with
    // *Save anyway* and *Cancel — nothing is saved*.
    checkTitle: "The verifier found the outputs differ from the record.",
    checkErrors: (n: number) => `Errors: ${n}`,
    checkWarnings: (n: number) => `Warnings: ${n}`,
    saveAnyway: "Save anyway",
    saveCancel: "Cancel — nothing is saved",
    amendTitle: (n: number) => `Redo batch ${n}`,
    // The second of two PDFs with the same name.
    duplicate: (code: string) => `Same name as group ${code}`,
    // The top button, on a new batch and on an amend alike (the same string, so
    // one home for it). It is not a discard: it returns to the PDF selection page the batch came
    // from — the new batch's file list, or the amend's selection page — with the files still
    // listed. Only *Cancel* on that page and an amend's *Cancel amend* discard.
    back: "Back",
    // Bottom, beside *Save batch `<b>` again* on an amend only. It leaves the
    // batch as it was and returns to the Day screen, and never asks (it writes nothing).
    cancelAmend: "Cancel amend",
  },

  settings: {
    title: "Settings",
    dataFolder: "Data folder",
    change: "Change…",
    open: "Open",
    // "Use `<folder>` for the data from now on? The data in `<old>` stays where it is."
    confirmChange: (folder: string, old: string) =>
      `Use ${folder} for the data from now on? The data in ${old} stays where it is.`,
    about: "About",
    version: (version: string) => `Version ${version}`,
    // The PDFium version (D14: version + PDFium version only).
    pdfium: (version: string) => `PDFium ${version}`,
    schema: (version: string) => `schema ${version}`,
  },

  // The first namespace's name, asked once when the data folder holds none.
  namespace: {
    ask: "Name the first namespace. Each namespace keeps its own batches and dates apart from the others.",
    placeholder: "Namespace name",
    create: "Use this name",
  },

  // The boxes editor and the file editor.
  categories: {
    title: "Categories",
    close: "Close",
    hint: "Order matters: an order is taken by the first category whose condition matches.",
    // The set row — the chooser, **+ New set** and **Delete set** — and the
    // file the chosen set is kept in. A batch is packed with one set, chosen on *New batch*;
    // same file name as the set name.
    grouping: "Grouping",
    newGrouping: "+ New grouping",
    deleteGrouping: "Delete grouping",
    groupingFile: (file: string) => `file: ${file}`,
    createGrouping: "Create",
    // A new set's name is refused before the file is written: empty, the reserved
    // name `default` (the data folder's own `categories.toml`), or a name already in the list.
    groupingNeedsName: "A grouping needs a name.",
    groupingNameReserved:
      "The name \"default\" belongs to the data folder's own categories.toml. Choose another name.",
    groupingNameTaken: "A grouping with that name is already there.",
    // **Delete set** asks first; the set's file is deleted and its categories fall back to the one
    // catch-all entry.
    confirmDeleteGrouping: (name: string) =>
      `Delete grouping ${name}? Its categories fall back to the one catch-all entry.`,
    code: "Code",
    name: "Name",
    // "In batch 3: 130", in order from what is left.
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
    // The condition language examples shown by *Help: examples ▸*.
    examples: [
      "name contains \"sepatu\"",
      "total_quantity = 1",
      "tracking_id starts_with \"JY\"",
      "courier starts_with \"J&T\"",
      "ship_by < \"2026-10-07 17:00\"",
      "not name contains \"sepatu\" and (name contains \"spion\" or name contains \"knalpot\")",
    ],
  },

  // Find a saved order by the same condition boxes as Categories. It only
  // reads; nothing is changed or saved.
  search: {
    title: "Search",
    // The line above the rows, "N orders in M batches".
    summary: (orders: number, batches: number) =>
      `${count(orders, "order", "orders")} in ${count(batches, "batch", "batches")}`,
    scope: "Scope",
    // *This day* (the day in the title bar) is the default.
    scopeDay: "This day",
    scopeAll: "All days",
    search: "Search",
    // One row's items, "name × quantity".
    item: (name: string, quantity: number) => `${name} ×${quantity}`,
    // An order saved before search existed keeps only its IDs.
    itemsNotStored: "items not stored",
    // The batch a matching order was saved in.
    batch: (n: number) => `batch ${n}`,
    noMatch: "No saved order matches.",
  },
} as const;

// The fields a condition can use,
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
  { value: "order_id", label: "Order ID", type: "text" },
  { value: "tracking_id", label: "Tracking ID", type: "text" },
  { value: "ship_by", label: "Ship by", type: "time" },
];

// The fields the Search panel offers: the ones a saved order keeps, a subset of
// `conditionFields` so a field reads the same in both screens. The display name and Ship by are
// not searched.
const searchFieldValues = [
  "order_id",
  "tracking_id",
  "courier",
  "name",
  "variation",
  "seller_sku",
  "line_quantity",
  "total_quantity",
  "distinct_items",
];

export const searchFields: { value: string; label: string; type: FieldType }[] =
  conditionFields.filter((field) => searchFieldValues.includes(field.value));

// The operators a text field takes.
export const textOperators: { value: string; label: string }[] = [
  { value: "contains", label: "contains" },
  { value: "equals", label: "equals" },
  { value: "starts_with", label: "starts_with" },
];

// The operators a number or date/time field takes.
export const numberOperators: { value: string; label: string }[] = [
  { value: "=", label: "=" },
  { value: "!=", label: "≠" },
  { value: "<", label: "<" },
  { value: "<=", label: "≤" },
  { value: ">", label: ">" },
  { value: ">=", label: "≥" },
];

// The day switch shows e.g. "Wed 7 Oct 2026". Derived from the date, so it
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

// The title bar's brand text — the program's name and its shown version,
// e.g. "Packing v0.1". The version is the one-place text the window title and *About* also show
// (from `StartInfo.version`, made in one function on the Rust side).
export function brandText(version: string): string {
  return `${texts.title} ${version}`;
}

// The stage of a running save, from the `save-progress` event — the stage in plain words with the pages done. The Plan screen
// shows this with a progress bar; before the first event it shows `plan.saving`.
export function saveStageText(
  stage: "CopyingInputs" | "Writing" | "Verifying" | "Saving",
  page: number,
  pages: number,
): string {
  switch (stage) {
    case "CopyingInputs":
      return texts.plan.saveCopying;
    case "Writing":
      return texts.plan.saveWriting(page, pages);
    case "Verifying":
      return texts.plan.saveVerifying;
    case "Saving":
      return texts.plan.saveCommitting;
  }
}

// The Plan screen's two ways out, from the kind of batch. The top button is
// *Back* on both — it returns to the PDF selection page the batch came from, with the files still
// listed, and is not a discard; the bottom discard button beside *Save batch* exists on an amend
// only and goes to the Day screen. Which
// screen each button leads to is part of the helper, so a test reads it without a window.

// The screen a Plan-screen button leads to: the PDF selection page (a new
// batch's file list, or the amend's selection page) or the Day screen.
export type PlanTarget = "selection" | "day";

// The Plan screen's buttons: their texts and the screen each one leads to. `discard` is `null`
// when there is no bottom discard button (a new batch).
export interface PlanButtons {
  top: string;
  topTarget: PlanTarget;
  discard: string | null;
  discardTarget: PlanTarget | null;
}

// The Plan screen's top button and bottom discard button, with their targets. `amend` is the
// batch being redone, or `null` for a new batch: an amend's top *Back* returns
// to its selection page and its *Cancel amend* to the Day screen; a new batch has neither a
// *Cancel amend* nor a button that leaves for the Day screen (only its file list's *Cancel*
// does, and it is not on this screen).
export function planButtons(amend: number | null): PlanButtons {
  return {
    top: texts.plan.back,
    topTarget: "selection",
    discard: amend === null ? null : texts.plan.cancelAmend,
    discardTarget: amend === null ? null : "day",
  };
}

// **Amend** on the newest batch reads the batch's own `download/` files
// again and opens the Plan screen (the picks) at once, titled *Redo batch `<b>`*; the PDF
// selection page comes only from that Plan screen's *Back*.
// Pure, so the flow is tested without a
// window.

// The batch-flow screens, named as the shell's own `Screen` is: *Day*, the
// PDF selection page (a new batch's file list, or the amend's selection page) or the Plan screen.
export type FlowScreen = "day" | "new-batch" | "plan";

// The screen **Amend** opens: the Plan screen — the picks first, read from the
// batch's own `download/` files — never the PDF selection page.
export function amendOpens(): FlowScreen {
  return "plan";
}

// The window side of the result, pure so it is tested without a window. One row
// reads "Sepatu ×2, Spion ×1"; an order saved before search existed kept no items and its row
// says *items not stored*.

// One Search row as the helpers below need it: the stored items, or the flag saying there are
// none.
export interface SearchRowText {
  items: { name: string; quantity: number }[];
  items_not_stored: boolean;
}

// The line above the Search panel's rows: "3 orders in 2 batches".
export function searchSummary(orders: number, batches: number): string {
  return texts.search.summary(orders, batches);
}

// One Search row's items: "Sepatu ×2, Spion ×1", or *items not stored* for an
// order saved before search existed.
export function rowItemsText(row: SearchRowText): string {
  if (row.items_not_stored) return texts.search.itemsNotStored;
  return row.items.map((item) => texts.search.item(item.name, item.quantity)).join(", ");
}
