// The version shown in one place: the title bar's brand text is the program's name and the shown
// version ("Packing v0.1"), and `app/package.json`'s `version` (npm's field, shown nowhere) is kept
// equal to the workspace `version` in the root `Cargo.toml` — the one place — so a release cannot
// drift.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import { texts, brandText } from "../src/texts.ts";

test("the brand text carries the version, and the two files that hold it agree", () => {
  // The same line as the OS window title: the program's name, then the version, in that order.
  assert.equal(brandText("v0.1"), "Packing v0.1");
  assert.equal(brandText("v0.1"), `${texts.title} v0.1`);

  // The workspace `version` in the root `Cargo.toml`, `../Cargo.toml` beside `app/package.json`,
  // is the one place; `app/package.json`'s own `version` is npm's field and is kept equal to it.
  const appDir = new URL("..", import.meta.url); // app/
  const cargo = readFileSync(new URL("../Cargo.toml", appDir), "utf8");
  const workspace = cargo.match(/^\s*version\s*=\s*"([^"]+)"/m);
  assert.ok(workspace, "the workspace version is in the root Cargo.toml");
  const pkg = JSON.parse(readFileSync(new URL("package.json", appDir), "utf8"));
  assert.equal(pkg.version, workspace[1]);
});
