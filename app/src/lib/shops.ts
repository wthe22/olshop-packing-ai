// The Shops list of Settings › Workspaces: what is wrong with each name before the list is saved.
// The back end refuses the same; spaces and upper and lower case do not make two names different.

export type ShopProblem = "empty" | "twice";

export function shopProblems(names: string[]): (ShopProblem | null)[] {
  const key = (name: string) => name.replace(/\s+/g, "").toLowerCase();
  return names.map((name, index) => {
    if (name.trim() === "") return "empty";
    return names.slice(0, index).some((other) => key(other) === key(name)) ? "twice" : null;
  });
}
