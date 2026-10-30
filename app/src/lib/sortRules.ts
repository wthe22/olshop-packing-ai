// The sort rule a screen starts on, why a new rule's name cannot be used, and why a rule cannot be
// switched off or deleted. Pure, so the rules are tested without a window; the reading and
// writing is the back end (`sort_rules`, `rules_*`, `rule_*`).

import type { RuleSummary } from "./api/rules";

// The rule the New batch screen preselects: the back end's choice (the open batch's rule, else
// the one used last) while it is in the list of rules that are on, else the first of them.
export function preselectRule(rules: string[], preselected: string): string {
  return rules.includes(preselected) ? preselected : (rules[0] ?? "");
}

// The rule the Sort rules screen opens on: the open draft's rule (the Plan screen's *Edit sort
// rule*), else the rule the next batch preselects.
export function openingRule(rules: string[], draftRule: string | null, preselected: string): string {
  return draftRule !== null && rules.includes(draftRule) ? draftRule : preselectRule(rules, preselected);
}

// Why a rule name cannot be used, or `null` when it can: blank, or already a rule's (upper and
// lower case count as the same).
export type RuleNameProblem = "empty" | "exists";

export function ruleNameProblem(name: string, rules: string[]): RuleNameProblem | null {
  const trimmed = name.trim();
  if (trimmed === "") return "empty";
  if (rules.some((rule) => rule.toLowerCase() === trimmed.toLowerCase())) return "exists";
  return null;
}

// Why a rule must stay as it is, or `null` when it can be switched off or deleted: the last rule
// cannot be deleted, and the only rule that is on stays on, since a new batch needs one.
export type RuleKept = "last" | "only-on";

export function switchOffProblem(rules: RuleSummary[], name: string): RuleKept | null {
  return onlyOneOn(rules, name) ? "only-on" : null;
}

export function deleteProblem(rules: RuleSummary[], name: string): RuleKept | null {
  if (rules.length < 2) return "last";
  return onlyOneOn(rules, name) ? "only-on" : null;
}

function onlyOneOn(rules: RuleSummary[], name: string): boolean {
  const on = rules.filter((rule) => rule.enabled);
  return on.length === 1 && on[0].name === name;
}
