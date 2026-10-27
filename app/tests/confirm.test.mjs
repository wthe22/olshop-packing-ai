// The app's own yes-or-no dialog: a question is shown until it is answered, and a question asked
// over an open one answers the first *no*.
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
  const asked = confirmBox.ask(texts.day.askRevert(2, 3));
  assert.deepEqual(confirmBox.question, {
    title: "Undo batch 2?",
    body: "Its 3 group PDFs and summary PDF are deleted. Throw away its printed labels.",
    ok: "Undo batch 2",
  });
  confirmBox.answer(true);
  assert.equal(await asked, true);
  assert.equal(confirmBox.question, null);

  const first = confirmBox.ask(texts.day.askAmend(2, "default"));
  const second = confirmBox.ask(texts.day.askDiscardDraft);
  assert.equal(await first, false, "the question asked over it answers the first no");
  assert.equal(confirmBox.question, texts.day.askDiscardDraft);
  confirmBox.answer(false);
  assert.equal(await second, false);
  confirmBox.answer(true);
  assert.equal(confirmBox.question, null, "an answer with no question open does nothing");
});
