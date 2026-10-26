// The boxes model of a condition and its conversion to and from the Rust
// `Tree`. Rust serialises a `Node` as
// `{"Group": …}` or `{"Row": …}` (serde's external tagging); the window uses a friendlier shape.

import { conditionFields, type FieldType } from "../texts";

export type GroupKind = "all" | "any";

// The operator of a text field with no value; its row has no value box.
export const IS_EMPTY = "is_empty";

// A row: one comparison `field · operator · value` with a **not** flag.
export interface UiRow {
  kind: "row";
  not: boolean;
  field: string;
  operator: string;
  // The value as typed, without quotes; turned into the field's type when sent to Rust.
  value: string;
}

// A group: **All of these** (`and`) or **Any of these** (`or`), holding rows and other groups.
export interface UiGroup {
  kind: "group";
  groupKind: GroupKind;
  not: boolean;
  children: UiNode[];
}

export type UiNode = UiRow | UiGroup;

// One condition: a root group.
export interface UiTree {
  root: UiGroup;
}

export function newGroup(groupKind: GroupKind = "all"): UiGroup {
  return { kind: "group", groupKind, not: false, children: [] };
}

export function newTree(): UiTree {
  return { root: newGroup() };
}

export function newRow(field = "name"): UiRow {
  return {
    kind: "row",
    not: false,
    field,
    operator: fieldType(field) === "text" ? "contains" : "=",
    value: "",
  };
}

// The field's type; an unknown field reads as text.
export function fieldType(field: string): FieldType {
  return conditionFields.find((entry) => entry.value === field)?.type ?? "text";
}


interface RustGroup {
  kind: GroupKind;
  not: boolean;
  children: RustNode[];
}

type RustNode = { Group: RustGroup } | { Row: RustRow };

interface RustRow {
  not: boolean;
  field: string;
  operator: string;
  value: RustValue;
}

type RustValue = { text: string } | { number: number } | { time: string } | "no_value";

export function fromRustTree(raw: unknown): UiTree {
  const tree = raw as { root: RustGroup };
  return { root: fromRustGroup(tree.root) };
}

function fromRustGroup(group: RustGroup): UiGroup {
  return {
    kind: "group",
    groupKind: group.kind,
    not: group.not,
    children: group.children.map(fromRustNode),
  };
}

function fromRustNode(node: RustNode): UiNode {
  if ("Group" in node) {
    return fromRustGroup(node.Group);
  }
  const row = node.Row;
  return {
    kind: "row",
    not: row.not,
    field: row.field,
    operator: row.operator,
    value: valueString(row.value),
  };
}

function valueString(value: RustValue): string {
  if (value === "no_value") return "";
  if ("text" in value) return value.text;
  if ("number" in value) return String(value.number);
  return value.time;
}

export function toRustTree(tree: UiTree): unknown {
  return { root: toRustGroup(tree.root) };
}

function toRustGroup(group: UiGroup): RustGroup {
  return {
    kind: group.groupKind,
    not: group.not,
    children: group.children.map(toRustNode),
  };
}

function toRustNode(node: UiNode): RustNode {
  if (node.kind === "group") {
    return { Group: toRustGroup(node) };
  }
  return {
    Row: {
      not: node.not,
      field: node.field,
      operator: node.operator,
      value: rowValue(node),
    },
  };
}

function rowValue(row: UiRow): RustValue {
  if (row.operator === IS_EMPTY) return "no_value";
  const type = fieldType(row.field);
  if (type === "number") {
    return { number: Number.parseInt(row.value || "0", 10) || 0 };
  }
  if (type === "time") {
    return { time: row.value };
  }
  return { text: row.value };
}
