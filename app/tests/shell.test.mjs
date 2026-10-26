// The app shell's state and commands, with a faked back end: which screen each step opens, what it
// keeps of a batch and what it asks the owner before dropping one.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, emitEvent } from "./tsModule.mjs";
import { Shell, backend, overview, read } from "./shellKit.mjs";
import { texts } from "../src/texts.ts";


test("starting the shell loads the folder, the namespace, the day and the day's overview", async () => {
  const calls = useBackend(
    backend({
      start_info: () => ({
        today: "2026-10-07",
        exe_dir: "C:/Packing",
        data_folder: "C:/Data",
        namespace: "Toko Contoh",
        version: "0.1",
      }),
      namespaces_list: () => ({ dates: [{ date: "2026-10-07" }, { date: "2026-10-06" }] }),
      about: () => ({ version: "0.1", pdfium: "140.0", schema: "1.0" }),
    }),
  );
  const shell = new Shell();
  assert.equal(shell.ready, false, "nothing is shown before the load");
  assert.equal(shell.needsFolder, true, "the start prompt shows until the folder is known");

  await shell.load();
  assert.equal(shell.ready, true);
  assert.equal(shell.today, "2026-10-07", "the day being shown is today");
  assert.equal(shell.day, "2026-10-07");
  assert.equal(shell.exeDir, "C:/Packing");
  assert.equal(shell.dataFolder, "C:/Data");
  assert.equal(shell.version, "0.1", "the title bar's version");
  assert.equal(shell.needsFolder, false, "the start prompt is gone");
  assert.deepEqual(shell.dayList, ["2026-10-07", "2026-10-06"]);
  assert.equal(shell.overview.totals.batches, 2, "the overview is today's");
  assert.equal(shell.namespace, "Toko Contoh");
  assert.equal(shell.needsNamespace, false);
  assert.equal(shell.about.pdfium, "140.0");
  assert.equal(shell.error, "");
  assert.equal(shell.overlay, null);
  assert.deepEqual(
    calls.map((call) => call.command),
    ["start_info", "namespaces_list", "day_overview", "about"],
  );
  assert.deepEqual(calls[2].args, { day: "2026-10-07" });

  // No folder yet: the start prompt stays and the day is not loaded.
  const noFolder = useBackend(backend({ start_info: () => ({ today: "2026-10-08", exe_dir: "", data_folder: null, version: "0.1" }) }));
  const prompt = new Shell();
  await prompt.load();
  assert.equal(prompt.needsFolder, true);
  assert.equal(prompt.dataFolder, null);
  assert.equal(prompt.day, "2026-10-08");
  assert.equal(noFolder.length, 1, "no folder: only `start_info` was asked");

  // A folder without an namespace: the first namespace's name is asked, then the day opens.
  const named = useBackend(
    backend({
      start_info: () => ({ today: "2026-10-07", exe_dir: "", data_folder: "C:/Data", namespace: null, version: "0.2" }),
      namespace_new: (args) => args.name,
    }),
  );
  const first = new Shell();
  await first.load();
  assert.equal(first.needsNamespace, true, "the namespace prompt shows");
  assert.deepEqual(named.map((call) => call.command), ["start_info"], "and no day is loaded");
  await first.nameNamespace("Toko Contoh");
  assert.equal(first.needsNamespace, false);
  assert.equal(first.namespace, "Toko Contoh");
  assert.deepEqual(named[1], { command: "namespace_new", args: { name: "Toko Contoh" } });
  assert.equal(first.overview.totals.batches, 2, "the day opens");

  // A name the folder cannot take keeps the prompt and says why.
  useBackend(
    backend({
      start_info: () => ({ today: "2026-10-07", exe_dir: "", data_folder: "C:/Data", namespace: null, version: "0.2" }),
      namespace_new: () => {
        throw "the name is taken";
      },
    }),
  );
  const refused = new Shell();
  await refused.load();
  await refused.nameNamespace("Toko Contoh");
  assert.equal(refused.needsNamespace, true);
  assert.equal(refused.error, "the name is taken");
});


test("a new batch goes New batch → Plan → Save → the day screen", async () => {
  const calls = useBackend(backend());
  const shell = new Shell();
  shell.savedMessage = "an older save";

  // *New batch* from the day screen.
  shell.startBatch();
  assert.equal(shell.screen, "new-batch");
  assert.equal(shell.overlay, null);
  assert.deepEqual(shell.files, []);
  assert.equal(shell.amendBatch, null, "a new batch, not an amend");
  assert.equal(shell.savedMessage, null, "the older line is gone");

  // *Read labels* ends on the Plan screen with the draft open.
  const plan = read();
  shell.onPlan(plan);
  assert.equal(shell.screen, "plan");
  assert.equal(shell.readResult, plan);
  assert.equal(shell.draftOpen, true);
  assert.equal(shell.overlay, null);

  // *Back* shows the file list again with the draft kept.
  shell.backToFiles();
  assert.equal(shell.screen, "new-batch");
  assert.equal(shell.draftOpen, true, "the draft is kept, so a read is not repeated");

  // *Save batch* returns to the day screen and reports the batch.
  shell.onPlan(plan);
  await shell.onSaved({ batch: 1, pdfs: 2, saved: true, errors: [], warnings: [], after_save: [] });
  assert.equal(shell.screen, "day");
  assert.equal(shell.draftOpen, false, "the draft is gone");
  assert.equal(shell.readResult, null);
  assert.deepEqual(shell.files, []);
  assert.equal(shell.overlay, null);
  assert.equal(shell.savedMessage, texts.day.saved(1, 2), "the green line names the batch");
  assert.equal(calls.filter((call) => call.command === "day_overview").length, 1, "the day was recounted");
  clearTimeout(shell.savedTimer);

  // A second save replaces the line and its earlier timer.
  shell.startBatch();
  shell.onPlan(plan);
  await shell.onSaved({ batch: 2, pdfs: 1, saved: true, errors: [], warnings: [], after_save: ["output\\1 summary.pdf is open in another program"] });
  assert.equal(shell.savedMessage, texts.day.saved(2, 1));
  assert.equal(shell.error, "output\\1 summary.pdf is open in another program", "a move after the commit is reported");
  shell.error = "";
  clearTimeout(shell.savedTimer);
  shell.closeOverlay();
  assert.equal(shell.overlay, null, "an overlay that is not open stays closed");

  // *Cancel* on the file list leaves the batch without asking: nothing is open.
  shell.startBatch();
  await shell.cancelBatch();
  assert.equal(shell.screen, "day");
  assert.equal(shell.draftOpen, false);
  assert.equal(calls.some((call) => call.command === "discard_draft"), false);
});


test("leaving an open batch asks before dropping it, and a refused answer stays", async () => {
  let discard = 0;
  const confirmations = [];
  useBackend(
    backend({
      "plugin:dialog|message": (args) => {
        confirmations.push(args.message);
        // The first answer is *Cancel*, the others are the dialog's *Ok*.
        return confirmations.length !== 1 ? (args.buttons?.OkCancelCustom?.[0] ?? "Ok") : "Cancel";
      },
      discard_draft: () => {
        discard += 1;
      },
    }),
  );

  // The owner says no: the batch and its screen stay.
  const open = new Shell();
  open.onPlan(read());
  open.screen = "plan";
  await open.navigate("categories");
  assert.equal(open.overlay, "categories", "an overlay opens over the batch without asking");
  assert.equal(confirmations.length, 0);

  await open.navigate("day");
  assert.equal(confirmations[0], texts.day.confirmDiscardDraft);
  assert.equal(open.screen, "plan", "the refused answer changes nothing");
  assert.equal(open.draftOpen, true);
  assert.equal(discard, 0, "the draft is kept");

  // The owner says yes: the draft is discarded and the day screen shows.
  await open.navigate("day");
  assert.equal(discard, 1, "the open draft is discarded once");
  assert.equal(open.draftOpen, false);
  assert.equal(open.screen, "day");
  assert.equal(open.overlay, null);
  clearTimeout(open.savedTimer);

  // *Today* returns to today's day without changing the screen, and asks nothing with no draft.
  open.day = "2026-10-05";
  open.today = "2026-10-07";
  await open.goToday();
  assert.equal(open.day, "2026-10-07", "back to today");
  assert.equal(confirmations.length, 2, "a closed batch is not asked about again");

  // A day switch: `dayList` is newest first, so a step of 1 walks towards the newest day.
  const steps = useBackend(backend());
  const stepping = new Shell();
  stepping.dayList = ["2026-10-07", "2026-10-06", "2026-10-05"];
  stepping.day = "2026-10-06";
  stepping.today = "2026-10-07";
  await stepping.stepDay(1);
  assert.equal(stepping.day, "2026-10-07", "the next day up the list");
  await stepping.stepDay(-1);
  assert.equal(stepping.day, "2026-10-06", "and back");
  assert.equal(steps.filter((call) => call.command === "day_overview").length, 2, "each switch recounts the day");
  assert.deepEqual(steps[0].args, { day: "2026-10-07" });
  assert.deepEqual(steps[1].args, { day: "2026-10-06" });

  // The newest day has no day after it, so nothing changes.
  stepping.day = "2026-10-07";
  await stepping.stepDay(1);
  assert.equal(stepping.day, "2026-10-07");
  assert.equal(steps.length, 2, "and the day is not recounted");
});


test("Amend reads the batch's own files and opens the Plan with that batch", async () => {
  const shell = new Shell();
  let during = null;
  const calls = useBackend(
    backend({
      start_amend: (args) => {
        assert.equal(args.day, "2026-10-07");
        // The read reports its progress while it runs; the shell shows it on the day screen.
        emitEvent("read-progress", { page: 3, pages: 24 });
        during = shell.amendReading;
        return { batch: 3, files: [], read: read({ amend: 3, plan: { rows: [], has_duplicate_names: false, add_at: 0, no_categories: false, grouping: "ramadhan" } }) };
      },
    }),
  );
  shell.day = "2026-10-07";
  shell.savedMessage = "Batch 2 saved";
  await shell.amend({
    batch: 3,
    time: "09:12",
    amended: false,
    sort_rule: "ramadhan",
    files: 2,
    orders: 23,
    left_out: 0,
    pdfs: [],
    summary: "3 summary.pdf",
    summary_missing: false,
    warnings: [],
    errors: [],
    folder: "C:/Data/Toko Contoh/2026-10-07/output",
  });

  assert.equal(shell.screen, "plan", "the Plan screen opens at once");
  assert.equal(shell.amendBatch, 3, "the batch being redone");
  assert.equal(shell.readResult.amend, 3);
  assert.equal(shell.draftOpen, true);
  assert.equal(shell.savedMessage, null, "the older green line is gone");
  assert.equal(shell.busy, false, "the read is finished");
  assert.equal(shell.amendReading, null, "the progress line is cleared when the Plan opens");
  assert.deepEqual(during, { page: 3, pages: 24 }, "and it was shown while the read ran");
  assert.deepEqual(shell.files, []);
  assert.equal(shell.overlay, null);
  assert.equal(calls.filter((call) => call.command === "start_amend").length, 1);
  assert.equal(calls.filter((call) => call.command === "plugin:event|listen").length, 1, "the read's progress is listened to");
  assert.equal(calls.filter((call) => call.command === "plugin:event|unlisten").length, 1, "and the listener is released");
  assert.equal(calls.filter((call) => call.command === "day_overview").length, 0);

  // A failed amend reports it and leaves the day screen alone.
  useBackend(
    backend({
      start_amend: () => {
        throw "no batch to redo";
      },
    }),
  );
  const failing = new Shell();
  failing.day = "2026-10-07";
  await failing.amend({
    batch: 3,
    time: "09:12",
    amended: false,
    sort_rule: "ramadhan",
    files: 2,
    orders: 23,
    left_out: 0,
    pdfs: [],
    summary: "3 summary.pdf",
    summary_missing: false,
    warnings: [],
    errors: [],
    folder: "C:/Data/Toko Contoh/2026-10-07/output",
  });
  assert.equal(failing.error, "no batch to redo");
  assert.equal(failing.screen, "day", "the screen stays on the day");
  assert.equal(failing.busy, false, "and the busy flag is cleared");
});
