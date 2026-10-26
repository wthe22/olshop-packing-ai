// Typed wrappers around the Tauri commands for the Categories screen and the groupings of a data
// folder. The Rust side sends snake_case fields; the
// interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

// One category entry as the file holds it: its values and the index it has
// in the loaded file (`origin`). The one catch-all entry of a folder without `categories.toml`
// has `origin: null`, so saving it adds the entry and writes the file.
export interface EntryView {
  code: string;
  name: string;
  when: string | null;
  origin: number | null;
}

// The Categories screen's first load: the whole file text, the entries and
// any file-level error.
export interface RulesLoad {
  text: string;
  entries: EntryView[];
  error: string | null;
}

// One entry the screen sends back: its values and the index it had in the loaded file (`origin`),
// so the save can keep its comment and unchanged condition text; `null` for a new entry.
export interface EntryInput {
  code: string;
  name: string;
  when: string | null;
  origin: number | null;
}

// The live check: each entry's condition error, a file-level error and the
// count each entry takes in the open batch.
export interface RulesCheck {
  errors: (string | null)[];
  general: string | null;
  counts: number[] | null;
  // The file of the grouping the check ran on, relative to the data folder: `categories.toml` for
  // the grouping `default`, else `categories/<grouping name>.toml`.
  file: string;
}

// One grouping of the data folder as the Categories screen lists it: its name and the file it is kept in, relative to the data folder.
export interface GroupingView {
  name: string;
  file: string;
}

// The groupings of a data folder: every grouping with its file, and the grouping
// used last, which the next New batch preselects.
export interface GroupingsList {
  groupings: GroupingView[];
  // The grouping used last, from `settings.json`; `default` when the file has none.
  last_grouping: string;
}

// The grouping `default` is the data folder's own `categories.toml`; every other
// grouping is `categories/<grouping name>.toml`. The screens send a grouping name; `default` is
// what an older screen that knows no groupings uses.
export const DEFAULT_GROUPING = "default";

// The sort rules of `packing-workflows.toml`, shown as the New batch screen's groupings until the
// screens take the sort rule's word; the rule of the newest batch is preselected.
export const groupingsList = async (): Promise<GroupingsList> => {
  const rules = await invoke<{ rules: string[]; preselected: string }>("sort_rules");
  return {
    groupings: rules.rules.map((name) => ({ name, file: "packing-workflows.toml" })),
    last_grouping: rules.preselected,
  };
};

// **+ New set**: write `categories/<name>.toml`, empty (the
// one catch-all entry) or a copy of `copyFrom`'s file. The name `default` is refused (it is the
// data folder's own `categories.toml`) and an existing file is never overwritten.
export const newGrouping = (name: string, copyFrom: string | null = null): Promise<void> =>
  invoke("new_grouping", { name, copyFrom });

// **Delete set**: delete that grouping's file, whose
// categories then fall back to the one catch-all entry. The grouping `default` cannot be
// deleted.
export const deleteGrouping = (name: string): Promise<void> =>
  invoke("delete_grouping", { name });

export const rulesLoad = (grouping = DEFAULT_GROUPING): Promise<RulesLoad> =>
  invoke("rules_load", { grouping });

export const rulesCheck = (
  entries: EntryInput[],
  grouping = DEFAULT_GROUPING,
): Promise<RulesCheck> => invoke("rules_check", { entries, grouping });

export const rulesCheckText = (text: string, grouping = DEFAULT_GROUPING): Promise<void> =>
  invoke("rules_check_text", { text, grouping });

export const rulesSave = (
  entries: EntryInput[],
  grouping = DEFAULT_GROUPING,
): Promise<void> => invoke("rules_save", { entries, grouping });

export const rulesSaveText = (text: string, grouping = DEFAULT_GROUPING): Promise<void> =>
  invoke("rules_save_text", { text, grouping });

// A condition text → boxes tree (the engine's parser); an error carries line/col.
export const conditionToTree = (text: string): Promise<unknown> =>
  invoke("condition_to_tree", { text });

// A boxes tree → the printer's canonical condition text.
export const treeToCondition = (tree: unknown): Promise<string> =>
  invoke("tree_to_condition", { tree });
