// The boxes of a condition and the shape Rust sends and takes: each value kind goes over and comes
// back, and an *is empty* row carries no value. Pure logic; no window opens.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import "./tsModule.mjs";

const { fromRustTree, toRustTree } = await import("../src/lib/conditions.ts");

const row = (field, operator, value, not = false) => ({ Row: { not, field, operator, value } });

test("a tree from Rust comes back to Rust unchanged", () => {
  const rust = {
    root: {
      kind: "all",
      not: false,
      children: [
        row("name", "contains", { text: "sepatu" }),
        row("courier", "is_empty", "no_value", true),
        {
          Group: {
            kind: "any",
            not: true,
            children: [
              row("total_quantity", ">=", { number: 2 }),
              row("ship_by", "<", { time: "2026-10-07" }),
            ],
          },
        },
      ],
    },
  };
  const ui = fromRustTree(rust);
  assert.deepEqual(ui.root.children[1], {
    kind: "row",
    not: true,
    field: "courier",
    operator: "is_empty",
    value: "",
  });
  assert.deepEqual(toRustTree(ui), rust);
});

test("an is-empty row sends no value, whatever was typed before", () => {
  const ui = fromRustTree({
    root: { kind: "all", not: false, children: [row("tracking_id", "equals", { text: "JY" })] },
  });
  ui.root.children[0].operator = "is_empty";
  assert.equal(toRustTree(ui).root.children[0].Row.value, "no_value");
});
