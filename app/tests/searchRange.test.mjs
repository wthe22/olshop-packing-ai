// The dates a search covers: each choice of the Dates list, counted from the viewed date or today.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import "./tsModule.mjs";
const { presetRange, rangePresets, shiftDay } = await import("../src/lib/searchRange.ts");

test("each choice of dates stands for its range", () => {
  const shown = { from: "2026-09-01", to: "2026-09-15" };
  const cases = [
    ["viewed", { from: "2026-10-20", to: "2026-10-20" }],
    ["today", { from: "2026-10-27", to: "2026-10-27" }],
    ["yesterday", { from: "2026-10-26", to: "2026-10-26" }],
    ["week", { from: "2026-10-21", to: "2026-10-27" }],
    ["month", { from: "2026-09-28", to: "2026-10-27" }],
    ["custom", shown],
    ["all", null],
  ];
  for (const [preset, want] of cases) {
    assert.deepEqual(presetRange(preset, "2026-10-20", "2026-10-27", shown), want, preset);
  }
  assert.deepEqual(
    rangePresets("2026-10-20", "2026-10-27"),
    cases.map(([preset]) => preset),
    "the list shows every choice, the viewed date first",
  );
});

test("the viewed date is not offered twice when it is today", () => {
  assert.deepEqual(rangePresets("2026-10-27", "2026-10-27"), [
    "today",
    "yesterday",
    "week",
    "month",
    "custom",
    "all",
  ]);
});

test("a date moves across a month and a year", () => {
  const cases = [
    ["2026-10-01", -1, "2026-09-30"],
    ["2026-03-01", -1, "2026-02-28"],
    ["2026-01-03", -6, "2025-12-28"],
    ["2026-12-31", 1, "2027-01-01"],
  ];
  for (const [day, days, want] of cases) assert.equal(shiftDay(day, days), want, `${day} ${days}`);
});
