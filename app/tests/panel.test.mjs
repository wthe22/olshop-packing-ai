// The panel's months: newest first, and only the open date's month and the current month start
// open.
//
// Run in `app/`: `npm test`.

import { test } from "node:test";
import assert from "node:assert/strict";

import "./tsModule.mjs";
import { formatMonth, formatShortDay } from "../src/dates.ts";
const { months } = await import("../src/lib/panel.ts");

const row = (date) => ({ date, today: false, orders: 1, damaged: false, folder: null });

test("the dates fold by month, the open and the current month open", () => {
  const dates = ["2026-11-02", "2026-10-21", "2026-10-20", "2026-09-30", "2026-08-01"].map(row);
  const list = months(dates, "2026-09-30", "2026-11-02");
  assert.deepEqual(
    list.map((month) => [month.key, month.rows.length, month.startsOpen]),
    [
      ["2026-11", 1, true],
      ["2026-10", 2, false],
      ["2026-09", 1, true],
      ["2026-08", 1, false],
    ],
  );
  assert.deepEqual(months([], "2026-10-07", "2026-10-07"), []);
  assert.deepEqual([formatMonth("2026-10"), formatShortDay("2026-10-21")], ["October 2026", "Wed 21 Oct"]);
});
