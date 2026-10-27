// The Day screen's and Settings' lines: the counts read singular where they should.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { texts } from "../src/texts.ts";

test("the Day screen's counts read singular where they should", () => {
  assert.deepEqual(
    [1, 88].map((n) => [texts.day.orderWord(n), texts.day.contentWord(n), texts.day.pageWord(n)]),
    [
      ["order", "content", "page"],
      ["orders", "contents", "pages"],
    ],
  );
  assert.equal(texts.day.leftOut(14), "Left out: 14 ▸");
  assert.deepEqual(
    [
      texts.day.leftOutRepeat("580000000000000042", texts.day.pages("labels_3.pdf", 12, 13), "labels_1.pdf", 4),
      texts.day.leftOutSaved("580000000000000057", texts.day.pages("labels_2.pdf", 7, 7), 1),
    ],
    [
      "580000000000000042 · labels_3.pdf p. 12–13 · repeat in this batch: the copy in labels_1.pdf p. 4 stays",
      "580000000000000057 · labels_2.pdf p. 7 · saved earlier in batch 1",
    ],
  );
  assert.deepEqual(
    [
      texts.day.problem("missing"),
      texts.day.reprints(1, 3),
      texts.day.reprints(2, 1),
    ],
    ["⚠ missing", "Reprints · 1 file · 3 orders", "Reprints · 2 files · 1 order"],
  );
  assert.equal(texts.day.saved(2, 1), "Batch 2 saved: 1 group PDF and the summary PDF.");
  assert.deepEqual(
    [texts.day.summary(1, 2, 3, 0), texts.day.summary(2, 729, 5, 1)],
    [
      "This day: 1 batch · 2 orders · 3 group PDFs",
      "This day: 2 batches · 729 orders · 5 group PDFs · 1 reprint",
    ],
  );
  assert.equal(
    texts.day.batch(2, "09:12", 1, 129),
    "Batch 2 · saved 09:12 · 1 file · 129 orders",
  );
  assert.equal(texts.plan.duplicate("A"), "Same name as group A");
});
