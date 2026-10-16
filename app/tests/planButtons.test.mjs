// The Plan screen's ways out (08 › *3. Plan*, 08 › *2. New batch* → *The amend's selection
// page*): the top button is *Back* on a new batch and on an amend, and it goes to the PDF
// selection page — the new batch's file list, or the amend's selection page — with the files still
// listed. The bottom row's discard button depends on which it is: none on a new batch (only
// *Save batch*), *Cancel amend* beside *Save batch `<b>` again* on an amend — and *Cancel amend*
// goes to the Day screen. Pure logic; no window opens and nothing is read.
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

test("a new batch: top *Back* to its file list, no bottom discard button", () => {
  assert.deepEqual(planButtons(null), {
    top: "Back",
    topTarget: "selection",
    discard: null,
    discardTarget: null,
  });
});

test("an amend: top *Back* to the selection page, *Cancel amend* to the Day screen", () => {
  assert.deepEqual(planButtons(2), {
    top: "Back",
    topTarget: "selection",
    discard: "Cancel amend",
    discardTarget: "day",
  });
});

test("the top button is *Back* on both kinds of batch, and neither leaves for the Day screen", () => {
  assert.equal(planButtons(null).top, planButtons(5).top);
  assert.equal(planButtons(5).top, "Back");
  assert.equal(planButtons(null).topTarget, planButtons(5).topTarget);
  // Only *Cancel amend* leaves for the Day screen.
  assert.notEqual(planButtons(5).topTarget, "day");
  assert.notEqual(planButtons(null).topTarget, "day");
});

test("a new batch's Plan screen has neither: no *Cancel amend* and no Day target", () => {
  const buttons = planButtons(null);
  assert.equal(buttons.discard, null);
  assert.equal(buttons.discardTarget, null);
  assert.notEqual(buttons.topTarget, "day");
});

test("the amendment's selection page reads *Redo batch `<b>`*, the file list *New batch `<b>`*", () => {
  // 08 › *2. New batch* → *The amend's selection page*.
  assert.equal(texts.newBatch.amendTitle(2), "Redo batch 2");
  assert.equal(texts.newBatch.title(3), "New batch 3");
  assert.notEqual(texts.newBatch.amendTitle(2), texts.newBatch.title(2));
});

test("the day switch still asks before discarding a draft, in the designed words", () => {
  assert.equal(texts.day.confirmDiscardDraft, "Discard the batch being prepared?");
});
