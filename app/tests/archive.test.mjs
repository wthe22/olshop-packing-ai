// Archiving against the faked back end: **Archive** in a date's row menu asks first with the date,
// its PDFs and their size; more than one date shows its progress; a PDF that stayed is named; at
// start the dates older than *Archive automatically*'s number go without a question; Settings ›
// Storage archives a range and turns *Archive automatically* on and off.
//
// Run in `app/`: `npm test`.

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, emitEvent } from "./tsModule.mjs";
import { Shell, backend } from "./shellKit.mjs";
import { answerQuestions } from "./answers.mjs";
import { texts } from "../src/texts.ts";

const { Storage } = await import("../src/lib/storage.svelte.ts");
const { access } = await import("../src/lib/readOnly.svelte.ts");

const MB = 1024 * 1024;
const ONE = { dates: ["2026-10-07"], pdfs: 14, bytes: 210 * MB };
const THREE = { dates: ["2026-09-01", "2026-09-02", "2026-09-03"], pdfs: 30, bytes: 1.5 * 1024 * MB };

// A shell whose back end archives what it previews, sending a step for each date; `left` names
// the PDFs that stay.
function archiving({ preview = ONE, left = [], days = null } = {}) {
  let shell;
  const shown = [];
  const calls = useBackend({
    ...backend(),
    archive_preview: () => preview,
    archive_after_days: () => days,
    set_archive_after_days: () => undefined,
    storage: () => ({ bytes: 3 * 1024 * MB, dates: 41, archived: 12 }),
    archive_scope: () => {
      preview.dates.forEach((date, i) => {
        emitEvent("archive-step", { date, number: i + 1, of: preview.dates.length });
        shown.push(shell.working);
      });
      return { archived: preview.dates, left };
    },
  });
  shell = new Shell();
  shell.day = "2026-10-07";
  const commands = (name) => calls.filter((call) => call.command === name);
  return { shell, calls, commands, shown };
}

test("**Archive** of one date asks with its PDFs and size, then shows the date again", async () => {
  const { shell, commands, shown } = archiving();
  const asked = answerQuestions(() => true);
  await shell.archiving.date("2026-10-07");
  assert.deepEqual(asked, [texts.archive.ask(["Wed, 7 Oct 2026"], 14, "210 MB")]);
  assert.equal(
    asked[0].body,
    "Its 14 PDFs (210 MB) are deleted. Its orders and summaries stay; it can no longer be printed, amended, reverted, reprinted or read again.",
  );
  assert.deepEqual(commands("archive_scope")[0].args.scope, { kind: "range", from: "2026-10-07", to: "2026-10-07" });
  assert.deepEqual(shown, [null], "one date shows no progress");
  assert.equal(commands("panel_dates").length, 1);
  assert.equal(commands("day_overview").length, 1);
  assert.equal(shell.error, "");
  assert.equal(shell.writing, false);
});

test("cancelled, nothing is archived; a date that cannot be archived now says so", async () => {
  const { shell, commands } = archiving();
  answerQuestions(() => false);
  await shell.archiving.date("2026-10-07");
  assert.equal(commands("archive_scope").length, 0);

  const none = archiving({ preview: { dates: [], pdfs: 0, bytes: 0 } });
  await none.shell.archiving.date("2026-10-07");
  assert.equal(none.shell.error, texts.archive.nothing);
  assert.equal(none.commands("archive_scope").length, 0);
});

test("more than one date shows its progress, and a PDF that stayed is named", async () => {
  const { shell, shown } = archiving({ preview: THREE, left: [{ date: "2026-09-02", file: "1 Input 01 abc.pdf" }] });
  await shell.archiving.run({ kind: "range", from: "2026-09-01", to: "2026-09-30" });
  assert.deepEqual(shown, [
    "Archiving: 2026-09-01, 1 of 3",
    "Archiving: 2026-09-02, 2 of 3",
    "Archiving: 2026-09-03, 3 of 3",
  ]);
  assert.equal(shell.working, null);
  assert.equal(shell.error, texts.archive.left(["2026-09-02\\1 Input 01 abc.pdf"]));
});

test("at start the dates older than the number go without a question; read-only, nothing", async () => {
  const off = archiving();
  await off.shell.archiving.atStart();
  assert.equal(off.commands("archive_scope").length, 0, "off");

  const on = archiving({ days: 60, preview: THREE });
  const asked = answerQuestions(() => true);
  await on.shell.archiving.atStart();
  assert.deepEqual(on.commands("archive_scope")[0].args.scope, { kind: "older_than", days: 60 });
  assert.deepEqual(asked, []);

  const readOnly = archiving({ days: 60 });
  access.readOnly = true;
  await readOnly.shell.archiving.atStart();
  access.readOnly = false;
  assert.equal(readOnly.commands("archive_after_days").length, 0);
});

test("Storage shows the sizes, archives its range and turns Archive automatically on and off", async () => {
  const { shell, commands } = archiving({ preview: THREE });
  const asked = answerQuestions(() => true);
  const storage = new Storage(shell.archiving, { from: "2026-09-01", to: "2026-10-26" });
  await storage.load();
  assert.deepEqual([storage.auto, storage.days], [false, 60]);
  assert.equal(storage.used.dates, 41);
  assert.deepEqual(storage.preview, THREE);

  await storage.setRange({ from: "2026-10-01", to: "2026-09-01" });
  assert.equal(storage.preview, null, "a range that runs backwards");
  await storage.archiveRange();
  assert.equal(commands("archive_scope").length, 0);
  await storage.setRange({ from: "2026-09-01", to: "2026-09-30" });
  await storage.archiveRange();
  assert.deepEqual(commands("archive_scope")[0].args.scope, { kind: "range", from: "2026-09-01", to: "2026-09-30" });
  assert.equal(asked.length, 1, "asked first");

  answerQuestions(() => false);
  await storage.turn(true);
  assert.equal(commands("set_archive_after_days").length, 0, "cancelled, nothing is written");
  assert.equal(storage.auto, false);

  answerQuestions(() => true);
  await storage.turn(true);
  assert.deepEqual(commands("set_archive_after_days")[0].args, { days: 60 });
  assert.deepEqual(commands("archive_scope")[1].args.scope, { kind: "older_than", days: 60 });
  assert.equal(storage.auto, true);

  await storage.changeDays(0);
  assert.deepEqual(commands("set_archive_after_days")[1].args, { days: 1 }, "at least one day");
  await storage.changeDays(Number.NaN);
  assert.equal(storage.days, 60);

  await storage.turn(false);
  assert.deepEqual(commands("set_archive_after_days").at(-1).args, { days: null });
  assert.equal(storage.auto, false);
});
