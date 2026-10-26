// The sort rule a screen starts on, and why a new rule's name cannot be used. Pure, so the rules
// are tested without a window; the reading and writing is the back end (`sort_rules`, `rules_*`,
// `rule_*`).

// The rule the New batch screen preselects: the rule of the newest batch while it is in the list,
// else the first rule.
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
