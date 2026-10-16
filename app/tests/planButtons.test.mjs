// The Plan screen's ways out (08 › *3. Plan*): the top button is *Back* on a new batch and on an
// amend, and the bottom row's discard button depends on which it is — none on a new batch (only
// *Save batch*), *Cancel amend* beside *Save batch `<b>` again* on an amend. Pure logic; no window
// opens and nothing is read.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { texts, planButtons } from "../src/texts.ts";

test("the Plan screen's way-out texts are the designed words", () => {
  assert.equal(texts.plan.back, "Back");
  assert.equal(texts.plan.cancelAmend, "Cancel amend");
  // The old *Discard batch n?* confirmation is gone: *Back* never asks and never discards.
  assert.equal("confirmDiscard" in texts.plan, false);
  assert.equal("cancel" in texts.plan, false);
});

test("a new batch: top *Back*, no bottom discard button", () => {
  assert.deepEqual(planButtons(null), { top: "Back", discard: null });
});

test("an amend: top *Back*, *Cancel amend* at the bottom", () => {
  assert.deepEqual(planButtons(2), { top: "Back", discard: "Cancel amend" });
});

test("the top button is *Back*, not a discard, on both kinds of batch", () => {
  assert.equal(planButtons(null).top, planButtons(5).top);
  assert.equal(planButtons(5).top, "Back");
});

test("the day switch still asks before discarding a draft, in the designed words", () => {
  assert.equal(texts.day.confirmDiscardDraft, "Discard the batch being prepared?");
});
