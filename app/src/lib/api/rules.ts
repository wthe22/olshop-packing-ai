// Typed wrappers around the Tauri commands for the Sort rules screen and the New batch screen's
// sort rule. The Rust side sends snake_case fields; the interfaces here match it.

import { invoke } from "@tauri-apps/api/core";

// The sort rules New batch offers (those that are on) and the one to preselect: the open draft's
// rule, else the rule of the workspace's newest batch on whichever date, else the first.
export interface SortRules {
  rules: string[];
  preselected: string;
  // The rule of the workspace's newest batch, on or off; `null` before its first batch.
  last_used: string | null;
}

export const sortRules = (): Promise<SortRules> => invoke("sort_rules");

export interface Courier {
  code: string;
  name: string;
}

// A sort rule of the file and whether New batch offers it.
export interface RuleSummary {
  name: string;
  enabled: boolean;
}

// The Sort rules screen's first load.
export interface RulesView {
  rules: RuleSummary[];
  // The whole file, for *Edit file as text*.
  text: string;
  // For the Courier box: shown by name, stored by code.
  couriers: Courier[];
  // The open draft's rule, so the screen opens on it; `null` without a draft.
  draft_rule: string | null;
}

// One label group as the file holds it; `origin` is its place in the rule.
export interface GroupView {
  code: string;
  name: string;
  filter: string | null;
  origin: number;
}

// One label group as the screen sends it back; `origin` is `null` for a group added on the screen.
export interface GroupInput {
  code: string;
  name: string;
  filter: string | null;
  origin: number | null;
}

// The live check: each group's filter error, what the whole file would fail, and the count each
// group takes in the open draft (`null` without a draft, or while something fails).
export interface RulesCheck {
  errors: (string | null)[];
  general: string | null;
  counts: number[] | null;
}

export const rulesList = (): Promise<RulesView> => invoke("rules_list");

export const rulesLoad = (rule: string): Promise<GroupView[]> => invoke("rules_load", { rule });

export const rulesCheck = (rule: string, groups: GroupInput[]): Promise<RulesCheck> =>
  invoke("rules_check", { rule, groups });

export const rulesSave = (rule: string, groups: GroupInput[]): Promise<void> =>
  invoke("rules_save", { rule, groups });

// **+ New rule**: one group `Z Lainnya`, or a copy of `copyFrom`.
export const ruleNew = (name: string, copyFrom: string | null): Promise<void> =>
  invoke("rule_new", { name, copyFrom });

export const ruleRename = (rule: string, name: string): Promise<void> =>
  invoke("rule_rename", { rule, name });

// The rule's switch: offered on New batch or not. The only rule that is on stays on.
export const ruleEnable = (rule: string, enabled: boolean): Promise<void> =>
  invoke("rule_enable", { rule, enabled });

export const ruleDelete = (rule: string): Promise<void> => invoke("rule_delete", { rule });

export const rulesCheckText = (text: string): Promise<void> => invoke("rules_check_text", { text });

export const rulesSaveText = (text: string): Promise<void> => invoke("rules_save_text", { text });

// A filter's text as boxes (the engine's parser); an error carries its line and column.
export const filterToTree = (text: string): Promise<unknown> => invoke("filter_to_tree", { text });

// Boxes as filter text, in the engine printer's one-line form.
export const treeToFilter = (tree: unknown): Promise<string> => invoke("tree_to_filter", { tree });
