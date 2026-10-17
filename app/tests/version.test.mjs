// The version shown in one place (08 › *Window frame*, 08 › *6. Settings* › *The version*): the
// title bar's brand text is the program's name and the shown version ("Packing v0.1"), and
// `app/package.json`'s `version` (npm's field, shown nowhere) is kept equal to the workspace
// `version` in the root `Cargo.toml` — the one place — so a release cannot drift.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import { texts, brandText } from "../src/texts.ts";

test("the title bar's brand text is the program's name and its version", () => {
  // 08 › *Window frame*: the same line as the OS window title.
  assert.equal(brandText("v0.1"), "Packing v0.1");
  // The program's name then the version, in that order.
  assert.equal(brandText("v0.1"), `${texts.title} v0.1`);
});

test("app/package.json's version equals the workspace version", () => {
  // The workspace `version` in the root `Cargo.toml` is the one place (08 › *6. Settings* ›
  // *The version*); `app/package.json`'s own `version` is npm's field and is kept equal to it.
  // The workspace `version` lives in the root `Cargo.toml`, `../Cargo.toml` beside
  // `app/package.json`.
  const appDir = new URL("..", import.meta.url); // app/
  const cargo = readFileSync(new URL("../Cargo.toml", appDir), "utf8");
  const workspace = cargo.match(/^\s*version\s*=\s*"([^"]+)"/m);
  assert.ok(workspace, "the workspace version is in the root Cargo.toml");
  const pkg = JSON.parse(readFileSync(new URL("package.json", appDir), "utf8"));
  assert.equal(pkg.version, workspace[1]);
});
