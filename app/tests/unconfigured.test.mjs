// The *No picks yet* line's rule (Q-UI-4, 08 › *5. Categories*): the line is shown while the
// picks are the one catch-all entry a data folder without `categories.toml` starts with, and the
// Categories screen applies it to the entries it is editing. Pure logic; no window opens and
// nothing is read.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { unconfigured } from "../src/lib/unconfigured.ts";
import { texts } from "../src/texts.ts";

test("the line's words are the designed words", () => {
  assert.equal(texts.noPicks, "No picks yet: all orders go into one PDF set");
});

test("the one catch-all entry is the unconfigured state", () => {
  assert.equal(unconfigured([{ when: null }]), true);
});

test("a pick, two entries and an empty list are not the unconfigured state", () => {
  assert.equal(unconfigured([{ when: 'name contains "sepatu"' }]), false);
  assert.equal(unconfigured([{ when: 'name contains "sepatu"' }, { when: null }]), false);
  assert.equal(unconfigured([]), false);
});
