// The shop prompt's rows and the answer they make: which account each unlisted shop goes to, and
// which account the labels of a file with no sender go to for this read.

import type { NoSenderRowView, ShopAnswer, ShopPromptView } from "./api/batch";

// The account chosen for a row: none yet, one already listed, or a new one by this name.
export type AccountChoice =
  | { kind: "none" }
  | { kind: "listed"; account: string }
  | { kind: "new"; name: string };

// A shop row as the owner edits it: the name as read, the name in the field, and its account.
export interface ShopDraft {
  read: string;
  name: string;
  choice: AccountChoice;
}

// A file row: the account its no-sender labels go to, `null` until chosen.
export interface NoSenderDraft {
  row: NoSenderRowView;
  account: string | null;
}

export function shopDrafts(view: ShopPromptView): ShopDraft[] {
  return view.shops.map((shop) => ({ read: shop.name, name: shop.name, choice: { kind: "none" } }));
}

// With one account and no shop to ask, a file's row starts on that account.
export function noSenderDrafts(view: ShopPromptView): NoSenderDraft[] {
  const only = view.shops.length === 0 && view.accounts.length === 1 ? view.accounts[0] : null;
  return view.no_sender.map((row) => ({ row, account: only }));
}

const chosen = (choice: AccountChoice) =>
  choice.kind === "listed" || (choice.kind === "new" && choice.name.trim() !== "");

// Whether every row has an account and every name is filled.
export function canContinue(shops: ShopDraft[], files: NoSenderDraft[]): boolean {
  return (
    shops.every((shop) => shop.name.trim() !== "" && chosen(shop.choice)) &&
    files.every((file) => file.account !== null)
  );
}

// The answer of the rows that have an account; `canContinue` is true before it is sent.
export function shopAnswer(shops: ShopDraft[], files: NoSenderDraft[]): ShopAnswer {
  const no_sender: Record<number, string> = {};
  for (const file of files) if (file.account !== null) no_sender[file.row.input] = file.account;
  return {
    shops: shops.flatMap(({ read, name, choice }): ShopAnswer["shops"] => {
      const answered = name.trim();
      if (choice.kind === "listed") return [{ read, name: answered, account: { listed: choice.account } }];
      if (choice.kind === "new") return [{ read, name: answered, account: { new: choice.name.trim() } }];
      return [];
    }),
    no_sender,
  };
}

// Whether the prompt has anything to ask.
export const needsShopPrompt = (view: ShopPromptView): boolean =>
  view.shops.length > 0 || view.no_sender.length > 0;
