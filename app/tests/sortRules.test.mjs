// The sort rule a screen starts on and a new rule's name: the newest batch's rule preselected, the
// open batch's rule on the Sort rules screen, a name refused when blank or taken, and the rule
// that must stay (the last one, the only one that is on). Pure logic; no window opens and nothing
// is read or written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import {
  deleteProblem,
  openingRule,
  preselectRule,
  ruleNameProblem,
  switchOffProblem,
} from "../src/lib/sortRules.ts";
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

test("the Amend question names the batch's sort rule, in the words the guide quotes", () => {
  const amend = texts.day.askAmend(2, "default");
  assert.deepEqual(amend, {
    title: "Redo batch 2?",
    body: "Its labels are read again with the sort rule default.",
    ok: "Redo batch 2",
  });
  // 03-changes.md quotes the Amend question, so the two must not drift. The quote may be wrapped
  // over two lines in the Markdown, so its whitespace is collapsed first.
  const doc = readFileSync(
    new URL("../../docs/user-guide/03-changes.md", import.meta.url),
    "utf8",
  ).replace(/\s+/g, " ");
  assert.ok(
    doc.includes(`${amend.title} ${amend.body}`),
    "03-changes.md must quote the Amend question as texts.ts writes it",
  );
});

test("the Sort rules screen asks in the words its design quotes", () => {
  const ask = texts.sortRules.askDiscard("harian");
  assert.deepEqual(ask, {
    title: "Discard the changes to harian?",
    body: "They are not saved yet.",
    ok: "Discard",
  });
  const doc = readFileSync(
    new URL("../../docs/software-design/ui/sort-rules.md", import.meta.url),
    "utf8",
  ).replace(/\s+/g, " ");
  assert.ok(doc.includes(`*${ask.title}*`), "sort-rules.md must quote the question as texts.ts writes it");
  assert.equal(texts.sortRules.inBatchHead(3), "In batch 3");
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

test("the only rule that is on stays on, and the last rule stays", () => {
  const summary = (name, enabled) => ({ name, enabled });
  const twoOn = [summary("default", true), summary("harian", true), summary("promo", false)];
  assert.equal(switchOffProblem(twoOn, "default"), null);
  assert.equal(deleteProblem(twoOn, "default"), null);
  assert.equal(deleteProblem(twoOn, "promo"), null, "a rule that is off can go");

  const oneOn = [summary("default", true), summary("promo", false)];
  assert.equal(switchOffProblem(oneOn, "default"), "only-on");
  assert.equal(deleteProblem(oneOn, "default"), "only-on");
  assert.equal(switchOffProblem(oneOn, "promo"), null);
  assert.equal(deleteProblem([summary("default", true)], "default"), "last");
});
