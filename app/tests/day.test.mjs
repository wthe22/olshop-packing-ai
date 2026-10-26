// The Day screen's and Settings' lines, and the sort rules as the New batch screen lists them: the
// counts read singular where they should, and the back end's sort-rule list becomes the screen's
// groupings with the newest batch's rule preselected.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend } from "./tsModule.mjs";
import { texts } from "../src/texts.ts";

const { groupingsList } = await import("../src/lib/api/categories.ts");

test("the Day screen's counts read singular where they should", () => {
  assert.deepEqual(
    [texts.day.counts(1, 1), texts.day.counts(88, 3)],
    ["1 order   1 order content", "88 orders   3 order contents"],
  );
  assert.deepEqual(
    [texts.day.leftOut(1), texts.day.leftOut(14)],
    ["Left out: 1 copy", "Left out: 14 copies"],
  );
  assert.equal(texts.day.saved(2, 1), "Batch 2 saved: 1 group PDF and the summary PDF.");
  assert.equal(
    texts.day.summary(1, 2, 3),
    "Today: 1 batch · 2 orders · 3 group PDFs",
  );
  assert.equal(
    texts.day.batch(2, "09:12", 1, 129),
    "Batch 2 · saved 09:12 · 1 file · 129 orders",
  );
  assert.equal(texts.plan.duplicate("A"), "Same name as group A");
  assert.deepEqual(
    [texts.settings.version("v0.2"), texts.settings.pdfium("chromium/7881"), texts.settings.schema("1.0")],
    ["Version v0.2", "PDFium chromium/7881", "schema 1.0"],
  );
});

test("the sort rules are the New batch screen's groupings, the newest batch's rule preselected", async () => {
  const calls = useBackend({
    sort_rules: () => ({ rules: ["default", "harian"], preselected: "harian" }),
  });
  const list = await groupingsList();
  assert.deepEqual(list, {
    groupings: [
      { name: "default", file: "packing-workflows.toml" },
      { name: "harian", file: "packing-workflows.toml" },
    ],
    last_grouping: "harian",
  });
  assert.deepEqual(calls.map((call) => call.command), ["sort_rules"]);
});
