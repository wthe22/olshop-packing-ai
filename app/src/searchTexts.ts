// The Search screen's texts, part of `texts` (`texts.search`).

import type { Question } from "./lib/confirm.svelte";
import { count } from "./count";

export const searchTexts = {
  title: "Search",
  // The line above the rows, "N orders in M batches".
  summary: (orders: number, batches: number) =>
    `${count(orders, "order", "orders")} in ${count(batches, "batch", "batches")}`,
  // The dates searched: the open date (the default), ranges that end today, a range of the
  // owner's own, or every date. The range's two dates show under each choice but *All days*.
  dates: "Dates",
  ranges: {
    day: "This day",
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
  // The search index is still being built.
  loading: "Loading…",
  // One line of a row's items, "name × quantity".
  item: (name: string, quantity: number) => `${name} ×${quantity}`,
  noMatch: "No saved order matches.",
  // The head row; its tick selects every row the filter found.
  columns: {
    orderId: "Order ID",
    trackingId: "Tracking ID",
    courier: "Courier",
    items: "Items",
    date: "Date",
    file: "Group PDF",
    page: "Page",
  },
  // The courier of an order whose label named none.
  noCourier: "Unknown",
  // The pages an order fills in its group PDF: "14", or "14–15" for a label with a second page.
  pages: (first: number, last: number) => (first === last ? `${first}` : `${first}–${last}`),
  selectAll: "Select all results",
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
    ok: `Reprint ${count(all, "order", "orders")}`,
  }),
  // A row's **Open**, named for its group PDF.
  open: "Open",
  openFile: (file: string) => `Open ${file}`,
  // The chevron that opens a row's trace line.
  trace: "Where it came from",
  reprint: (n: number) => `Reprint ${count(n, "order", "orders")}`,
  // "Reprint 1 written: 2 orders, 2 pages."
  made: (n: number, orders: number, pages: number) =>
    `Reprint ${n} written: ${count(orders, "order", "orders")}, ${count(pages, "page", "pages")}.`,
  notKept: "The reprint was not kept: the written file did not match its sources.",
} as const;
