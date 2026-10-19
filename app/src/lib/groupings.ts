// The grouping control's rules (04 › *Sets*; 08 › *2. New batch* → *The set*, 08 › *5.
// Categories*): which grouping the screens start on, and a new grouping's name. Pure, so the
// rules are tested without a window; the reading and writing is the backend (`groupings_list`,
// `new_grouping`, `delete_grouping`, `rules_*`).

/// One grouping as the data folder's list carries it (04 › *Sets*): its name (the file name
/// without `.toml`) and the file it is kept in, relative to the data folder.
export interface NamedGrouping {
  name: string;
  file: string;
}

/// The grouping every data folder has, kept in its own `categories.toml` (04 › *Sets*); mirrors
/// the engine's `Grouping::default()`.
const DEFAULT_GROUPING = "default";

/// The grouping the New batch screen preselects (08 › *2. New batch* → *The set*): the grouping
/// used last, or `default` when that grouping's file is gone and its name has left the list — the
/// fallback that keeps the control working after the grouping used last is deleted. `default` is
/// always in the list, so the fallback is always a real grouping.
export function preselectGrouping(
  groupings: NamedGrouping[],
  lastUsed: string,
): string {
  return groupings.some((entry) => entry.name === lastUsed)
    ? lastUsed
    : DEFAULT_GROUPING;
}

/// The grouping the Categories screen opens on (08 › *5. Categories*): the batch's own grouping
/// when the screen is opened with a batch (the Plan screen's *Edit categories*), else the grouping
/// used last. A batch's grouping that is no longer in the list falls back the same way.
export function categoriesGrouping(
  batchGrouping: string | null,
  groupings: NamedGrouping[],
  lastUsed: string,
): string {
  if (
    batchGrouping !== null &&
    groupings.some((entry) => entry.name === batchGrouping)
  ) {
    return batchGrouping;
  }
  return preselectGrouping(groupings, lastUsed);
}

/// A new grouping's name as a file name: trimmed, with the characters Windows forbids in a file
/// name (`\ / : * ? " < > |`) turned into `-` (04 › *Sets*); mirrors the engine's
/// `groupings::file_of`.
export function normalizeGroupingName(name: string): string {
  return name.trim().replace(/[\\/:*?"<>|]/g, "-");
}

/// Why a new grouping's name cannot be used, or `null` when it can (04 › *Sets*): `empty` for a
/// name that is blank after normalising, `reserved` for the name `default` (the data folder's own
/// `categories.toml`, which is neither created nor deleted), and `exists` for a name already in
/// the list.
export type GroupingNameProblem = "empty" | "reserved" | "exists";

export function groupingProblem(
  name: string,
  groupings: NamedGrouping[],
): GroupingNameProblem | null {
  const normalized = normalizeGroupingName(name);
  if (normalized === "") return "empty";
  if (normalized === DEFAULT_GROUPING) return "reserved";
  if (groupings.some((entry) => entry.name === normalized)) return "exists";
  return null;
}

/// The file a grouping is kept in, from the list (the grouping row's `file:` line, 08 › *5.
/// Categories*); the empty string when the name is not in the list.
export function groupingFile(groupings: NamedGrouping[], name: string): string {
  return groupings.find((entry) => entry.name === name)?.file ?? "";
}
