// The sort rule a screen starts on and a new rule's name: the newest batch's rule preselected, the
// open batch's rule on the Sort rules screen, a name refused when blank or taken. Pure logic; no
// window opens and nothing is read or written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import { openingRule, preselectRule, ruleNameProblem } from "../src/lib/sortRules.ts";
import { texts } from "../src/texts.ts";

const rules = ["default", "harian", "ramadhan"];

test("the sort rule's words are the designed words", () => {
  assert.equal(texts.newBatch.sortRule, "Sort rule");
  assert.equal(texts.newBatch.sortRuleFixed("default"), "default (as now)");
  assert.equal(texts.sortRules.copyOf("default"), "a copy of default");
  assert.equal(texts.noGroups, "No groups yet: all orders go into one PDF");
  assert.equal(
    texts.plan.header(3, "default", 2, 257, 256, 0),
    "Batch 3 · sort rule default · 2 files · 257 pages · 256 orders · 0 already saved",
  );
});

test("the Amend confirmation names the batch's sort rule, in the words the guide quotes", () => {
  const amend = texts.day.confirmAmend(2, "default", "09:12");
  assert.equal(
    amend,
    "Redo batch 2 with its sort rule (default)? Batch 2 was saved at 09:12. If you already printed it, print it again after saving.",
  );
  // 03-changes.md quotes the Amend confirmation, so the two must not drift. The quote is wrapped
  // over two lines in the Markdown, so its whitespace is collapsed first.
  const doc = readFileSync(
    new URL("../../docs/user-guide/03-changes.md", import.meta.url),
    "utf8",
  ).replace(/\s+/g, " ");
  assert.ok(doc.includes(amend), "03-changes.md must quote the Amend confirmation as texts.ts writes it");
});

test("the newest batch's rule is preselected, the first when it is gone", () => {
  assert.equal(preselectRule(rules, "harian"), "harian");
  assert.equal(preselectRule(rules, "lebaran"), "default");
  assert.equal(preselectRule([], "harian"), "");
});

test("the Sort rules screen opens on the open batch's rule, else the preselected one", () => {
  assert.equal(openingRule(rules, "ramadhan", "harian"), "ramadhan");
  assert.equal(openingRule(rules, null, "harian"), "harian");
  assert.equal(openingRule(rules, "lebaran", "harian"), "harian", "a rule that is gone");
});

test("a rule's name is refused when blank or taken, upper and lower case alike", () => {
  assert.equal(ruleNameProblem("  ", rules), "empty");
  assert.equal(ruleNameProblem("Harian", rules), "exists");
  assert.equal(ruleNameProblem(" lebaran ", rules), null);
});
