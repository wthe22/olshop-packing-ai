// The app's own dialog: a question is shown until it is answered, a question asked over an open
// one answers the first *cancel*, and a question with two ways to do it says which was pressed.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useRunes } from "./tsModule.mjs";
import { texts } from "../src/texts.ts";

useRunes();
const { confirmBox } = await import("../src/lib/confirm.svelte.ts");

test("a question is shown until it is answered", async () => {
  assert.equal(confirmBox.question, null);
  const asked = confirmBox.ask(texts.day.askAmend(2, "default"));
  assert.deepEqual(confirmBox.question, texts.day.askAmend(2, "default"));
  confirmBox.answer("ok");
  assert.equal(await asked, true);
  assert.equal(confirmBox.question, null);

  const first = confirmBox.ask(texts.day.askAmend(2, "default"));
  const second = confirmBox.ask(texts.day.askDiscardDraft);
  assert.equal(await first, false, "the question asked over it answers the first no");
  assert.equal(confirmBox.question, texts.day.askDiscardDraft);
  confirmBox.answer("cancel");
  assert.equal(await second, false);
  confirmBox.answer("ok");
  assert.equal(confirmBox.question, null, "an answer with no question open does nothing");
});

test("a revert asks whether to save its PDFs or discard them", async () => {
  const question = texts.day.askRevert(2, 3);
  assert.deepEqual(question, {
    title: "Undo batch 2?",
    body: "Its 3 group PDFs and summary PDF are removed. Save asks for a folder to keep a copy in; Discard deletes them. Throw away its printed labels.",
    ok: "Save",
    other: "Discard",
  });
  for (const pressed of ["ok", "other", "cancel"]) {
    const asked = confirmBox.choose(question);
    confirmBox.answer(pressed);
    assert.equal(await asked, pressed);
  }

  const choice = confirmBox.choose(question);
  const yesOrNo = confirmBox.ask(texts.day.askDiscardDraft);
  confirmBox.answer("ok");
  assert.equal(await yesOrNo, true);
  assert.equal(await choice, "cancel", "a question asked over a choice cancels it");
});
