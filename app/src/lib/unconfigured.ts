// The *No groups yet* line: a sort rule whose only group has no filter sends every order into
// one PDF. The Sort rules screen shows the line for the groups it is editing; the Plan screen's
// `no_groups` comes from the back end, which applies the same rule to the plan.

// One group as the rule needs it: its filter text, `null` when it has none.
export interface GroupFilter {
  filter: string | null;
}

// Whether the rule has no groups yet: exactly one group and it has no filter.
export function unconfigured(groups: GroupFilter[]): boolean {
  return groups.length === 1 && groups[0].filter === null;
}
