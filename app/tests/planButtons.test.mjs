// The Plan screen's ways out: the top button is *Back* on a new batch and on an amend, and it goes
// to the PDF selection page — the new batch's file list, or the amend's selection page — with the
// files still listed. The bottom row's discard button depends on which it is: none on a new batch
// (only *Save batch*), *Cancel amend* beside *Save batch `<b>` again* on an amend — and *Cancel
// amend* goes to the Day screen. **Amend** itself opens the Plan screen first (the picks), not the
// selection page. Pure logic; no window opens and nothing is read.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import "./tsModule.mjs";
import { texts } from "../src/texts.ts";
const { planButtons, amendOpens } = await import("../src/lib/planFlow.ts");

test("the Plan screen's buttons depend only on whether it is an amend", () => {
  // A new batch: *Back* to its file list, and no bottom discard button.
  assert.deepEqual(planButtons(null), {
    top: "Back",
    topTarget: "selection",
    discard: null,
    discardTarget: null,
  });
  // An amend: the same *Back* to the selection page, and *Cancel amend* to the Day screen.
  assert.deepEqual(planButtons(2), {
    top: "Back",
    topTarget: "selection",
    discard: "Cancel amend",
    discardTarget: "day",
  });
  // The top button is the same on both kinds of batch, and neither top button leaves for the Day
  // screen: only *Cancel amend* does.
  assert.equal(planButtons(null).top, planButtons(5).top);
  assert.equal(planButtons(null).topTarget, planButtons(5).topTarget);
  assert.notEqual(planButtons(5).topTarget, "day");
  assert.notEqual(planButtons(null).topTarget, "day");

  // **Amend** opens the Plan screen (the picks), not the PDF selection page: the batch's own files
  // are read again and that Plan screen's *Back* is the only way to the selection page.
  assert.equal(amendOpens(), "plan");
  assert.notEqual(amendOpens(), "new-batch");
});

test("the Plan screen's words are the designed words", () => {
  assert.equal(texts.plan.back, "Back");
  assert.equal(texts.plan.cancelAmend, "Cancel amend");
  // The old *Discard batch n?* confirmation is gone: *Back* never asks and never discards.
  assert.equal("confirmDiscard" in texts.plan, false);
  assert.equal("cancel" in texts.plan, false);

  // *2. New batch* → the amend's selection page reads *Redo batch 2*.
  assert.equal(texts.newBatch.amendTitle(2), "Redo batch 2");
  assert.equal(texts.newBatch.title(3), "New batch 3");
  assert.notEqual(texts.newBatch.amendTitle(2), texts.newBatch.title(2));

  // The amend's read shows its progress while the batch's labels are read again.
  assert.equal(
    texts.day.amendReading(143, 257),
    "Reading the batch's labels …  page 143 of 257",
  );
  // The day switch still asks before discarding a draft.
  assert.equal(texts.day.askDiscardDraft.title, "Discard the batch being prepared?");
  // A new batch and an amend save with the same button.
  assert.equal(texts.plan.save, "Save batch");
  assert.equal("saveAgain" in texts.plan, false);
});
