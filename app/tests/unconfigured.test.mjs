// The *No picks yet* line's rule: the line is shown while the picks are the one catch-all entry a
// data folder without `categories.toml` starts with. Pure logic; no window opens and nothing is
// read.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { unconfigured } from "../src/lib/unconfigured.ts";
import { texts } from "../src/texts.ts";

test("the line's words are the designed words", () => {
  assert.equal(texts.noCategories, "No picks yet: all orders go into one PDF set");
});

test("only the one catch-all entry is the unconfigured state", () => {
  // (the entries, whether the line shows)
  const cases = [
    [[{ when: null }], true],
    // A category, two entries and an empty list are all configured.
    [[{ when: 'name contains "sepatu"' }], false],
    [[{ when: 'name contains "sepatu"' }, { when: null }], false],
    [[], false],
  ];
  for (const [entries, expected] of cases) {
    assert.equal(unconfigured(entries), expected, JSON.stringify(entries));
  }
});
