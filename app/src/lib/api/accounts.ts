// Typed wrappers around the Tauri commands for the Accounts screen. The Rust side sends
// snake_case fields; the interfaces here match it. The commands that change the accounts resolve to
// the accounts as written, or reject with the message to show under the field.

import { invoke } from "@tauri-apps/api/core";

export interface Account {
  name: string;
  shops: string[];
}

// One date with orders of an account.
export interface AccountDate {
  date: string;
  today: boolean;
  orders: number;
  batches: number;
}

export interface AccountDates {
  // Newest first.
  dates: AccountDate[];
  // The search index is still being built: no dates yet.
  loading: boolean;
}

// The accounts, in the order they were made.
export const accountsList = (): Promise<Account[]> => invoke("accounts_list");

export const accountDates = (account: string): Promise<AccountDates> =>
  invoke("account_dates", { account });

export const accountNew = (name: string): Promise<Account[]> => invoke("account_new", { name });

export const accountRename = (from: string, to: string): Promise<Account[]> =>
  invoke("account_rename", { from, to });

export const accountSetShops = (account: string, shops: string[]): Promise<Account[]> =>
  invoke("account_set_shops", { account, shops });

export const accountDelete = (name: string): Promise<Account[]> =>
  invoke("account_delete", { name });
