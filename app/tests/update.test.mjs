// The question at start when the data folder would be updated, with a faked back end: no date
// opens before the answer, **Update** shows the date running, a bar and the step, and **Open
// read-only** opens the folder in read-only mode.
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

test("Update shows the date running, a bar for the whole run and the step under it", async () => {
  let finish;
  useBackend(backend({ update_data: () => new Promise((resolve) => (finish = resolve)) }));
  const update = new Update();
  const running = update.run();
  await tick();
  assert.equal(update.running, true);
  emitEvent("migration-step", { kind: "starting", date: "2026-11-24", number: 3, of: 29 });
  assert.deepEqual([update.line, update.step, update.done, update.of], ["Updating 2026-11-24, 3 of 29", null, 2, 29]);
  emitEvent("migration-step", { kind: "reading", date: "2026-11-24", batch: 1, page: 30, pages: 120 });
  assert.deepEqual([update.step, update.done], ["Reading labels: page 30 of 120", 2.25]);
  emitEvent("migration-step", { kind: "moving", date: "2026-11-24", file: 12, files: 12 });
  assert.deepEqual([update.step, update.done], ["Moving files: 12 of 12", 3]);
  emitEvent("migration-step", { kind: "moving", date: "2026-11-24", file: 6, files: 12 });
  assert.equal(update.done, 2.5, "the bar counts from the dates finished");
  finish([{ date: "2026-11-24", moved: false, batches: 1, skipped: [] }]);
  assert.equal(await running, true, "every batch read: the Day screen opens at once");
  assert.equal(update.running, false);
  assert.deepEqual(update.skipped, []);
});

test("a batch not read is named in one line, and the card waits for Continue", async () => {
  useBackend(
    backend({
      update_data: () => [
        { date: "2026-11-24", moved: false, batches: 1, skipped: [{ batch: 2, reason: { kind: "label_changed" } }] },
        { date: "2026-11-25", moved: false, batches: 0, skipped: [] },
      ],
    }),
  );
  const update = new Update();
  assert.equal(await update.run(), false);
  assert.equal(words.skipped(update.skipped), "1 batch not read: 2026-11-24 batch 2, its label PDF has changed.");
  assert.equal(
    words.skipped([
      { date: "2026-11-24", batch: 2, error: null },
      { date: "2026-11-25", batch: 1, error: "The labels could not be read." },
    ]),
    "2 batches not read: 2026-11-24 batch 2, its label PDF has changed; 2026-11-25 batch 1, The labels could not be read.",
  );
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
  assert.equal(update.line, null);
  assert.equal(update.running, false);
});

test("the card says how many dates and that a copy is kept", () => {
  assert.equal(
    words.explain(29),
    "29 dates were saved by an older Packing. Updating takes a few minutes and keeps a copy of every file it changes.",
  );
  assert.match(words.explain(1), /^1 date was saved/);
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
  assert.equal(words.failed("the disk is full"), "The update stopped: the disk is full");
});
