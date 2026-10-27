// The app shell's state and commands, with a faked back end: which screen each step opens, what it
// keeps of a batch and what it asks the owner before dropping one.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, emitEvent } from "./tsModule.mjs";
import { Shell, backend, overview, read } from "./shellKit.mjs";
import { answerQuestions } from "./answers.mjs";
import { texts } from "../src/texts.ts";


test("starting the shell loads the folder, the workspace, the day and the day's overview", async () => {
  const calls = useBackend(
    backend({
      start_info: () => ({
        today: "2026-10-07",
        exe_dir: "C:/Packing",
        data_folder: "C:/Data",
        workspace: "Toko Contoh",
        version: "0.1",
      }),
      workspaces_list: () => ({ dates: [{ date: "2026-10-07" }, { date: "2026-10-06" }] }),
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
  assert.deepEqual(shell.panel.dates.map((row) => row.date), ["2026-10-07", "2026-10-06"]);
  assert.equal(shell.overview.totals.batches, 2, "the overview is today's");
  assert.equal(shell.workspace, "Toko Contoh");
  assert.equal(shell.needsWorkspace, false);
  assert.equal(shell.about.pdfium, "140.0");
  assert.equal(shell.error, "");
  assert.equal(shell.overlay, null);
  assert.deepEqual(
    calls.map((call) => call.command),
    ["start_info", "workspaces_list", "day_overview", "about"],
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

  // A folder without a workspace: the first workspace's name is asked, then the day opens.
  const named = useBackend(
    backend({
      start_info: () => ({ today: "2026-10-07", exe_dir: "", data_folder: "C:/Data", workspace: null, version: "0.2" }),
      workspace_new: (args) => args.name,
    }),
  );
  const first = new Shell();
  await first.load();
  assert.equal(first.needsWorkspace, true, "the workspace prompt shows");
  assert.deepEqual(named.map((call) => call.command), ["start_info"], "and no day is loaded");
  await first.nameWorkspace("Toko Contoh");
  assert.equal(first.needsWorkspace, false);
  assert.equal(first.workspace, "Toko Contoh");
  assert.deepEqual(named[1], { command: "workspace_new", args: { name: "Toko Contoh" } });
  assert.equal(first.overview.totals.batches, 2, "the day opens");

  // A name the folder cannot take keeps the prompt and says why.
  useBackend(
    backend({
      start_info: () => ({ today: "2026-10-07", exe_dir: "", data_folder: "C:/Data", workspace: null, version: "0.2" }),
      workspace_new: () => {
        throw "the name is taken";
      },
    }),
  );
  const refused = new Shell();
  await refused.load();
  await refused.nameWorkspace("Toko Contoh");
  assert.equal(refused.needsWorkspace, true);
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
  useBackend(
    backend({
      discard_draft: () => {
        discard += 1;
      },
    }),
  );
  // The first answer is *Cancel*, the others are yes.
  const confirmations = answerQuestions(() => confirmations.length !== 1);

  // The owner says no: the batch and its screen stay.
  const open = new Shell();
  open.onPlan(read());
  open.screen = "plan";
  await open.navigate("settings");
  assert.equal(open.overlay, "settings", "an overlay opens over the batch without asking");
  assert.equal(confirmations.length, 0);

  await open.openDate("2026-10-06");
  assert.equal(confirmations[0], texts.day.askDiscardDraft);
  assert.equal(open.screen, "plan", "the refused answer changes nothing");
  assert.equal(open.draftOpen, true);
  assert.equal(discard, 0, "the draft is kept");

  // The owner says yes: the draft is discarded and the chosen date's Day screen shows.
  await open.openDate("2026-10-06");
  assert.equal(discard, 1, "the open draft is discarded once");
  assert.equal(open.draftOpen, false);
  assert.equal(open.screen, "day");
  assert.equal(open.day, "2026-10-06");
  assert.equal(open.overlay, null);
  clearTimeout(open.savedTimer);
});


test("the panel opens another date and workspace, and a new workspace on today", async () => {
  const calls = useBackend(
    backend({
      workspace_open: () => undefined,
      workspace_new: (args) => {
        if (args.name === "Toko Contoh") throw "A workspace Toko Contoh is already there.";
        return args.name;
      },
    }),
  );
  const shell = new Shell();
  shell.workspace = "Toko Contoh";
  shell.today = "2026-10-07";
  shell.day = "2026-10-05";

  await shell.openWorkspace("Toko Contoh");
  assert.equal(calls.length, 0, "the open workspace is not opened again");
  await shell.openWorkspace("Toko Lain");
  assert.equal(shell.workspace, "Toko Lain");
  assert.equal(shell.day, "2026-10-07", "another workspace opens on today");
  assert.deepEqual(
    calls.map((call) => call.command).sort(),
    ["day_overview", "workspace_open", "workspaces_list"],
  );

  assert.equal(await shell.newWorkspace("Toko Contoh"), "A workspace Toko Contoh is already there.");
  assert.equal(shell.workspace, "Toko Lain", "a refused name changes nothing");
  shell.day = "2026-10-05";
  assert.equal(await shell.newWorkspace("Toko Baru"), null);
  assert.equal(shell.workspace, "Toko Baru");
  assert.equal(shell.day, "2026-10-07");

  useBackend(
    backend({
      workspace_open: () => {
        throw "Toko Hilang is not a workspace of this data folder.";
      },
    }),
  );
  await shell.openWorkspace("Toko Hilang");
  assert.equal(shell.workspace, "Toko Baru", "a workspace that cannot open changes nothing");
  assert.equal(shell.error, "Toko Hilang is not a workspace of this data folder.");

  assert.equal(shell.panelShown, true, "the panel is shown at every start");
  shell.togglePanel();
  assert.equal(shell.panelShown, false);
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
        return { batch: 3, files: [], read: read({ amend: 3, plan: { rows: [], has_duplicate_names: false, add_at: 0, no_groups: false, sort_rule: "ramadhan" } }) };
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


test("a batch with a repeat opens Repeated orders before the plan, and comes back to it", () => {
  useBackend(backend());
  const shell = new Shell();
  const repeats = { orders: 1, extra_copies: 1, extra_pages: 1, files: 1, all_files: 1, groups: [] };
  shell.onPlan(read({ repeats }));
  assert.equal(shell.screen, "repeats");
  shell.onRepeats(read({ repeats, orders: 22 }));
  assert.equal(shell.readResult.orders, 22, "a choice recounts the read");
  shell.continueToPlan();
  assert.equal(shell.screen, "plan");
  shell.reviewRepeats();
  assert.equal(shell.screen, "repeats", "Change… on the Plan screen");
  shell.backToFiles();
  assert.equal(shell.screen, "new-batch");
  assert.equal(shell.draftOpen, true, "Back keeps the draft");
  shell.onPlan(read({ repeats: null }));
  assert.equal(shell.screen, "plan", "no repeat: the plan at once");
});


test("a saved sort rule rebuilds the open plan, which the Plan screen opens again on", async () => {
  const calls = useBackend(
    backend({ plan: () => ({ rows: [], has_duplicate_names: false, add_at: 0, no_groups: true, sort_rule: "harian" }) }),
  );
  const shell = new Shell();
  await shell.afterRulesSaved();
  assert.equal(calls.length, 0, "no batch open: nothing to rebuild");
  shell.onPlan(read());
  await shell.afterRulesSaved();
  assert.deepEqual(calls.at(-1), { command: "plan", args: { edits: [] } });
  assert.equal(shell.readResult.plan.sort_rule, "harian");
  assert.equal(shell.planVersion, 1);
  shell.navigate("sort-rules");
  assert.equal(shell.overlay, "sort-rules");
});
