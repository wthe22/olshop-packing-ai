// The Search panel's window side (08 › *Search*, 2.21): the line above the results, what a row
// shows for its items, and the texts and field list the panel offers. Pure, so it is tested
// without a window.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import {
  conditionFields,
  rowItemsText,
  searchFields,
  searchSummary,
  texts,
} from "../src/texts.ts";

test("the result line counts orders and batches", () => {
  assert.equal(searchSummary(1, 1), "1 order in 1 batch");
  assert.equal(searchSummary(3, 2), "3 orders in 2 batches");
  assert.equal(texts.search.summary(576, 3), "576 orders in 3 batches");
});

test("a row shows its items as name × quantity", () => {
  const row = {
    items: [
      { name: "Sepatu", quantity: 2 },
      { name: "Spion Beat", quantity: 1 },
    ],
    items_not_stored: false,
  };
  assert.equal(rowItemsText(row), "Sepatu ×2, Spion Beat ×1");
});

test("a row of an order saved before search says items not stored", () => {
  assert.equal(rowItemsText({ items: [], items_not_stored: true }), "items not stored");
  assert.equal(texts.search.itemsNotStored, "items not stored");
});

test("the panel's fixed texts are the designed words", () => {
  assert.equal(texts.nav.search, "Search");
  assert.equal(texts.search.title, "Search");
  assert.equal(texts.search.scope, "Scope");
  // *This day* (the day in the title bar) is the default (Q-UI-3).
  assert.equal(texts.search.scopeDay, "This day");
  assert.equal(texts.search.scopeAll, "All days");
  assert.equal(texts.search.search, "Search");
  assert.equal(texts.search.batch(2), "batch 2");
  assert.equal(texts.search.noMatch, "No saved order matches.");
  // A row's *Open* is the Day screen's button, so the two screens read the same.
  assert.equal(texts.day.open, "Open");
});

test("Search offers the fields a saved order keeps", () => {
  const values = searchFields.map((field) => field.value).sort();
  assert.deepEqual(values, [
    "courier",
    "distinct_items",
    "line_quantity",
    "name",
    "order_id",
    "seller_sku",
    "total_quantity",
    "tracking_id",
    "variation",
  ]);
  // The display name and Ship by are not searched (2.21).
  assert.ok(!values.includes("display_name"));
  assert.ok(!values.includes("ship_by"));
});

test("the shared field table offers Order ID for both screens", () => {
  const order = conditionFields.find((field) => field.value === "order_id");
  assert.deepEqual(order, { value: "order_id", label: "Order ID", type: "text" });
  // Categories keeps every field of the rules; Search is a subset of the same table.
  assert.equal(conditionFields.length, 11);
  assert.ok(searchFields.length < conditionFields.length);
  for (const field of searchFields) {
    assert.ok(conditionFields.includes(field), `${field.value} comes from the shared table`);
  }
});
