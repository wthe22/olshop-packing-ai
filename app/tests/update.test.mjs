// The question at start when the data folder would be updated, with a faked back end: no date
// opens before the answer, **Update** shows its progress line by line, and **Open read-only**
// opens the folder in read-only mode.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, emitEvent } from "./tsModule.mjs";
import { Shell, backend } from "./shellKit.mjs";
import { updateTexts as words } from "../src/updateTexts.ts";

const { Update } = await import("../src/lib/update.svelte.ts");
const { access } = await import("../src/lib/readOnly.svelte.ts");

const asked = [{ date: "2026-11-25", older_views: true, batches: 2, orders: 61 }];
const startInfo = (migration = asked) => () => ({
  today: "2026-11-26",
  exe_dir: "C:/Packing",
  data_folder: "C:/Data",
  version: "0.8.2",
  migration,
  read_only: false,
});
const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

test("no date opens before the owner answers; read-only opens the folder for viewing", async () => {
  const calls = useBackend(backend({ start_info: startInfo(), read_only: () => undefined }));
  const shell = new Shell();
  await shell.load();
  assert.deepEqual(shell.migration, asked, "the question shows");
  assert.deepEqual(calls.map((call) => call.command), ["start_info"], "nothing else is asked");

  await shell.openReadOnly();
  assert.equal(access.readOnly, true);
  assert.deepEqual(shell.migration, [], "the question is gone");
  assert.ok(calls.some((call) => call.command === "day_overview"), "the Day screen opens");
  assert.equal(shell.error, "");

  // A folder chosen in Settings that would be updated asks again; one that would not opens.
  useBackend(
    backend({
      start_info: startInfo([]),
      choose_data_folder: (args) => ({ data_folder: args.path, migration: asked, read_only: false }),
      "plugin:dialog|open": () => "D:/Other",
    }),
  );
  await shell.changeFolder();
  assert.equal(access.readOnly, false);
  assert.deepEqual(shell.migration, asked);
});

test("Update shows the step running and a line for each date done", async () => {
  let finish;
  useBackend(backend({ update_data: () => new Promise((resolve) => (finish = resolve)) }));
  const update = new Update();
  const running = update.run();
  await tick();
  assert.equal(update.running, true);
  emitEvent("migration-step", { kind: "reading", date: "2026-11-25", batch: 1, page: 3, pages: 120 });
  assert.equal(update.current, "Reading labels: 2026-11-25, batch 1, page 3 of 120");

  const done = {
    date: "2026-11-25",
    batches: 1,
    skipped: [{ batch: 2, reason: "labels_2.pdf, an input of 2026-11-25, is missing or has changed: its labels cannot be read." }],
  };
  emitEvent("migration-step", { kind: "updated", ...done });
  assert.equal(update.current, null);
  assert.deepEqual(
    update.lines.map((line) => [line.text, line.skipped]),
    [
      [words.updated("2026-11-25", false, 1), false],
      [words.skipped(2, done.skipped[0].reason), true],
    ],
  );
  finish([done]);
  await running;
  assert.equal(update.done, true);
  assert.equal(update.running, false);
  assert.equal(update.lines.length, 2, "the dates returned are the lines kept");
});

test("an Update that stops says why and offers both answers again", async () => {
  useBackend(
    backend({
      update_data: () => {
        throw "another copy of the app is writing";
      },
    }),
  );
  const update = new Update();
  await update.run();
  assert.equal(update.error, "another copy of the app is writing");
  assert.equal(update.done, false);
  assert.equal(update.running, false);
});

test("each date's row says what changes", () => {
  assert.equal(words.changes(2, 61, true), "Addresses and weights of 2 batches, 61 orders · the date's lists");
  assert.equal(words.changes(0, 0, true), "the date's lists");
  assert.equal(words.changes(1, 1, false), "Addresses and weights of 1 batch, 1 order · the date's lists");
  assert.match(words.explain(3), /addresses and the parcels' weights of 3 batches/);
  assert.doesNotMatch(words.explain(0), /addresses/);
});

test("Continue after the update opens the Day screen; a read-only open that fails keeps the question", async () => {
  const calls = useBackend(backend({ start_info: startInfo() }));
  const shell = new Shell();
  await shell.load();
  await shell.updated();
  assert.deepEqual(shell.migration, []);
  assert.ok(calls.some((call) => call.command === "day_overview"));

  useBackend(
    backend({
      start_info: startInfo(),
      read_only: () => {
        throw "the data folder is out of reach";
      },
    }),
  );
  const failing = new Shell();
  await failing.load();
  await failing.openReadOnly();
  assert.equal(failing.error, "the data folder is out of reach");
  assert.deepEqual(failing.migration, asked, "the question stays");
  assert.equal(failing.busy, false);
});

test("a second Update while one runs starts nothing", async () => {
  let finish;
  const calls = useBackend(backend({ update_data: () => new Promise((resolve) => (finish = resolve)) }));
  const update = new Update();
  const running = update.run();
  await tick();
  await update.run();
  finish([{ date: "2026-11-24", batches: 0, skipped: [] }]);
  await running;
  assert.equal(calls.filter((call) => call.command === "update_data").length, 1);
  assert.deepEqual(update.lines, [{ text: "2026-11-24: lists written", skipped: false }]);
  assert.equal(words.failed("the disk is full"), "The update stopped: the disk is full");
});

test("a date of v0.7 says its files are moved, and the move shows how far it is", async () => {
  assert.equal(words.changes(0, 0, false, true), "Files of v0.7 moved in · the date's lists");
  assert.equal(
    words.explain(2, 1),
    "Update moves the files of 1 date out of the input and output folders of Packing v0.7 into their date folders, keeping the old folders in v0.7 backup. It reads the receivers' addresses and the parcels' weights of 2 batches from their label PDFs and writes the files that list each date again. No batch, label or PDF is changed or deleted.",
  );
  assert.match(words.explain(0, 3), /of 3 dates .* It writes the files/);

  let finish;
  useBackend(backend({ update_data: () => new Promise((resolve) => (finish = resolve)) }));
  const update = new Update();
  const running = update.run();
  await tick();
  emitEvent("migration-step", { kind: "moving", date: "2026-11-25", file: 3, files: 12 });
  assert.equal(update.current, "Moving files: 2026-11-25, file 3 of 12");
  finish([{ date: "2026-11-25", moved: true, batches: 2, skipped: [] }]);
  await running;
  assert.deepEqual(update.lines, [
    { text: "2026-11-25: files moved, addresses and weights of 2 batches read, lists written", skipped: false },
  ]);
});
