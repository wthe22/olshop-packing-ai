// The **Printed** tick's rule (07 › *`state.json`*; 08 › *1. Day*): what a tick or an untick does
// to a batch's ticked saved-PDF numbers, and the tick's accessible label. Pure logic; no window
// opens and nothing is read or written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { togglePrinted } from "../src/lib/printed.ts";
import { texts } from "../src/texts.ts";

test("the tick's accessible label names the saved PDF", () => {
  // The tick carries no word beside it: its accessible label and tooltip name the saved PDF (its
  // file name), so one row's tick reads apart from another's.
  assert.equal(texts.day.printedLabel("4 A Sepatu ×88.pdf"), "Printed 4 A Sepatu ×88.pdf");
});

test("ticking adds the number once", () => {
  assert.deepEqual(togglePrinted([], 4, true), [4]);
  // Ticking the same row again leaves one entry.
  assert.deepEqual(togglePrinted([4], 4, true), [4]);
  assert.deepEqual(togglePrinted([4, 5], 4, true), [4, 5]);
});

test("unticking removes the number", () => {
  assert.deepEqual(togglePrinted([4, 5], 4, false), [5]);
  assert.deepEqual(togglePrinted([5], 5, false), []);
  assert.deepEqual(togglePrinted([], 5, false), []);
});

test("another row's number is untouched", () => {
  assert.deepEqual(togglePrinted([4], 5, true), [4, 5]);
  assert.deepEqual(togglePrinted([4, 5], 5, false), [4]);
});

test("another batch keeps its own list", () => {
  const batch1 = togglePrinted([4], 5, true);
  const batch2 = togglePrinted([6], 7, true);
  assert.deepEqual(batch1, [4, 5]);
  assert.deepEqual(batch2, [6, 7]);
});
