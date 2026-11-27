// The account question's rows and the answer they make: the account each new shop goes to, kept
// with that account. Every row has one list: the accounts, the new accounts other rows name, and
// *New account*.

import type { ShopAccount, ShopAnswer, ShopPromptView } from "./api/batch";
import { texts } from "../texts";

// The account chosen for a row: none yet, an account already there, a new one this row names, or
// whatever account the row `key` gets.
export type AccountChoice =
  | { kind: "none" }
  | { kind: "listed"; account: string }
  | { kind: "new"; name: string }
  | { kind: "as"; key: string };

export interface AccountRow {
  key: string;
  // The shop name no account has.
  name: string;
  orders: number;
  choice: AccountChoice;
}

// A choice of the list: its value and what it shows.
export interface ListEntry {
  value: string;
  text: string;
}

// Where a row's choice lands: no account yet, an account already there, or the new account the row
// `owner` names.
type Resolved = { kind: "none" } | { kind: "listed"; account: string } | { kind: "new"; name: string; owner: string };

const shopKey = (name: string) => `shop:${name}`;

// The rows. With no account yet a shop starts on a new account of its name.
export function accountRows(view: ShopPromptView): AccountRow[] {
  return view.shops.map(
    (shop): AccountRow => ({
      key: shopKey(shop.name),
      name: shop.name,
      orders: shop.orders,
      choice: view.accounts.length === 0 ? { kind: "new", name: shop.name } : { kind: "none" },
    }),
  );
}

// Follows `as` to the row it names; the depth bound ends a loop of rows that follow each other.
function resolve(rows: AccountRow[], row: AccountRow, depth = 0): Resolved {
  const choice = row.choice;
  if (choice.kind === "listed") return { kind: "listed", account: choice.account };
  if (choice.kind === "new") {
    const name = choice.name.trim();
    return name === "" ? { kind: "none" } : { kind: "new", name, owner: row.key };
  }
  const target = choice.kind === "as" ? rows.find((other) => other.key === choice.key) : undefined;
  return target && depth < rows.length ? resolve(rows, target, depth + 1) : { kind: "none" };
}

// The list of `row`: the accounts, the new accounts the other rows name, and *New account*.
export function listOf(rows: AccountRow[], row: AccountRow, accounts: string[]): ListEntry[] {
  const made = rows.flatMap((other) =>
    other !== row && other.choice.kind === "new" && other.choice.name.trim() !== ""
      ? [{ value: `as:${other.key}`, text: texts.shops.made(other.choice.name.trim()) }]
      : [],
  );
  return [
    ...accounts.map((account) => ({ value: `listed:${account}`, text: account })),
    ...made,
    { value: "new", text: texts.shops.newAccount },
  ];
}

// The list's value for `row`: "" while it has no account.
export function valueOf(rows: AccountRow[], row: AccountRow): string {
  if (row.choice.kind === "new") return "new";
  const resolved = resolve(rows, row);
  if (resolved.kind === "listed") return `listed:${resolved.account}`;
  if (resolved.kind === "new") return `as:${resolved.owner}`;
  return "";
}

// The choice a list value makes; *New account* starts on the shop's name.
export function choiceOf(row: AccountRow, value: string): AccountChoice {
  if (value === "new") return { kind: "new", name: row.name };
  if (value.startsWith("listed:")) return { kind: "listed", account: value.slice("listed:".length) };
  if (value.startsWith("as:")) return { kind: "as", key: value.slice("as:".length) };
  return { kind: "none" };
}

// Whether every row has an account and every new account a name.
export const canContinue = (rows: AccountRow[]): boolean => rows.every((row) => resolve(rows, row).kind !== "none");

// The answer of the rows; `canContinue` is true before it is sent. A new account is `new` on the row
// that names it and `listed` on the rows that take it, so it is made once.
export function shopAnswer(rows: AccountRow[]): ShopAnswer {
  const accountOf = (row: AccountRow): ShopAccount | null => {
    const resolved = resolve(rows, row);
    if (resolved.kind === "listed") return { listed: resolved.account };
    if (resolved.kind === "new") return resolved.owner === row.key ? { new: resolved.name } : { listed: resolved.name };
    return null;
  };
  const answer: ShopAnswer = { shops: [] };
  for (const row of rows) {
    const account = accountOf(row);
    if (account !== null) answer.shops.push({ read: row.name, account });
  }
  return answer;
}

// Whether the prompt has anything to ask.
export const needsShopPrompt = (view: ShopPromptView): boolean => view.shops.length > 0;
