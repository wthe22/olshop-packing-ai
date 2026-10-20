// Typed wrappers around the Tauri commands for the Search screen. The Rust side sends snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

// One line of a Search row: the product name and its quantity.
export interface SearchItem {
  name: string;
  quantity: number;
}

// One matching saved order as the Search panel shows it: its IDs, courier, items,
// the batch it was saved in, its saved PDF (name, and the full `path` *Open* uses) and whether its
// items were stored at all (an order saved before search existed).
export interface SearchRow {
  day: string;
  order_id: string;
  // The order's tracking ID; `null` when the label carried none (the table shows an empty cell).
  tracking_id: string | null;
  // The named courier; `null` when it is unknown.
  courier: string | null;
  items: SearchItem[];
  batch: number;
  pdf: number;
  file: string;
  path: string;
  items_not_stored: boolean;
}

// What a search returns: the counts shown above the rows, then one row per
// matching saved order.
export interface SearchResults {
  rows: SearchRow[];
  orders: number;
  batches: number;
}

// Which day folders *search_orders* reads: `day` is *This day* (the day in the title
// bar, the default), `all` is *All days*.
export type SearchScope = "day" | "all";

// Search the saved orders for the condition boxes: the same tree as Categories
// (`toRustTree`), with the scope. Reads `state.json` only; *Open* then uses `openPath`.
export const searchOrders = (
  conditions: unknown,
  scope: SearchScope,
): Promise<SearchResults> => invoke("search_orders", { conditions, scope });
