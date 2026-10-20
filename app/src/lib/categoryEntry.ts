// One category entry as the Categories screen edits it:
// the values `categories.toml` holds plus the state of the row that shows it, and the conversion of
// the entry's condition between the boxes editor's tree and the printer's text.

import * as api from "./api/categories";
import { fromRustTree, toRustTree, type UiTree } from "./conditions";

export interface CatEntry {
  code: string;
  name: string;
  // The condition text; `null` for the last entry with no condition (it takes the rest).
  when: string | null;
  // The index the entry had in the loaded file, so the save keeps its comment and text.
  origin: number | null;
  mode: "boxes" | "text";
  tree: UiTree;
  text: string;
  error: string;
  // The boxes editor's own error (e.g. an empty group), which a recount must not clear.
  boxesError: string;
  count: number | null;
}

// The entry's boxes as the condition text: an empty group is an error until a row is added. Says
// whether it parsed, so the caller can recount.
export async function boxesToCondition(entry: CatEntry): Promise<boolean> {
  try {
    entry.when = await api.treeToCondition(toRustTree(entry.tree));
    entry.boxesError = "";
    return true;
  } catch (err) {
    entry.boxesError = String(err);
    entry.error = entry.boxesError;
    return false;
  }
}

// The entry's typed condition read as boxes; on success the two views agree, so the row shows the
// boxes.
export async function textToTree(entry: CatEntry): Promise<boolean> {
  try {
    entry.tree = fromRustTree(await api.conditionToTree(entry.text));
    entry.when = entry.text;
    entry.mode = "boxes";
    entry.boxesError = "";
    return true;
  } catch (err) {
    entry.boxesError = String(err);
    entry.error = entry.boxesError;
    return false;
  }
}
