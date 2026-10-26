// The Repeated orders choices and lines: a card's choice clears its orders chosen by hand, an
// order's radio sets its copy, and the Plan's list and the file chips read from the back end's view.
//
// Run in `app/`: `npm test`.

import { test } from "node:test";
import assert from "node:assert/strict";

import "./tsModule.mjs";
import { texts } from "../src/texts.ts";
const { choicesOf, fileChips, leftOutLines, sameChoice, withCopy, withGroupChoice } = await import(
  "../src/lib/repeats.ts"
);

const order = (order_id, copies, kept, by_hand = false) => ({
  order_id,
  tracking_id: "JY0000001234",
  difference: null,
  copies: copies.map(([copy, file, page_start]) => ({ copy, file, page_start })),
  kept,
  by_hand,
});

// One group over files 1 and 2: order 102 twice in file 1, order 103 in both, 103 chosen by hand.
const view = {
  orders: 2,
  extra_copies: 2,
  extra_pages: 3,
  files: 2,
  all_files: 2,
  groups: [
    {
      number: 1,
      files: [],
      choice: { kind: "keep_first" },
      by_hand: true,
      take_out: [{ file: 2, only_here: 1 }],
      orders: [
        order("580000000000000102", [[1, 1, 2], [2, 1, 4]], 1),
        order("580000000000000103", [[3, 1, 6], [4, 2, 1]], 4, true),
      ],
    },
  ],
};
view.groups[0].files = [{ file: 1 }, { file: 2 }];

test("the choices are read from the view and changed one at a time", () => {
  assert.deepEqual(choicesOf(view), {
    groups: [{ group: 1, choice: { kind: "keep_first" } }],
    by_hand: [{ order_id: "580000000000000103", copy: 4 }],
  });
  assert.deepEqual(withGroupChoice(view, 1, { kind: "keep_last" }), {
    groups: [{ group: 1, choice: { kind: "keep_last" } }],
    by_hand: [],
  }, "a card's choice clears its orders chosen by hand");
  assert.deepEqual(withCopy(view, "580000000000000102", 2).by_hand, [
    { order_id: "580000000000000103", copy: 4 },
    { order_id: "580000000000000102", copy: 2 },
  ]);
  assert.equal(sameChoice({ kind: "take_out", file: 2 }, { kind: "take_out", file: 2 }), true);
  assert.equal(sameChoice({ kind: "take_out", file: 2 }, { kind: "take_out", file: 1 }), false);
  assert.equal(sameChoice({ kind: "keep_first" }, { kind: "keep_last" }), false);
});

test("the Plan's list of copies left out and the file list's chips", () => {
  assert.deepEqual(leftOutLines(view), [
    "580000000000000102 · file 1 p. 4 · the copy in file 1 p. 2 stays",
    "580000000000000103 · file 1 p. 6 · the copy in file 2 p. 1 stays",
  ]);
  const chips = fileChips(view);
  assert.deepEqual(chips.get(1), { chip: "G1", line: "2 orders also in file 2" });
  assert.deepEqual(chips.get(2), { chip: "G1", line: "1 order also in file 1" });
  assert.equal(texts.repeats.alsoIn(4, []), "4 orders twice in this file");
});

test("the screen's lines are the designed words", () => {
  assert.equal(
    texts.repeats.summary(14, 14, 14, 3, 5),
    "14 orders appear more than once · 14 extra copies · 14 extra pages · 3 of 5 files",
  );
  assert.deepEqual(
    [texts.repeats.group(1, [1, 3], 10), texts.repeats.group(2, [2], 4), texts.repeats.group(3, [1, 2, 4], 1)],
    ["Group 1 · files 1 and 3 · 10 orders", "Group 2 · file 2 only · 4 orders", "Group 3 · files 1, 2 and 4 · 1 order"],
  );
  assert.equal(texts.repeats.onlyHere(3, 12), "File 3 also holds 12 orders that are in no other file.");
  assert.equal(texts.repeats.foot(14, 986, 1000), "14 copies are left out; 986 of 1000 pages go on to the plan.");
  assert.deepEqual(
    [texts.repeats.difference(null), texts.repeats.difference("tracking_id")],
    ["same", "differs: the tracking ID"],
  );
  assert.equal(texts.plan.leftOut(14), "14 copies left out ▸");
});
