// The Search screen's texts, part of `texts` (`texts.search`).

import type { Question } from "./lib/confirm.svelte";
import { count } from "./count";

export const searchTexts = {
  title: "Search",
  // The line above the rows, "N orders in M batches".
  summary: (orders: number, batches: number) =>
    `${count(orders, "order", "orders")} in ${count(batches, "batch", "batches")}`,
  // The dates searched: the date the Day screen shows (the default), ranges that end today, a range
  // of the owner's own, or every date. The range's two dates show beside each choice but *All days*.
  dates: "Dates",
  ranges: {
    viewed: "Viewed day",
    today: "Today",
    yesterday: "Yesterday",
    week: "Last 7 days",
    month: "Last 30 days",
    custom: "Custom range",
    all: "All days",
  },
  rangeTo: "to",
  rangeFrom: "First date",
  rangeUntil: "Last date",
  search: "Search",
  reset: "Reset",
  resetTitle: "Empty the boxes and set the dates back to the viewed day",
  // The search index is still being built.
  loading: "Loading",
  // One line of an order's items, "name × quantity".
  item: (name: string, quantity: number) => `${name} ×${quantity}`,
  noMatch: "No saved order matches.",
  // The heads in a card's head and at the foot of its items.
  shipBy: "Ship by",
  weight: "Weight",
  total: "Total",
  // In place of the tracking ID of a label that printed none.
  noTracking: "no tracking ID",
  // The courier of an order whose label named none.
  noCourier: "Unknown",
  // The pages an order fills in its group PDF: "page 14", or "pages 14–15" for a label with a
  // second page.
  pages: (first: number, last: number) =>
    first === last ? `page ${first}` : `pages ${first}–${last}`,
  // Ticks every order the filter found, or none.
  selectAll: "Select all",
  tick: (order: string) => `Reprint ${order}`,
  // An order printed again before, and its tooltip, one line per reprint.
  reprinted: (n: number) => (n === 1 ? "Reprinted" : `Reprinted ${n}×`),
  reprintOn: (n: number, day: string) => `Reprint ${n} on ${day}`,
  // The reprint bar at the foot of the screen.
  selected: (n: number, all: number) => `${n} of ${all} selected`,
  clear: "Clear",
  reprintedBefore: (n: number) => `${n} of them ${n === 1 ? "was" : "were"} reprinted before`,
  // Asked before printing again an order that was reprinted before.
  askAgain: (before: number, all: number): Question => ({
    title: `Reprint ${count(all, "order", "orders")}?`,
    body:
      all === 1
        ? "It was reprinted before."
        : `${before} of them ${before === 1 ? "was" : "were"} reprinted before.`,
    ok: "Reprint",
  }),
  // A row's **Open**, named for its group PDF.
  open: "Open",
  openFile: (file: string) => `Open ${file}`,
  // The bar under a card's items that opens its details, and the details' heads.
  trace: "Details",
  details: {
    address: "Address",
    pickupCode: "Pickup code",
    account: "Account",
    input: "Input file",
    group: "Group",
  },
  // "1,250 g".
  grams: (grams: number) => `${grams.toLocaleString("en-US")} g`,
  // An order saved before the area or the weight was read, its data folder not updated since.
  notRead: "not read yet",
  notReadWhy: "Saved before this was read. Start the program again and choose Update.",
  batch: (n: number) => `Batch ${n}`,
  reprint: "Reprint",
  // "Reprint 1 written: 2 orders, 2 pages."
  made: (n: number, orders: number, pages: number) =>
    `Reprint ${n} written: ${count(orders, "order", "orders")}, ${count(pages, "page", "pages")}.`,
  notKept: "The reprint was not kept: the written file did not match its sources.",
} as const;
