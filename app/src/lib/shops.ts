// An account's Shops list: what is wrong with each name before the list is saved. The back end
// refuses the same; spaces and upper and lower case do not make two names different.

export type ShopProblem =
  | { kind: "empty" }
  | { kind: "twice" }
  | { kind: "other"; account: string };

// A shop another account lists.
export interface ListedShop {
  shop: string;
  account: string;
}

const key = (name: string) => name.replace(/\s+/g, "").toLowerCase();

export function shopProblems(names: string[], others: ListedShop[]): (ShopProblem | null)[] {
  return names.map((name, index) => {
    if (name.trim() === "") return { kind: "empty" };
    if (names.slice(0, index).some((earlier) => key(earlier) === key(name))) return { kind: "twice" };
    const other = others.find((listed) => key(listed.shop) === key(name));
    return other ? { kind: "other", account: other.account } : null;
  });
}

// Whether the fields differ from the saved list (a name's outer spaces do not count).
export function shopsChanged(saved: string[], names: string[]): boolean {
  const trimmed = names.map((name) => name.trim());
  return saved.length !== trimmed.length || saved.some((shop, i) => shop !== trimmed[i]);
}

// The shops other accounts than `account` list.
export function listedElsewhere(
  accounts: { name: string; shops: string[] }[],
  account: string | null,
): ListedShop[] {
  return accounts
    .filter((other) => other.name !== account)
    .flatMap((other) => other.shops.map((shop) => ({ shop, account: other.name })));
}
