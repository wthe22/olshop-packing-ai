// One label group as the Sort rules screen edits it: the values `packing-workflows.toml` holds
// plus the state of the row that shows it, and the conversion of its filter between the boxes and
// the printer's text.

import * as api from "./api/rules";
import { fromRustTree, toRustTree, type UiTree } from "./conditions";

export interface GroupEntry {
  code: string;
  name: string;
  // The filter text; `null` for the last group with no filter (it takes the rest).
  filter: string | null;
  // Its place in the rule as loaded, so the save keeps its comment and text.
  origin: number | null;
  mode: "boxes" | "text";
  tree: UiTree;
  text: string;
  error: string;
  // The boxes editor's own error (e.g. an empty box), which a recount must not clear.
  boxesError: string;
  count: number | null;
}

// The group's boxes as filter text: an empty box is an error until a row is added. Says whether
// it parsed, so the caller can recount.
export async function boxesToFilter(entry: GroupEntry): Promise<boolean> {
  try {
    entry.filter = await api.treeToFilter(toRustTree(entry.tree));
    entry.boxesError = "";
    return true;
  } catch (err) {
    entry.boxesError = String(err);
    entry.error = entry.boxesError;
    return false;
  }
}

// The group's typed filter read as boxes; on success the two views agree, so the row shows the
// boxes.
export async function textToTree(entry: GroupEntry): Promise<boolean> {
  try {
    entry.tree = fromRustTree(await api.filterToTree(entry.text));
    entry.filter = entry.text;
    entry.mode = "boxes";
    entry.boxesError = "";
    return true;
  } catch (err) {
    entry.boxesError = String(err);
    entry.error = entry.boxesError;
    return false;
  }
}
