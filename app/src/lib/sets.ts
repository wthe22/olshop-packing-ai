// The set control's rules (04 › *Sets*; 08 › *2. New batch* → *The set*, 08 › *5. Categories*):
// which set the screens start on, and a new set's name. Pure, so the rules are tested without a
// window; the reading and writing is the backend (`sets_list`, `new_set`, `delete_set`, `rules_*`).

/// One set as the data folder's list carries it (04 › *Sets*): its name (the file name without
/// `.toml`) and the file it is kept in, relative to the data folder.
export interface NamedSet {
  name: string;
  file: string;
}

/// The set every data folder has, kept in its own `categories.toml` (04 › *Sets*); mirrors the
/// engine's `sets::DEFAULT_SET`.
const DEFAULT_SET = "default";

/// The set the New batch screen preselects (08 › *2. New batch* → *The set*): the set used last,
/// or `default` when that set's file is gone and its name has left the list — the fallback that
/// keeps the control working after the set used last is deleted. `default` is always in the list,
/// so the fallback is always a real set.
export function preselectSet(sets: NamedSet[], lastUsed: string): string {
  return sets.some((set) => set.name === lastUsed) ? lastUsed : DEFAULT_SET;
}

/// The set the Categories screen opens on (08 › *5. Categories*): the batch's own set when the
/// screen is opened with a batch (the Plan screen's *Edit categories*), else the set used last.
/// A batch's set that is no longer in the list falls back the same way.
export function categoriesSet(
  batchSet: string | null,
  sets: NamedSet[],
  lastUsed: string,
): string {
  if (batchSet !== null && sets.some((set) => set.name === batchSet)) return batchSet;
  return preselectSet(sets, lastUsed);
}

/// A new set's name as a file name: trimmed, with the characters Windows forbids in a file name
/// (`\ / : * ? " < > |`) turned into `-` (04 › *Sets*); mirrors the engine's `sets::sanitize_name`.
export function normalizeSetName(name: string): string {
  return name.trim().replace(/[\\/:*?"<>|]/g, "-");
}

/// Why a new set's name cannot be used, or `null` when it can (04 › *Sets*): `empty` for a name
/// that is blank after normalising, `reserved` for the name `default` (the data folder's own
/// `categories.toml`, which is neither created nor deleted), and `exists` for a name already in
/// the list.
export type SetNameProblem = "empty" | "reserved" | "exists";

export function setProblem(name: string, sets: NamedSet[]): SetNameProblem | null {
  const normalized = normalizeSetName(name);
  if (normalized === "") return "empty";
  if (normalized === DEFAULT_SET) return "reserved";
  if (sets.some((set) => set.name === normalized)) return "exists";
  return null;
}

/// The file a set is kept in, from the list (the set row's `file:` line, 08 › *5. Categories*);
/// the empty string when the name is not in the list.
export function setFile(sets: NamedSet[], name: string): string {
  return sets.find((set) => set.name === name)?.file ?? "";
}
