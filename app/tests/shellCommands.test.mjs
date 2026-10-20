// The shell's folder, file, settings and printed commands, with a faked back end: what each one
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
    start_info: () => ({ today: "2026-10-07", exe_dir: "C:/Packing", data_folder: null, version: "0.1" }),
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
  assert.equal(shell.dayList.length, 0);

  // The dialog was dismissed: the folder is still unknown and nothing else is asked.
  await shell.chooseFolder();
  assert.equal(shell.dataFolder, null);
  assert.equal(shell.needsFolder, true);
  assert.equal(shell.busy, false, "a dismissed dialog leaves nothing busy");
  assert.equal(calls.some((call) => call.command === "choose_data_folder"), false, "a dismissed dialog chooses nothing");

  // *Use the program's folder*: the folder is taken, the day, the settings and About load.
  await shell.useProgramFolder();
  assert.equal(shell.dataFolder, "C:/Packing");
  assert.equal(shell.needsFolder, false);
  assert.equal(shell.busy, false);
  assert.deepEqual(shell.dayList, ["2026-10-07"]);
  assert.equal(shell.overview.totals.batches, 2);
  assert.equal(shell.settings.grouping, "default");
  assert.equal(shell.about.version, "0.1");
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
  assert.equal(shell.settings.grouping, "default", "the change reloads the settings");

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


test("the file, settings and printed commands reach the back end", async () => {
  let fail = "";
  const calls = useBackend({
    ...backend(),
    start_info: () => ({ today: "2026-10-07", exe_dir: "", data_folder: "C:/Data", version: "0.1" }),
    open_path: (args) => {
      if (fail === "open_path") throw "the file is gone";
      return { ...args };
    },
    show_in_folder: () => {
      if (fail === "show_in_folder") throw "the folder is gone";
    },
    settings_set: (args) => {
      if (fail === "settings_set") throw "settings.json is read only";
      return args.settings;
    },
    set_printed: () => {
      if (fail === "set_printed") throw "the batch is gone";
    },
    revert_last: () => {
      if (fail === "revert_last") throw "nothing to revert";
    },
    "plugin:dialog|message": (args) => args.buttons?.OkCancelCustom?.[0] ?? "Ok",
  });
  const shell = new Shell();
  await shell.load();
  shell.day = "2026-10-07";

  // The day's own file opens and the folder shows in the file manager.
  await shell.openFile("C:/Data/labels/2026-10-07/batch 2/1 A Sepatu ×2.pdf");
  await shell.openFolder("C:/Data/labels/2026-10-07");
  assert.deepEqual(calls.find((call) => call.command === "open_path").args, { path: "C:/Data/labels/2026-10-07/batch 2/1 A Sepatu ×2.pdf" });
  assert.deepEqual(calls.find((call) => call.command === "show_in_folder").args, { path: "C:/Data/labels/2026-10-07" });
  assert.equal(shell.error, "");

  // The settings are written and kept.
  await shell.saveSettings({ scope: "per-pdf", layouts: ["full", "pick"], grouping: "ramadhan" });
  assert.deepEqual(calls.find((call) => call.command === "settings_set").args, { settings: { scope: "per-pdf", layouts: ["full", "pick"], grouping: "ramadhan" } });
  assert.equal(shell.settings.grouping, "ramadhan");

  // The **Printed** tick of one row: written, and the batch's rows updated in place.
  const pdfs = [
    { number: 1, code: "A", name: "Sepatu", file: "1 A Sepatu ×2.pdf", orders: 2, runs: 1, printed: false },
    { number: 2, code: "B", name: "Spion", file: "2 B Spion ×2.pdf", orders: 2, runs: 1, printed: true },
  ];
  const row = { batch: 2, save_time: "2026-10-07T09:00:00+07:00", time: "09:00", grouping: "default", files: 2, orders: 4, pdfs, packing_lists: [], warnings: [] };
  await shell.setPrinted(row, pdfs[0], true);
  assert.deepEqual(pdfs.map((pdf) => pdf.printed), [true, true], "the tick is set");
  await shell.setPrinted(row, pdfs[1], false);
  assert.deepEqual(pdfs.map((pdf) => pdf.printed), [true, false], "and cleared");
  assert.deepEqual(calls.filter((call) => call.command === "set_printed").map((call) => call.args), [
    { day: "2026-10-07", batch: 2, pdf: 1, printed: true },
    { day: "2026-10-07", batch: 2, pdf: 2, printed: false },
  ]);

  // *Revert* asks first, then deletes the batch and recounts the day.
  const before = calls.filter((call) => call.command === "day_overview").length;
  await shell.revert(row);
  assert.equal(calls.filter((call) => call.command === "revert_last").length, 1);
  assert.deepEqual(calls.find((call) => call.command === "revert_last").args, { day: "2026-10-07" });
  assert.equal(calls.filter((call) => call.command === "day_overview").length, before + 1, "the day is recounted");

  // A failure of any of them shows the error line and keeps the day screen.
  for (const command of ["open_path", "show_in_folder", "settings_set", "set_printed", "revert_last"]) {
    fail = command;
    if (command === "open_path") await shell.openFile("C:/gone.pdf");
    if (command === "show_in_folder") await shell.openFolder("C:/gone");
    if (command === "settings_set") await shell.saveSettings({ scope: "whole", layouts: ["pick"], grouping: "default" });
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
  assert.deepEqual(shell.dayList, []);

  // A failing day list, settings or About leaves the day usable: the list is empty and the error
  // line carries the fault.
  const partial = new Shell();
  useBackend({
    ...backend(),
    start_info: () => ({ today: "2026-10-07", exe_dir: "", data_folder: "C:/Data", version: "0.1" }),
    days: () => {
      throw "the day folder is unreadable";
    },
    settings_get: () => {
      throw "settings.json is broken";
    },
    about: () => {
      throw "pdfium is missing";
    },
    day_overview: () => {
      throw "state.json is broken";
    },
  });
  await partial.load();
  assert.equal(partial.ready, true);
  assert.deepEqual(partial.dayList, [], "no days to step through");
  assert.equal(partial.settings, null);
  assert.equal(partial.about, null);
  assert.equal(partial.overview, null);
  assert.equal(partial.error, "pdfium is missing", "the last failure is the one shown");

  // No categories file at all is not an error.
  const fresh = new Shell();
  useBackend({
    ...backend(),
    start_info: () => ({ today: "2026-10-07", exe_dir: "", data_folder: "C:/Data", version: "0.1" }),
    rules_load: () => {
      throw "no categories.toml";
    },
  });
  await fresh.load();
  assert.equal(fresh.error, "");
  assert.equal(fresh.overlay, null);

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
