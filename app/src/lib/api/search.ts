// Typed wrappers around the Tauri commands for the Search screen. The Rust side sends snake_case
// fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

// One line of a Search row: the product name and its quantity.
export interface SearchItem {
  name: string;
  quantity: number;
}

// One saved order the filter matches.
export interface SearchRow {
  day: string;
  order_id: string;
  // `null` when the label printed none.
  tracking_id: string | null;
  // The courier's code; `null` when no courier was named.
  courier: string | null;
  items: SearchItem[];
  batch: number;
  // `<batch>-<code>`.
  group: string;
  // The group PDF, for *Open*.
  path: string;
  // Its input and pages, and the clue that named its courier.
  trace: string;
  // The reprints that printed it again, oldest first.
  reprints: ReprintMark[];
}

// A reprint that printed an order again.
export interface ReprintMark {
  // The date the reprint was made on.
  day: string;
  number: number;
}

export interface SearchResults {
  // The search index is still being built: no rows yet.
  loading: boolean;
  orders: number;
  batches: number;
  rows: SearchRow[];
}

// *This day* (the open date, the default) or *All days* of the open workspace.
export type SearchScope = "day" | "all";

export const searchOrders = (
  filter: string,
  scope: SearchScope,
  day: string,
): Promise<SearchResults> => invoke("search_orders", { filter, scope, day });

// One ticked row: the order and the date it was saved on.
export interface ReprintChoice {
  day: string;
  order_id: string;
}

export interface ReprintMade {
  number: number;
  file: string;
  path: string;
  orders: number;
  pages: number;
}

// What **Reprint** did: the file written into today, or the verifier's findings (nothing kept).
export interface ReprintResult {
  made: ReprintMade | null;
  errors: string[];
}

export const reprint = (orders: ReprintChoice[]): Promise<ReprintResult> =>
  invoke("reprint", { orders });

// One order of a reprint: where it was saved, and the order as Search shows it; `found` is `null`
// once no saved batch holds it (its batch was reverted since).
export interface ReprintedOrder {
  order_id: string;
  day: string;
  batch: number;
  found: SearchRow | null;
}

export interface ReprintedOrders {
  // The search index is still being built: no orders yet.
  loading: boolean;
  orders: ReprintedOrder[];
}

// The orders of reprint `number` of `day`, for the Day screen's *Orders ▸*.
export const reprintedOrders = (day: string, number: number): Promise<ReprintedOrders> =>
  invoke("reprint_orders_of", { day, number });
