// The look for what other copies of the app wrote, with a faked back end: the Day screen and the
// panel are read again only where something changed, and a workspace another copy renamed or
// deleted is followed.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend } from "./tsModule.mjs";
import { Shell, backend } from "./shellKit.mjs";

const { lookForChanges } = await import("../src/lib/changes.ts");

const startInfo = () => ({
  today: "2026-10-07",
  exe_dir: "C:/Packing",
  data_folder: "C:/Data",
  version: "0.3.3",
});

const nothing = { day: false, panel: false };

test("a look reads again only what another copy changed", async () => {
  let found = nothing;
  const calls = useBackend(backend({ start_info: startInfo, data_changes: () => found }));
  const shell = new Shell();
  await shell.load();
  const count = (command) => calls.filter((call) => call.command === command).length;
  const before = [count("day_overview"), count("panel_dates")];

  await lookForChanges(shell);
  assert.deepEqual([count("day_overview"), count("panel_dates")], before, "nothing changed");

  found = { ...nothing, day: true };
  await lookForChanges(shell);
  assert.deepEqual([count("day_overview"), count("panel_dates")], [before[0] + 1, before[1]]);

  found = { ...nothing, panel: true };
  await lookForChanges(shell);
  assert.deepEqual([count("day_overview"), count("panel_dates")], [before[0] + 1, before[1] + 1]);
});
