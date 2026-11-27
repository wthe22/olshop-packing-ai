// Archiving against the faked back end: the question names the dates and the size of their PDFs;
// more than one date shows its progress; a PDF that stayed is named; at start the dates older than
// the Automatic number go without a question; the Archive page ticks dates (one, a Shift range, a
// month), archives them and turns Automatic on and off; read-only, it changes nothing.
//
// Run in `app/`: `npm test`.

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, emitEvent } from "./tsModule.mjs";
import { Shell, backend } from "./shellKit.mjs";
import { answerQuestions } from "./answers.mjs";
import { texts } from "../src/texts.ts";

const { ArchivePage } = await import("../src/lib/archivePage.svelte.ts");
const { ArchiveAuto } = await import("../src/lib/archiveAuto.svelte.ts");
const { access } = await import("../src/lib/readOnly.svelte.ts");

const MB = 1024 * 1024;
const THREE = { dates: ["2026-09-01", "2026-09-02", "2026-09-03"], pdfs: 30, bytes: 210 * MB };

// A date of the page; `archive` is what archiving it deletes, `null` without a tick.
const row = (date, archive = { pdfs: 4, bytes: 70 * MB }, extra = {}) => ({
  date,
  today: false,
  orders: 12,
  size_bytes: 80 * MB,
  archived: false,
  archive,
  ...extra,
});
const PAGE = {
  loading: false,
  total_bytes: 3.2 * 1024 * MB,
  dates: [
    row("2026-11-27", null, { today: true }),
    row("2026-11-26"),
    row("2026-10-27"),
    row("2026-10-26", null, { archived: true }),
    row("2026-10-25"),
    row("2026-10-24"),
  ],
};

// A shell whose back end archives the asked dates, sending a step for each; `left` names the PDFs
// that stay.
function archiving({ preview = THREE, left = [], days = null } = {}) {
  let shell;
  const shown = [];
  const calls = useBackend({
    ...backend(),
    archive_preview: () => preview,
    archive_after_days: () => days,
    set_archive_after_days: () => undefined,
    archive_page: () => PAGE,
    archive_scope: (args) => {
      const dates = args.scope.kind === "dates" ? args.scope.dates : preview.dates;
      dates.forEach((date, i) => {
        emitEvent("archive-step", { date, number: i + 1, of: dates.length });
        shown.push(shell.working);
      });
      return { archived: dates, left };
    },
  });
  shell = new Shell();
  shell.day = "2026-10-07";
  const commands = (name) => calls.filter((call) => call.command === name);
  return { shell, calls, commands, shown };
}

test("the question names the dates and the size of their PDFs", () => {
  assert.deepEqual(texts.archive.ask(3, "210 MB"), {
    title: "Archive 3 dates?",
    body: "Their PDFs (210 MB) are deleted; their orders and summaries stay.",
    ok: "Archive",
  });
  assert.equal(texts.archive.ask(1, "72.3 MB").body, "Its PDFs (72.3 MB) are deleted; its orders and summaries stay.");
  assert.equal(texts.archive.foot(1, "72.3 MB"), "1 date · 72.3 MB of PDFs");
  assert.equal(texts.archive.total("3.2 GB"), "3.2 GB in all");
});

test("more than one date shows its progress, a PDF that stayed is named, and the panel follows", async () => {
  const { shell, shown, commands } = archiving({ left: [{ date: "2026-09-02", file: "1 Input 01 abc.pdf" }] });
  await shell.archiving.run({ kind: "dates", dates: THREE.dates });
  assert.deepEqual(shown, [
    "Archiving: 2026-09-01, 1 of 3",
    "Archiving: 2026-09-02, 2 of 3",
    "Archiving: 2026-09-03, 3 of 3",
  ]);
  assert.equal(shell.working, null);
  assert.equal(shell.writing, false);
  assert.equal(shell.error, texts.archive.left(["2026-09-02\\1 Input 01 abc.pdf"]));
  assert.equal(commands("panel_dates").length, 1);
  assert.equal(commands("day_overview").length, 1);

  const one = archiving();
  await one.shell.archiving.run({ kind: "dates", dates: ["2026-09-01"] });
  assert.deepEqual(one.shown, [null], "one date shows no progress");
});

test("at start the dates older than the number go without a question; read-only, nothing", async () => {
  const off = archiving();
  await off.shell.archiving.atStart();
  assert.equal(off.commands("archive_scope").length, 0, "off");

  const on = archiving({ days: 60 });
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

test("a tick only on a date that can be archived; Shift ticks a range, a month tick its dates", async () => {
  const { shell } = archiving();
  const page = new ArchivePage(shell.archiving);
  await page.load();
  assert.equal(page.view.dates.length, 6);

  page.tick("2026-11-27", false);
  page.tick("2026-10-26", false);
  assert.deepEqual(page.ticked, [], "today and an archived date have no tick");

  page.tick("2026-11-26", false);
  page.tick("2026-10-24", true);
  assert.deepEqual(
    page.ticked,
    ["2026-11-26", "2026-10-27", "2026-10-25", "2026-10-24"],
    "the archived date is passed over",
  );
  assert.equal(page.bytes(), 4 * 70 * MB);
  page.tick("2026-10-27", false);
  assert.deepEqual(page.ticked, ["2026-11-26", "2026-10-25", "2026-10-24"], "a click clears");

  const october = PAGE.dates.filter((one) => one.date.startsWith("2026-10"));
  assert.equal(page.monthTicked(october), false);
  page.tickMonth(october);
  assert.equal(page.monthTicked(october), true);
  page.tickMonth(october);
  assert.deepEqual(page.ticked, ["2026-11-26"], "a ticked month clears its dates");
  assert.equal(page.monthTicked([PAGE.dates[0]]), false, "a month with nothing to tick");
});

test("Archive asks first, archives the ticked dates and reads the page again", async () => {
  const { shell, commands } = archiving();
  const page = new ArchivePage(shell.archiving);
  await page.load();
  page.tick("2026-10-25", false);
  page.tick("2026-10-24", false);

  const asked = answerQuestions(() => false);
  await page.archive();
  assert.deepEqual(asked, [texts.archive.ask(2, "140 MB")]);
  assert.equal(commands("archive_scope").length, 0, "cancelled");

  answerQuestions(() => true);
  await page.archive();
  assert.deepEqual(commands("archive_scope")[0].args.scope, { kind: "dates", dates: ["2026-10-25", "2026-10-24"] });
  assert.deepEqual(page.ticked, []);
  assert.equal(commands("archive_page").length, 2);
});

test("Automatic asks when it would archive now, keeps at least one day, and Off removes it", async () => {
  const { shell, commands } = archiving();
  const page = new ArchiveAuto(shell.archiving);
  await page.load();
  assert.deepEqual([page.auto, page.days], [false, 60]);

  const asked = answerQuestions(() => false);
  await page.turn(true);
  assert.deepEqual(asked, [texts.archive.ask(3, "210 MB")]);
  assert.equal(commands("set_archive_after_days").length, 0, "cancelled, nothing is written");
  assert.equal(page.auto, false);

  answerQuestions(() => true);
  await page.turn(true);
  assert.deepEqual(commands("set_archive_after_days")[0].args, { days: 60 });
  assert.deepEqual(commands("archive_scope")[0].args.scope, { kind: "older_than", days: 60 });
  assert.equal(page.auto, true);

  await page.changeDays(0);
  assert.deepEqual(commands("set_archive_after_days")[1].args, { days: 1 }, "at least one day");
  await page.changeDays(Number.NaN);
  assert.equal(page.days, 60);

  await page.turn(false);
  assert.deepEqual(commands("set_archive_after_days").at(-1).args, { days: null });
  assert.equal(page.auto, false);
});

test("read-only, nothing is ticked; a page read while the index is built is read again", async () => {
  const { shell } = archiving();
  const page = new ArchivePage(shell.archiving);
  await page.load();
  access.readOnly = true;
  page.tick("2026-11-26", false);
  page.tickMonth(PAGE.dates);
  access.readOnly = false;
  assert.deepEqual(page.ticked, []);

  let reads = 0;
  const calls = useBackend({
    ...backend(),
    archive_page: () => ({ ...PAGE, loading: ++reads === 1 }),
  });
  const loading = new ArchivePage(shell.archiving);
  await loading.load();
  assert.equal(loading.view.loading, true);
  await new Promise((resolve) => setTimeout(resolve, 600));
  assert.equal(loading.view.loading, false);
  loading.close();
  assert.equal(calls.filter((call) => call.command === "archive_page").length, 2);
});
