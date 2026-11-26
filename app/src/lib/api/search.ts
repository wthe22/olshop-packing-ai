// Typed wrappers around the Tauri commands for the Search screen. The Rust side sends snake_case
// fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";
import type { DateRange } from "../searchRange";

// One item of a saved order: the product name, its variation (empty for none), its quantity and the
// Seller SKU the slip prints for it.
export interface SearchItem {
  name: string;
  variant: string;
  quantity: number;
  seller_sku: string | null;
}

// The receiver's province, city and district as the label prints them; `null` where it prints none.
export interface SearchArea {
  province: string | null;
  city: string | null;
  district: string | null;
}

// The parcel's weight as the label prints it: its grams, or none printed.
export type SearchWeight = { grams: number } | "not_printed";

// One saved order the filter matches.
export interface SearchRow {
  day: string;
  order_id: string;
  // `null` when the label printed none.
  tracking_id: string | null;
  // The account the order was saved under, and the listed shop its label shows; `null` when the
  // label printed no sender.
  account: string;
  shop: string | null;
  // The courier's name; `null` when no courier was named.
  courier: string | null;
  items: SearchItem[];
  // The date the slip says to ship by; `null` when it prints none.
  ship_by: string | null;
  // The `Qty Total` the slip prints; `null` on the Shopee-style slip.
  total_quantity: number | null;
  // The instant-service label's pickup code.
  pickup_code: string | null;
  // `null`: saved before the area was read, and the data folder not updated since.
  area: SearchArea | null;
  // `null`: saved before the weight was read, and the data folder not updated since.
  weight: SearchWeight | null;
  batch: number;
  // `<batch>-<code>`.
  group: string;
  // The group PDF's file name, for *Open*.
  file: string;
  // The pages the order fills in that group PDF, counted from 1.
  first_page: number;
  last_page: number;
  // The batch's last save, `09:12`.
  batch_time: string;
  // The input PDF it was read from, by the name it had when it was chosen, and its pages there.
  input: string;
  // The input's short hash, the one its file name carries (`{date}_{pdf}_{short hash}.pdf`).
  input_hash: string;
  input_first_page: number;
  input_last_page: number;
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

// The saved orders the filter matches on the dates of `range`, both ends included; `null` searches
// every date.
export const searchOrders = (filter: string, range: DateRange | null): Promise<SearchResults> =>
  invoke("search_orders", { filter, range });

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
