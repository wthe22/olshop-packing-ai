// The shell's folder, file and printed commands, with a faked back end: what each one
// sends and what the screen keeps when the back end refuses it.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, emitEvent } from "./tsModule.mjs";
import { Shell, backend, overview, read } from "./shellKit.mjs";
import { texts } from "../src/texts.ts";


test("the data folder: the start prompt, the program folder and a change", async () => {
  let picked = null;
  let answer = "Ok";
  let failChoose = false;
  const calls = useBackend({
    ...backend(),
    start_info: () => ({ today: "2026-10-07", exe_dir: "C:/Packing", data_folder: null, namespace: "Toko Contoh", version: "0.1" }),
    "plugin:dialog|open": () => picked,
    "plugin:dialog|message": (args) =>
      answer === "Ok" ? (args.buttons?.OkCancelCustom?.[0] ?? "Ok") : "Cancel",
    use_program_folder: () => "C:/Packing",
    choose_data_folder: (args) => {
      if (failChoose) throw "the folder cannot be written to";
      return args.path;
    },
  });
  const shell = new Shell();
  await shell.load();
  assert.equal(shell.needsFolder, true, "no folder yet");
  assert.equal(shell.panel, null);

  // The dialog was dismissed: the folder is still unknown and nothing else is asked.
  await shell.chooseFolder();
  assert.equal(shell.dataFolder, null);
  assert.equal(shell.needsFolder, true);
  assert.equal(shell.busy, false, "a dismissed dialog leaves nothing busy");
  assert.equal(calls.some((call) => call.command === "choose_data_folder"), false, "a dismissed dialog chooses nothing");

  // *Use the program's folder*: the folder is taken, its namespace, the day and About load.
  await shell.useProgramFolder();
  assert.equal(shell.dataFolder, "C:/Packing");
  assert.equal(shell.needsFolder, false);
  assert.equal(shell.busy, false);
  assert.deepEqual(shell.panel.dates, [{ date: "2026-10-07" }]);
  assert.equal(shell.overview.totals.batches, 2);
  assert.equal(shell.namespace, "Toko Contoh");
  assert.equal(shell.about.version, "0.2");
  assert.deepEqual(calls.find((call) => call.command === "use_program_folder").args, {}, "no argument is sent");

  // A chosen folder replaces it.
  calls.length = 0;
  picked = "C:/Data";
  await shell.chooseFolder();
  assert.equal(shell.dataFolder, "C:/Data");
  assert.equal(shell.needsFolder, false);
  assert.deepEqual(calls.find((call) => call.command === "plugin:dialog|open").args, { options: { directory: true, multiple: false, title: texts.start.chooseFolder } });

  // *Change folder* asks first; a refused answer leaves the folder alone.
  const chosen = calls.filter((call) => call.command === "choose_data_folder").length;
  answer = "Cancel";
  picked = "C:/Elsewhere";
  await shell.changeFolder();
  assert.equal(shell.dataFolder, "C:/Data", "the refused answer changes nothing");
  assert.equal(calls.filter((call) => call.command === "choose_data_folder").length, chosen, "and chooses nothing");

  answer = "Ok";
  await shell.changeFolder();
  assert.equal(shell.dataFolder, "C:/Elsewhere");
  assert.deepEqual(calls.filter((call) => call.command === "choose_data_folder").at(-1).args, { path: "C:/Elsewhere" });
  assert.ok(calls.filter((call) => call.command === "day_overview").length > 0, "the change reloads the day");

  // A folder the back end refuses: the error line shows it and the folder stays as it was.
  failChoose = true;
  picked = "C:/Broken";
  await shell.chooseFolder();
  await shell.changeFolder();
  assert.equal(shell.error, "the folder cannot be written to");
  assert.equal(shell.dataFolder, "C:/Elsewhere");
  assert.equal(shell.busy, false);
  assert.equal(shell.needsFolder, false);
});


test("the file and printed commands reach the back end", async () => {
  let fail = "";
  const calls = useBackend({
    ...backend(),
    start_info: () => ({ today: "2026-10-07", exe_dir: "", data_folder: "C:/Data", namespace: "Toko Contoh", version: "0.1" }),
    open_path: (args) => {
      if (fail === "open_path") throw "the file is gone";
      return { ...args };
    },
    show_in_folder: () => {
      if (fail === "show_in_folder") throw "the folder is gone";
    },
    set_printed: () => {
      if (fail === "set_printed") throw "the batch is gone";
    },
    revert_last: () => {
      if (fail === "revert_last") throw "nothing to revert";
    },
    restore_output: (args) => {
      if (fail === "restore_output") throw "the input is gone";
      return args.file === "2-B Spion x2.pdf"
        ? { restored: false, errors: ["page 3 of labels_1.pdf is in no output."] }
        : { restored: true, errors: [] };
    },
    "plugin:dialog|message": (args) => args.buttons?.OkCancelCustom?.[0] ?? "Ok",
  });
  const shell = new Shell();
  await shell.load();
  shell.day = "2026-10-07";

  // The day's own file opens and the folder shows in the file manager.
  await shell.openFile("C:/Data/Toko Contoh/2026-10-07/output/2-A Sepatu x2.pdf");
  await shell.openFolder("C:/Data/Toko Contoh/2026-10-07/output");
  assert.deepEqual(calls.find((call) => call.command === "open_path").args, { path: "C:/Data/Toko Contoh/2026-10-07/output/2-A Sepatu x2.pdf" });
  assert.deepEqual(calls.find((call) => call.command === "show_in_folder").args, { path: "C:/Data/Toko Contoh/2026-10-07/output" });
  assert.equal(shell.error, "");

  // The **Printed** tick of one group PDF: written, and the batch's rows updated in place.
  const pdfs = [
    { group: "A", name: "Sepatu", file: "2-A Sepatu x2.pdf", orders: 2, contents: 1, printed: false, problem: null },
    { group: "B", name: "Spion", file: "2-B Spion x2.pdf", orders: 2, contents: 1, printed: true, problem: "missing" },
  ];
  const row = { batch: 2, time: "09:00", amended: false, sort_rule: "default", files: 2, orders: 4, left_out: [], pdfs, summary: "2 summary.pdf", summary_problem: null, warnings: [], errors: [], folder: "C:/Data/Toko Contoh/2026-10-07/output" };
  await shell.setPrinted(row, pdfs[0], true);
  assert.deepEqual(pdfs.map((pdf) => pdf.printed), [true, true], "the tick is set");
  await shell.setPrinted(row, pdfs[1], false);
  assert.deepEqual(pdfs.map((pdf) => pdf.printed), [true, false], "and cleared");
  assert.deepEqual(calls.filter((call) => call.command === "set_printed").map((call) => call.args), [
    { day: "2026-10-07", batch: 2, group: "A", printed: true },
    { day: "2026-10-07", batch: 2, group: "B", printed: false },
  ]);

  // **Restore** writes a lost output again and recounts the day; a finding is shown, nothing kept.
  const counted = calls.filter((call) => call.command === "day_overview").length;
  await shell.restore("2-A Sepatu x2.pdf");
  assert.deepEqual(calls.find((call) => call.command === "restore_output").args, { day: "2026-10-07", file: "2-A Sepatu x2.pdf" });
  assert.equal(shell.error, "");
  assert.equal(calls.filter((call) => call.command === "day_overview").length, counted + 1);
  await shell.restore("2-B Spion x2.pdf");
  assert.equal(shell.error, texts.day.restoreFailed("2-B Spion x2.pdf", ["page 3 of labels_1.pdf is in no output."]));
  assert.equal(shell.busy, false);
  shell.error = "";

  // *Revert* asks first, then reverts the batch and recounts the day.
  const before = calls.filter((call) => call.command === "day_overview").length;
  await shell.revert(row);
  assert.equal(calls.filter((call) => call.command === "revert_last").length, 1);
  assert.deepEqual(calls.find((call) => call.command === "revert_last").args, { day: "2026-10-07" });
  assert.equal(calls.filter((call) => call.command === "day_overview").length, before + 1, "the day is recounted");

  // A failure of any of them shows the error line and keeps the day screen.
  for (const command of ["open_path", "show_in_folder", "set_printed", "restore_output", "revert_last"]) {
    fail = command;
    if (command === "restore_output") await shell.restore("2-A Sepatu x2.pdf");
    if (command === "open_path") await shell.openFile("C:/gone.pdf");
    if (command === "show_in_folder") await shell.openFolder("C:/gone");
    if (command === "set_printed") await shell.setPrinted(row, pdfs[0], true);
    if (command === "revert_last") await shell.revert(row);
    assert.notEqual(shell.error, "", `${command} reports its failure`);
    assert.equal(shell.screen, "day");
  }
  assert.equal(shell.error, "nothing to revert", "the last failure is the one shown");
});


test("a command that fails while loading shows the error and still opens the day", async () => {
  const shell = new Shell();
  useBackend({
    start_info: () => {
      throw "the program folder cannot be read";
    },
  });
  await shell.load();
  assert.equal(shell.ready, true, "the day opens even so");
  assert.equal(shell.error, "the program folder cannot be read");
  assert.equal(shell.needsFolder, true);
  assert.equal(shell.panel, null);

  // A failing day list, day or About leaves the day usable: the list is empty and the error
  // line carries the fault.
  const partial = new Shell();
  useBackend({
    ...backend(),
    start_info: () => ({ today: "2026-10-07", exe_dir: "", data_folder: "C:/Data", namespace: "Toko Contoh", version: "0.1" }),
    namespaces_list: () => {
      throw "the day folder is unreadable";
    },
    about: () => {
      throw "pdfium is missing";
    },
    day_overview: () => {
      throw "the journal cannot be read";
    },
  });
  await partial.load();
  assert.equal(partial.ready, true);
  assert.equal(partial.panel, null, "no panel list");
  assert.equal(partial.about, null);
  assert.equal(partial.overview, null);
  assert.equal(partial.error, "pdfium is missing", "the last failure is the one shown");

  // *Use the program's folder* failing leaves the start prompt as it was.
  const stuck = new Shell();
  useBackend({
    ...backend(),
    start_info: () => ({ today: "2026-10-07", exe_dir: "C:/Packing", data_folder: null, version: "0.1" }),
    use_program_folder: () => {
      throw "the program folder is read only";
    },
  });
  await stuck.load();
  await stuck.useProgramFolder();
  assert.equal(stuck.error, "the program folder is read only");
  assert.equal(stuck.dataFolder, null);
  assert.equal(stuck.needsFolder, true);
  assert.equal(stuck.busy, false);
});
