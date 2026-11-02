// The look for what other copies of the app wrote, with a faked back end: the Day screen and the
// panel are read again only where something changed, and a workspace another copy renamed or
// deleted is followed.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend } from "./tsModule.mjs";
import { Shell, backend } from "./shellKit.mjs";
import { texts } from "../src/texts.ts";

const { lookForChanges } = await import("../src/lib/changes.ts");

const startInfo = () => ({
  today: "2026-10-07",
  exe_dir: "C:/Packing",
  data_folder: "C:/Data",
  workspace: "Toko Contoh",
  version: "0.3.3",
});

const nothing = { day: false, panel: false, gone: null };

test("a look reads again only what another copy changed", async () => {
  let found = nothing;
  const calls = useBackend(backend({ start_info: startInfo, data_changes: () => found }));
  const shell = new Shell();
  await shell.load();
  const count = (command) => calls.filter((call) => call.command === command).length;
  const before = [count("day_overview"), count("workspaces_list")];

  await lookForChanges(shell);
  assert.deepEqual([count("day_overview"), count("workspaces_list")], before, "nothing changed");

  found = { ...nothing, day: true };
  await lookForChanges(shell);
  assert.deepEqual([count("day_overview"), count("workspaces_list")], [before[0] + 1, before[1]]);

  found = { ...nothing, panel: true };
  await lookForChanges(shell);
  assert.deepEqual([count("day_overview"), count("workspaces_list")], [before[0] + 1, before[1] + 1]);
});

test("a workspace another copy renamed is followed and the batch being prepared dropped", async () => {
  let nowOpen = "Toko Baru";
  const calls = useBackend(
    backend({
      start_info: startInfo,
      discard_draft: () => undefined,
      data_changes: () => ({ day: true, panel: true, gone: { workspace: "Toko Contoh", now_open: nowOpen } }),
    }),
  );
  const shell = new Shell();
  await shell.load();
  shell.screen = "plan";
  shell.draftOpen = true;
  shell.day = "2026-10-05";

  await lookForChanges(shell);
  assert.deepEqual([shell.workspace, shell.screen, shell.day, shell.draftOpen], ["Toko Baru", "day", "2026-10-07", false]);
  assert.equal(shell.error, texts.day.workspaceGone("Toko Contoh"));
  assert.equal(calls.filter((call) => call.command === "discard_draft").length, 1);

  nowOpen = null;
  await lookForChanges(shell);
  assert.deepEqual([shell.workspace, shell.needsWorkspace], [null, true], "none left: the first name is asked");
});
