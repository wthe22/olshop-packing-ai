// The Accounts screen's texts, part of `texts` (`texts.accounts`).

import type { Question } from "./lib/confirm.svelte";
import type { ShopProblem } from "./lib/shops";
import { count } from "./count";

export const accountsTexts = {
  title: "Accounts",
  none: "No accounts yet. The first read of labels asks which account each shop belongs to.",
  newAccount: "New account",
  nameLabel: "Account name",
  namePlaceholder: "Account name, e.g. Toko Alfa",
  create: "Create",
  cancel: "Cancel",
  save: "Save",
  rename: "Rename",
  renameDone: "Rename",
  renameLabel: (name: string) => `New name for ${name}`,
  renameHint: "Orders saved before keep the name they were saved with.",
  delete: "Delete account",
  askDelete: (name: string): Question => ({
    title: `Delete account ${name}?`,
    body: "Its shops are asked about again at the next read. Its saved orders stay, under its name.",
    ok: "Delete account",
  }),
  askDiscard: (name: string): Question => ({
    title: `Discard the changes to ${name}'s shops?`,
    body: null,
    ok: "Discard",
  }),

  shops: "Shops",
  shopsHint: "The labels of these senders are this account's orders.",
  noShops: "No shops: only a label with no sender can be put here.",
  shopLabel: (n: number) => `Shop ${n}`,
  removeShop: (name: string) => (name ? `Take off ${name}` : "Take off this shop"),
  addShop: "+ Add shop",
  problem: (problem: ShopProblem) => {
    switch (problem.kind) {
      case "empty":
        return "A shop needs a name.";
      case "twice":
        return "This shop is listed twice.";
      case "other":
        return `${problem.account} already lists this shop.`;
    }
  },

  dates: "Dates",
  orders: "Orders",
  batches: "Batches",
  orderWord: (n: number) => (n === 1 ? "order" : "orders"),
  batchWord: (n: number) => (n === 1 ? "batch" : "batches"),
  noDates: "No orders saved under this name yet.",
  loading: "Loading",
  today: "today",
  unsaved: "Unsaved changes",
  monthOrders: (n: number) => count(n, "order", "orders"),
} as const;
