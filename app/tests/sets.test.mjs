// The set control's rules (04 › *Sets*; 08 › *2. New batch* → *The set*, 08 › *5. Categories*):
// which set the New batch and Categories controls start on, the fallback when the set used last
// is gone, and a new set's name. Pure logic; no window opens and nothing is read or written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import {
  preselectSet,
  categoriesSet,
  normalizeSetName,
  setProblem,
  setFile,
} from "../src/lib/sets.ts";
import { texts } from "../src/texts.ts";

const sets = [
  { name: "default", file: "categories.toml" },
  { name: "ramadhan", file: "categories/ramadhan.toml" },
  { name: "shoes", file: "categories/shoes.toml" },
];

test("the set control's words are the designed words", () => {
  // 08 › *2. New batch* → *The set*: *Set: [default ▾]* before *Read labels*.
  assert.equal(texts.newBatch.set, "Grouping");
  // The amend's selection page: *Set: default (fixed)*.
  assert.equal(texts.newBatch.setFixed("default"), "default (fixed)");
  // 08 › *5. Categories*: the set row's two buttons and the file line.
  assert.equal(texts.categories.newSet, "+ New grouping");
  assert.equal(texts.categories.deleteSet, "Delete grouping");
  assert.equal(texts.categories.setFile("categories.toml"), "file: categories.toml");
});

test("the choice the batch was packed with names the set in the Plan header", () => {
  // 08 › *3. Plan*: "Batch 3 · set default · 2 files · 257 pages · 256 orders · 0 already saved".
  assert.equal(
    texts.plan.header(3, "default", 2, 257, 256, 0),
    "Batch 3 · grouping default · 2 files · 257 pages · 256 orders · 0 already saved",
  );
});

test("the Amend confirmation names the batch's set", () => {
  // 08 › *1. Day*, verbatim.
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

test("the set used last is preselected", () => {
  assert.equal(preselectSet(sets, "ramadhan"), "ramadhan");
  assert.equal(preselectSet(sets, "default"), "default");
  assert.equal(preselectSet(sets, "shoes"), "shoes");
});

test("a set used last whose file is gone falls back to `default`", () => {
  // The set was deleted (or renamed) since it was used: its name is no longer in the list.
  assert.equal(preselectSet(sets, "gone"), "default");
  // `default` is always in the list, so the fallback is always a real set.
  assert.equal(preselectSet([{ name: "default", file: "categories.toml" }], "gone"), "default");
});

test("the Categories screen opens the batch's set, else the set used last", () => {
  // From the Plan screen's *Edit categories*: the batch's own set.
  assert.equal(categoriesSet("shoes", sets, "ramadhan"), "shoes");
  // From the top bar: the set used last.
  assert.equal(categoriesSet(null, sets, "ramadhan"), "ramadhan");
  // A batch whose set is gone opens the set used last, like the top bar.
  assert.equal(categoriesSet("gone", sets, "ramadhan"), "ramadhan");
});

test("a new set's name is a file name: trimmed, forbidden characters become `-`", () => {
  assert.equal(normalizeSetName("  ramadhan  "), "ramadhan");
  assert.equal(normalizeSetName("a/b:c*d?e\"f<g>h|i\\j"), "a-b-c-d-e-f-g-h-i-j");
});

test("a new set's name is refused when blank, reserved or already there", () => {
  assert.equal(setProblem("", sets), "empty");
  assert.equal(setProblem("   ", sets), "empty");
  assert.equal(setProblem("default", sets), "reserved");
  assert.equal(setProblem("  default ", sets), "reserved");
  assert.equal(setProblem("ramadhan", sets), "exists");
  // The check sees the normalised name: surrounding spaces are trimmed first.
  assert.equal(setProblem("shoes ", sets), "exists");
});

test("a fresh name is accepted; the file it will get is its name", () => {
  assert.equal(setProblem("ramadhan baru", sets), null);
  assert.equal(setProblem("a/b", sets), null); // written as `categories/a-b.toml`
  assert.equal(normalizeSetName("a/b"), "a-b");
});

test("the file line comes from the list, and is empty for a set that is not there", () => {
  assert.equal(setFile(sets, "default"), "categories.toml");
  assert.equal(setFile(sets, "ramadhan"), "categories/ramadhan.toml");
  assert.equal(setFile(sets, "gone"), "");
});
