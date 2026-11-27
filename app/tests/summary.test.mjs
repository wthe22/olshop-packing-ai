// The Summary screen's shell: *Day summary* and a batch's *Summary* open it, its read asks the back
// end for the summary's parts, and **PDF** writes the summary it shows.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend } from "./tsModule.mjs";
import { Shell, backend, overview } from "./shellKit.mjs";

const { summaryView } = await import("../src/lib/api/summary.ts");

const BATCH = { batch: 2, summary: "2 Summary.pdf", accounts: [{ account: "Toko Alfa", orders: 3 }] };

test("the summary is read without a PDF, and PDF writes the one shown", async () => {
  const calls = useBackend({
    ...backend(),
    summary_view: () => ({ title: "Day summary · All accounts · 2026-10-07", counts: "7 orders", parts: [] }),
    open_output: () => ({ kind: "opened" }),
    account_summary_pdf: () => ({ path: "C:/Data/2026-10-07/2 Summary - Toko Alfa.pdf", errors: [] }),
    day_summary: () => "C:/Data/2026-10-07/Day Summary.pdf",
    open_path: () => undefined,
  });
  const shell = new Shell();
  shell.day = "2026-10-07";
  shell.overview = { ...overview(), batches: [BATCH] };

  const view = await summaryView("2026-10-07", null, null);
  assert.equal(view.counts, "7 orders");
  assert.deepEqual(calls.at(-1), { command: "summary_view", args: { day: "2026-10-07", batch: null, account: null } });

  // The date's summary: PDF writes the day summary of the account chosen.
  shell.openSummary(null);
  assert.equal(shell.screen, "summary");
  assert.equal(shell.summaryBatch, null);
  await shell.summaryPdf("Toko Alfa");
  assert.deepEqual(calls.find((call) => call.command === "day_summary").args, { day: "2026-10-07", account: "Toko Alfa" });

  // A batch's summary: every account opens its own summary PDF; one account's is written.
  shell.openSummary(2);
  calls.length = 0;
  await shell.summaryPdf(null);
  assert.deepEqual(calls[0], { command: "open_output", args: { day: "2026-10-07", file: "2 Summary.pdf" } });
  await shell.summaryPdf("Toko Alfa");
  assert.deepEqual(calls.find((call) => call.command === "account_summary_pdf").args, {
    day: "2026-10-07",
    batch: 2,
    account: "Toko Alfa",
  });

  // A batch no longer on the date opens nothing.
  shell.openSummary(9);
  calls.length = 0;
  await shell.summaryPdf(null);
  assert.equal(calls.length, 0);
});
