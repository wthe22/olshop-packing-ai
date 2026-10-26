// One label group's filter converted between the boxes editor's tree and the printer's text,
// with a faked back end. No window opens and nothing is written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend } from "./tsModule.mjs";

const { boxesToFilter, textToTree } = await import("../src/lib/groupEntry.ts");

// One entry as the Sort rules screen holds it: the file's values and the row's own state.
function entry(overrides = {}) {
  return {
    code: "A",
    name: "Sepatu",
    filter: null,
    origin: null,
    mode: "boxes",
    tree: { root: { kind: "group", groupKind: "all", not: false, children: [] } },
    text: "",
    error: "",
    boxesError: "",
    count: null,
    ...overrides,
  };
}

const row = (not, field, operator, value) => ({
  kind: "row",
  not,
  field,
  operator,
  value,
});

test("the boxes become the file's filter text, and a refusal stays on the row", async () => {
  const calls = useBackend({ tree_to_filter: () => 'name contains "sepatu"' });
  const boxes = entry({
    filter: "an older condition",
    error: "an older error",
    boxesError: "an older boxes error",
    tree: {
      root: {
        kind: "group",
        groupKind: "all",
        not: false,
        children: [row(false, "name", "contains", "sepatu")],
      },
    },
  });

  assert.equal(await boxesToFilter(boxes), true, "the boxes parsed");
  assert.equal(boxes.filter, 'name contains "sepatu"', "the file's text is what Rust printed");
  assert.equal(boxes.boxesError, "", "the boxes editor's error is cleared");
  assert.equal(boxes.count, null, "the caller recounts");
  // The typed text goes to Rust as the field's own type: a name is text.
  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0], {
    command: "tree_to_filter",
    args: {
      tree: {
        root: {
          kind: "all",
          not: false,
          children: [
            { Row: { not: false, field: "name", operator: "contains", value: { text: "sepatu" } } },
          ],
        },
      },
    },
  });

  // An empty group, which Rust refuses: the row shows the refusal and nothing is written.
  useBackend({
    tree_to_filter: () => {
      throw "line 1, col 1: a group needs a condition";
    },
  });
  const empty = entry({ mode: "boxes" });
  assert.equal(await boxesToFilter(empty), false, "the empty group is an error");
  assert.equal(empty.boxesError, "line 1, col 1: a group needs a condition");
  assert.equal(empty.error, "line 1, col 1: a group needs a condition", "the row shows it too");
  assert.equal(empty.filter, null, "the file's text is left alone");
});

test("the typed text becomes boxes, and a text Rust refuses keeps the row in text mode", async () => {
  const calls = useBackend({
    filter_to_tree: () => ({
      root: {
        kind: "all",
        not: false,
        children: [{ Row: { not: true, field: "total_quantity", operator: ">=", value: { number: 2 } } }],
      },
    }),
  });
  const text = entry({
    mode: "text",
    text: 'not total_quantity >= 2',
    filter: null,
    boxesError: "an older boxes error",
  });

  assert.equal(await textToTree(text), true, "the text parsed");
  assert.equal(text.mode, "boxes", "the row shows the boxes now");
  assert.equal(text.filter, text.text, "the two views agree");
  assert.equal(text.boxesError, "");
  assert.deepEqual(text.tree.root.children[0], row(true, "total_quantity", ">=", "2"));
  assert.equal(text.tree.root.groupKind, "all");
  assert.deepEqual(calls[0], {
    command: "filter_to_tree",
    args: { text: "not total_quantity >= 2" },
  });

  // A condition Rust cannot read: the row stays in text mode and shows the refusal.
  useBackend({
    filter_to_tree: () => {
      throw "line 1, col 5: unknown field \"size\"";
    },
  });
  const bad = entry({ mode: "text", text: 'size equals "L"' });
  assert.equal(await textToTree(bad), false, "the bad text is refused");
  assert.equal(bad.mode, "text", "the row is still the text editor");
  assert.equal(bad.boxesError, 'line 1, col 5: unknown field "size"');
  assert.equal(bad.error, bad.boxesError);
  assert.equal(bad.tree.root.children.length, 0, "no boxes were built");
});

test("each field's own value type travels both ways, nested groups included", async () => {
  const calls = useBackend({ tree_to_filter: () => "the printed condition" });
  const boxes = entry({
    tree: {
      root: {
        kind: "group",
        groupKind: "any",
        not: true,
        children: [
          {
            kind: "group",
            groupKind: "all",
            not: false,
            children: [row(false, "name", "contains", "sepatu")],
          },
          row(false, "line_quantity", ">=", "2"),
          row(false, "distinct_items", "=", ""),
          row(false, "ship_by", "equals", "2026-10-07T10:00:00+07:00"),
          row(false, "paid_time", "equals", "2026-10-07"),
        ],
      },
    },
  });

  assert.equal(await boxesToFilter(boxes), true);
  assert.equal(boxes.filter, "the printed condition");
  // A number field carries a number, a time field carries its text, an unknown field is text.
  assert.deepEqual(calls[0].args.tree, {
    root: {
      kind: "any",
      not: true,
      children: [
        {
          Group: {
            kind: "all",
            not: false,
            children: [
              { Row: { not: false, field: "name", operator: "contains", value: { text: "sepatu" } } },
            ],
          },
        },
        { Row: { not: false, field: "line_quantity", operator: ">=", value: { number: 2 } } },
        { Row: { not: false, field: "distinct_items", operator: "=", value: { number: 0 } } },
        {
          Row: {
            not: false,
            field: "ship_by",
            operator: "equals",
            value: { time: "2026-10-07T10:00:00+07:00" },
          },
        },
        { Row: { not: false, field: "paid_time", operator: "equals", value: { text: "2026-10-07" } } },
      ],
    },
  });

  // And back: the same shapes, with the group's own kind and **not** kept.
  useBackend({
    filter_to_tree: () => ({
      root: {
        kind: "any",
        not: true,
        children: [
          {
            Group: {
              kind: "all",
              not: false,
              children: [
                { Row: { not: false, field: "name", operator: "contains", value: { text: "sepatu" } } },
              ],
            },
          },
          { Row: { not: false, field: "line_quantity", operator: ">=", value: { number: 2 } } },
          {
            Row: {
              not: true,
              field: "ship_by",
              operator: "equals",
              value: { time: "2026-10-07T10:00:00+07:00" },
            },
          },
        ],
      },
    }),
  });
  const back = entry({ mode: "text", text: "the typed condition" });
  assert.equal(await textToTree(back), true);
  assert.equal(back.tree.root.groupKind, "any");
  assert.equal(back.tree.root.not, true);
  assert.deepEqual(back.tree.root.children, [
    {
      kind: "group",
      groupKind: "all",
      not: false,
      children: [row(false, "name", "contains", "sepatu")],
    },
    row(false, "line_quantity", ">=", "2"),
    row(true, "ship_by", "equals", "2026-10-07T10:00:00+07:00"),
  ]);
});
