// The **Printed** tick's rule: what a tick or an untick does to a batch's ticked saved-PDF numbers,
// and the tick's accessible label. Pure logic; no window opens and nothing is read or written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { togglePrinted } from "../src/lib/printed.ts";
import { texts } from "../src/texts.ts";

test("a tick is added once and an untick removes it, leaving the other rows alone", () => {
  // The tick carries no word beside it: its accessible label and tooltip name the saved PDF (its
  // file name), so one row's tick reads apart from another's.
  assert.equal(
    texts.day.printedLabel("4 A Sepatu ×88.pdf"),
    "Printed 4 A Sepatu ×88.pdf",
  );

  // (ticked, the row's number, the new state, what the list becomes)
  const cases = [
    [[], 4, true, [4]],
    // Ticking the same row again leaves one entry.
    [[4], 4, true, [4]],
    [[4, 5], 4, true, [4, 5]],
    [[4, 5], 4, false, [5]],
    [[5], 5, false, []],
    [[], 5, false, []],
    // Another row's number is untouched, and another batch keeps its own list.
    [[4], 5, true, [4, 5]],
    [[4, 5], 5, false, [4]],
    [[6], 7, true, [6, 7]],
  ];
  for (const [ticked, number, on, expected] of cases) {
    assert.deepEqual(
      togglePrinted(ticked, number, on),
      expected,
      `${JSON.stringify(ticked)} ${number} ${on}`,
    );
  }
});
