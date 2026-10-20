// Typed wrappers around the Tauri commands for the Categories screen and the groupings of a data
// folder (07 › *Commands between window and Rust*). The Rust side sends snake_case fields; the
// interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

// One category entry as the file holds it (08 › *5. Categories*): its values and the index it has
// in the loaded file (`origin`). The one catch-all entry of a folder without `categories.toml`
// has `origin: null`, so saving it adds the entry and writes the file.
export interface EntryView {
  code: string;
  name: string;
  when: string | null;
  origin: number | null;
}

// The Categories screen's first load (08 › *5. Categories*): the whole file text, the entries and
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

// The live check (08 › *5. Categories*): each entry's condition error, a file-level error and the
// count each entry takes in the open batch.
export interface RulesCheck {
  errors: (string | null)[];
  general: string | null;
  counts: number[] | null;
  // The file of the grouping the check ran on, relative to the data folder: `categories.toml` for
  // the grouping `default`, else `categories/<grouping name>.toml` (04 › *Sets*).
  file: string;
}

// One grouping of the data folder as the Categories screen lists it (04 › *Sets*; 08 › *5.
// Categories*): its name and the file it is kept in, relative to the data folder.
export interface GroupingView {
  name: string;
  file: string;
}

// The groupings of a data folder (04 › *Sets*): every grouping with its file, and the grouping
// used last, which the next New batch preselects (07 › *Settings*).
export interface GroupingsList {
  groupings: GroupingView[];
  // The grouping used last, from `settings.json`; `default` when the file has none.
  last_grouping: string;
}

// The grouping `default` is the data folder's own `categories.toml` (04 › *Sets*); every other
// grouping is `categories/<grouping name>.toml`. The screens send a grouping name; `default` is
// what an older screen that knows no groupings uses.
export const DEFAULT_GROUPING = "default";

// The groupings of the data folder and the grouping used last (04 › *Sets*).
export const groupingsList = (): Promise<GroupingsList> => invoke("groupings_list");

// **+ New set** (08 › *5. Categories*; 04 › *Sets*): write `categories/<name>.toml`, empty (the
// one catch-all entry) or a copy of `copyFrom`'s file. The name `default` is refused (it is the
// data folder's own `categories.toml`) and an existing file is never overwritten.
export const newGrouping = (name: string, copyFrom: string | null = null): Promise<void> =>
  invoke("new_grouping", { name, copyFrom });

// **Delete set** (08 › *5. Categories*; 04 › *Sets*): delete that grouping's file, whose
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
