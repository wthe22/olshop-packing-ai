// The *No picks yet* line (Q-UI-4): the picks are unconfigured while the list holds no pick but
// the one catch-all entry a data folder without `categories.toml` starts with (07 › *Inputs*), so
// every order goes into one PDF set. The Categories screen shows the line for the entries it is
// editing; the Plan screen's `no_picks` comes from the back end, which applies the same rule to
// the plan (`rules::unconfigured`).

/// One entry as the rule needs it: its condition text, `null` when it has none.
export interface EntryCondition {
  when: string | null;
}

/// Whether the picks are unconfigured: exactly one entry and it has no condition.
export function unconfigured(entries: EntryCondition[]): boolean {
  return entries.length === 1 && entries[0].when === null;
}
