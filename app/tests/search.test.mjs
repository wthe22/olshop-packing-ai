// The Search panel's window side: the line above the results, what a row shows for its items, and
// the texts and field list the panel offers. Pure, so it is tested without a window.
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

test("a row shows its items as name × quantity, or says they were not stored", () => {
  const row = {
    items: [
      { name: "Sepatu", quantity: 2 },
      { name: "Spion Beat", quantity: 1 },
    ],
    items_not_stored: false,
  };
  assert.equal(rowItemsText(row), "Sepatu ×2, Spion Beat ×1");
  // An order saved before search existed kept no items.
  assert.equal(rowItemsText({ items: [], items_not_stored: true }), "items not stored");
  assert.equal(texts.search.itemsNotStored, "items not stored");
});

test("the panel's fixed texts are the designed words", () => {
  assert.equal(texts.nav.search, "Search");
  assert.equal(texts.search.title, "Search");
  assert.equal(texts.search.scope, "Scope");
  // *This day* (the day in the title bar) is the default.
  assert.equal(texts.search.scopeDay, "This day");
  assert.equal(texts.search.scopeAll, "All days");
  assert.equal(texts.search.search, "Search");
  assert.equal(texts.search.batch(2), "batch 2");
  assert.equal(texts.search.noMatch, "No saved order matches.");
  // A row's *Open* is the Day screen's button, so the two screens read the same.
  assert.equal(texts.day.open, "Open");
});

test("Search offers only fields of the shared table, and not the ones it cannot search", () => {
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
  // The display name and Ship by are not searched.
  assert.ok(!values.includes("display_name"));
  assert.ok(!values.includes("ship_by"));

  // Categories keeps every field of the rules; Search is a subset of the same table.
  const order = conditionFields.find((field) => field.value === "order_id");
  assert.deepEqual(order, { value: "order_id", label: "Order ID", type: "text" });
  assert.equal(conditionFields.length, 11);
  assert.ok(searchFields.length < conditionFields.length);
  for (const field of searchFields) {
    assert.ok(conditionFields.includes(field), `${field.value} comes from the shared table`);
  }
});
