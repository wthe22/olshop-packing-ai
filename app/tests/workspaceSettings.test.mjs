// Settings › Workspaces against a faked back end and a stand-in for the shell: the list, a rename
// (refused, of another workspace, of the open one), a delete (asked first, of the open one, the
// last one). No window opens and nothing is written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, useRunes } from "./tsModule.mjs";
import { answerQuestions } from "./answers.mjs";
import { texts } from "../src/texts.ts";

useRunes();
const { WorkspaceSettings } = await import("../src/lib/workspaceSettings.svelte.ts");

// What the settings ask of the shell, recorded.
function fakeShell(open, keepDraft = true) {
  const did = [];
  return {
    did,
    workspace: open,
    leaveBatch: async () => (did.push("leave"), keepDraft),
    refreshPanel: async () => did.push("panel"),
    workspaceRenamed: async (name) => did.push(`renamed ${name}`),
    workspaceDeleted: async (name) => did.push(`deleted, open ${name}`),
  };
}

const list = [
  { name: "Toko Contoh", dates: 3, open: true },
  { name: "Toko Lain", dates: 0, open: false },
];

test("a rename is refused beside the field, or followed by the shell", async () => {
  const calls = useBackend({
    workspaces_overview: () => list,
    workspace_rename: ({ newName }) => {
      if (newName === "toko lain") throw new Error('"toko lain": another workspace has that name');
    },
  });
  const shell = fakeShell("Toko Contoh");
  const settings = new WorkspaceSettings(shell);
  await settings.load();
  assert.deepEqual(settings.list, list);

  assert.match(await settings.rename("Toko Contoh", "toko lain"), /another workspace/);
  assert.equal(await settings.rename("Toko Lain", "Toko Dua"), null);
  assert.equal(await settings.rename("Toko Contoh", "Toko Satu"), null);
  assert.deepEqual(shell.did, ["leave", "panel", "leave", "renamed Toko Satu"], "only the open one asks about a draft");
  assert.deepEqual(calls.filter((call) => call.command === "workspace_rename").at(-1).args, {
    name: "Toko Contoh",
    newName: "Toko Satu",
  });

  const kept = fakeShell("Toko Contoh", false);
  assert.equal(await new WorkspaceSettings(kept).rename("Toko Contoh", "Toko Satu"), null);
  assert.equal(calls.filter((call) => call.command === "workspace_rename").length, 3, "the draft was kept");
});

test("a delete asks first and the shell opens what the back end opened", async () => {
  let open = "Toko Lain";
  const calls = useBackend({
    workspaces_overview: () => list,
    workspace_delete: ({ name }) => {
      if (name === "Toko Tiada") throw new Error("Toko Tiada is not there");
      return open;
    },
  });
  let yes = false;
  const asked = answerQuestions(() => yes);
  const shell = fakeShell("Toko Contoh");
  const settings = new WorkspaceSettings(shell);

  await settings.remove(list[0]);
  assert.deepEqual(asked, [texts.settings.askDelete("Toko Contoh", 3)]);
  assert.equal(calls.some((call) => call.command === "workspace_delete"), false, "Cancel deletes nothing");

  yes = true;
  await settings.remove(list[1]);
  await settings.remove(list[0]);
  open = null;
  await settings.remove(list[0]);
  assert.deepEqual(shell.did, ["panel", "leave", "deleted, open Toko Lain", "leave", "deleted, open null"]);

  await settings.remove({ name: "Toko Tiada", dates: 0, open: false });
  assert.match(settings.error, /not there/);
  assert.deepEqual(
    [texts.settings.askDelete("Toko Lain", 0).body, texts.settings.dates(0), texts.settings.dates(1)],
    ["Its folder goes to the Recycle Bin.", "No dates yet", "1 date"],
  );
});
