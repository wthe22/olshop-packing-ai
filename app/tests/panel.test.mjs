// The panel's months: newest first, and only the open date's month and the current month start
// open; which rows offer **Archive**.
//
// Run in `app/`: `npm test`.

import { test } from "node:test";
import assert from "node:assert/strict";

import "./tsModule.mjs";
import { formatMonth, formatShortDay } from "../src/dates.ts";
const { canArchive, months } = await import("../src/lib/panel.ts");

const row = (date) => ({ date, today: false, orders: 1, damaged: false, folder: null, size_bytes: 0, archived: false });

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
  assert.deepEqual([formatMonth("2026-10"), formatShortDay("2026-10-21")], ["October 2026", "Wed, 21 Oct"]);
});

test("only a date with a saved batch that is not today, damaged or archived offers Archive", () => {
  const saved = { ...row("2026-10-07"), folder: "D:/data/2026-10-07", orders: 12 };
  for (const [change, offered] of [
    [{}, true],
    [{ today: true }, false],
    [{ damaged: true }, false],
    [{ archived: true }, false],
    [{ orders: 0 }, false],
    [{ orders: null }, false],
    [{ folder: null }, false],
  ]) {
    assert.equal(canArchive({ ...saved, ...change }), offered, JSON.stringify(change));
  }
});
