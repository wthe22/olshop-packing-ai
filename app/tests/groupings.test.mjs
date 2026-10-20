// The grouping control's rules: which grouping the New batch and Categories controls start on, the fallback when
// the grouping used last is gone, and a new grouping's name. Pure logic; no window opens and
// nothing is read or written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import {
  preselectGrouping,
  categoriesGrouping,
  normalizeGroupingName,
  groupingProblem,
  groupingFile,
} from "../src/lib/groupings.ts";
import { texts } from "../src/texts.ts";

const groupings = [
  { name: "default", file: "categories.toml" },
  { name: "ramadhan", file: "categories/ramadhan.toml" },
  { name: "shoes", file: "categories/shoes.toml" },
];

test("the grouping control's words are the designed words", () => {
  // *Set: [default ▾]* before *Read labels*.
  assert.equal(texts.newBatch.grouping, "Grouping");
  // The amend's selection page: *Set: default (fixed)*.
  assert.equal(texts.newBatch.groupingFixed("default"), "default (fixed)");
  // The grouping row's two buttons and the file line.
  assert.equal(texts.categories.newGrouping, "+ New grouping");
  assert.equal(texts.categories.deleteGrouping, "Delete grouping");
  assert.equal(
    texts.categories.groupingFile("categories.toml"),
    "file: categories.toml",
  );
});

test("the choice the batch was packed with names the grouping in the Plan header", () => {
  // "Batch 3 · set default · 2 files · 257 pages · 256 orders · 0 already saved".
  assert.equal(
    texts.plan.header(3, "default", 2, 257, 256, 0),
    "Batch 3 · grouping default · 2 files · 257 pages · 256 orders · 0 already saved",
  );
});

test("the Amend confirmation names the batch's grouping", () => {
  // Verbatim: the literal must not drift from `texts.ts`.
  assert.equal(
    texts.day.confirmAmend(2, "default", "09:12"),
    "Redo batch 2 with its grouping (default)? Batch 2 was saved at 09:12. If you already printed it, print it again after saving.",
  );
});

test("the guide's quoted Amend confirmation is the app's own text", () => {
  // 03-changes.md quotes the Amend confirmation, so the two must not drift. The quote is wrapped
  // over two lines in the Markdown, so its whitespace is collapsed first.
  const doc = readFileSync(
    new URL("../../docs/user-guide/03-changes.md", import.meta.url),
    "utf8",
  ).replace(/\s+/g, " ");
  assert.ok(
    doc.includes(texts.day.confirmAmend(2, "default", "09:12")),
    "03-changes.md must quote the Amend confirmation as texts.ts writes it",
  );
});

test("the grouping used last is preselected", () => {
  assert.equal(preselectGrouping(groupings, "ramadhan"), "ramadhan");
  assert.equal(preselectGrouping(groupings, "default"), "default");
  assert.equal(preselectGrouping(groupings, "shoes"), "shoes");
});

test("a grouping used last whose file is gone falls back to `default`", () => {
  // The grouping was deleted (or renamed) since it was used: its name is no longer in the list.
  assert.equal(preselectGrouping(groupings, "gone"), "default");
  // `default` is always in the list, so the fallback is always a real grouping.
  assert.equal(
    preselectGrouping([{ name: "default", file: "categories.toml" }], "gone"),
    "default",
  );
});

test("the Categories screen opens the batch's grouping, else the grouping used last", () => {
  // From the Plan screen's *Edit categories*: the batch's own grouping.
  assert.equal(categoriesGrouping("shoes", groupings, "ramadhan"), "shoes");
  // From the top bar: the grouping used last.
  assert.equal(categoriesGrouping(null, groupings, "ramadhan"), "ramadhan");
  // A batch whose grouping is gone opens the grouping used last, like the top bar.
  assert.equal(categoriesGrouping("gone", groupings, "ramadhan"), "ramadhan");
});

test("a new grouping's name is a file name: trimmed, forbidden characters become `-`", () => {
  assert.equal(normalizeGroupingName("  ramadhan  "), "ramadhan");
  assert.equal(
    normalizeGroupingName("a/b:c*d?e\"f<g>h|i\\j"),
    "a-b-c-d-e-f-g-h-i-j",
  );
});

test("a new grouping's name is refused when blank, reserved or already there", () => {
  assert.equal(groupingProblem("", groupings), "empty");
  assert.equal(groupingProblem("   ", groupings), "empty");
  assert.equal(groupingProblem("default", groupings), "reserved");
  assert.equal(groupingProblem("  default ", groupings), "reserved");
  assert.equal(groupingProblem("ramadhan", groupings), "exists");
  // The check sees the normalised name: surrounding spaces are trimmed first.
  assert.equal(groupingProblem("shoes ", groupings), "exists");
});

test("a fresh name is accepted; the file it will get is its name", () => {
  assert.equal(groupingProblem("ramadhan baru", groupings), null);
  assert.equal(groupingProblem("a/b", groupings), null); // written as `categories/a-b.toml`
  assert.equal(normalizeGroupingName("a/b"), "a-b");
});

test("the file line comes from the list, and is empty for a grouping that is not there", () => {
  assert.equal(groupingFile(groupings, "default"), "categories.toml");
  assert.equal(groupingFile(groupings, "ramadhan"), "categories/ramadhan.toml");
  assert.equal(groupingFile(groupings, "gone"), "");
});
