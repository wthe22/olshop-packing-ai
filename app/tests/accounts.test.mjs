// The Accounts screen against a faked back end: the shop checks, the screen's commands (new,
// rename, delete, save, the discard question), and the pure choices that follow from accounts
// elsewhere (the shop prompt's answer, the Day summary menu, the Account box's value). No window
// opens and nothing is written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, useRunes } from "./tsModule.mjs";
import { answerQuestions } from "./answers.mjs";
import { texts } from "../src/texts.ts";

useRunes();
const { AccountsState } = await import("../src/lib/accountsState.svelte.ts");
const { listedElsewhere, shopProblems, shopsChanged } = await import("../src/lib/shops.ts");
const { accountRows, canContinue, choiceOf, listOf, needsShopPrompt, shopAnswer, valueOf } = await import(
  "../src/lib/shopPrompt.ts"
);
const { accountsOf } = await import("../src/lib/dayAccounts.ts");
const { accountValue } = await import("../src/lib/conditions.ts");

const alfa = { name: "Toko Alfa", shops: ["Toko Alfa", "Toko Alfa Official"] };
const beta = { name: "Toko Beta", shops: ["Toko Beta"] };

test("a shop is checked against the list and the other accounts, spaces and case ignored", () => {
  const others = listedElsewhere([alfa, beta], "Toko Alfa");
  assert.deepEqual(others, [{ shop: "Toko Beta", account: "Toko Beta" }]);
  assert.deepEqual(
    shopProblems(["Toko Alfa", " ", "toko  ALFA", "tokobeta", "Toko Gamma"], others),
    [null, { kind: "empty" }, { kind: "twice" }, { kind: "other", account: "Toko Beta" }, null],
  );
  assert.equal(listedElsewhere([alfa], null).length, 2, "no account shown: every shop is elsewhere");
  assert.equal(shopsChanged(alfa.shops, [" Toko Alfa ", "Toko Alfa Official"]), false);
  assert.equal(shopsChanged(alfa.shops, ["Toko Alfa"]), true);
  assert.equal(shopsChanged([], [""]), true, "an added empty field is a change");
  assert.equal(texts.accounts.problem({ kind: "other", account: "Toko Beta" }), "Toko Beta already lists this shop.");
  assert.equal(texts.accounts.problem({ kind: "twice" }), "This shop is listed twice.");
  assert.equal(texts.accounts.problem({ kind: "empty" }), "A shop needs a name.");
});

test("the screen opens on the first account and keeps edits until the owner discards them", async () => {
  let yes = false;
  const asked = answerQuestions(() => yes);
  const calls = useBackend({
    accounts_list: () => [alfa, beta],
    account_dates: ({ account }) => ({ dates: [{ date: "2026-10-07", today: true, orders: 5, batches: 1 }], loading: false, account }),
  });
  const screen = new AccountsState();
  await screen.load();
  assert.equal(screen.current, "Toko Alfa");
  assert.deepEqual(screen.names, alfa.shops);
  assert.equal(screen.dates.dates[0].orders, 5);
  assert.equal(screen.dirty, false);

  screen.addShop();
  screen.names[2] = "Toko Beta";
  assert.equal(screen.dirty, true);
  assert.equal(screen.blocked, texts.accounts.problem({ kind: "other", account: "Toko Beta" }), "Save is off with the reason");

  await screen.select("Toko Beta");
  assert.deepEqual(asked, [texts.accounts.askDiscard("Toko Alfa")]);
  assert.equal(screen.current, "Toko Alfa", "a refused discard stays");
  assert.equal(await screen.startNew(), false);

  yes = true;
  await screen.select("Toko Beta");
  assert.deepEqual([screen.current, screen.names, screen.dirty], ["Toko Beta", beta.shops, false]);
  assert.equal(calls.filter((call) => call.command === "account_dates").length, 2);
  assert.equal(await screen.startNew(), true, "nothing unsaved: no question");
  screen.stop();
});

test("new, rename, save and delete use the accounts as written, and a refusal is told", async () => {
  let accounts = [alfa];
  const calls = useBackend({
    accounts_list: () => accounts,
    account_dates: () => ({ dates: [], loading: false }),
    account_new: ({ name }) => {
      if (name === "Toko Alfa") throw "An account with this name is already there.";
      accounts = [...accounts, { name, shops: [] }];
      return accounts;
    },
    account_rename: ({ from, to }) => {
      accounts = accounts.map((account) => (account.name === from ? { ...account, name: to } : account));
      return accounts;
    },
    account_set_shops: ({ account, shops }) => {
      if (shops.includes("Toko Rusak")) throw "Toko Rusak: not a shop name.";
      accounts = accounts.map((one) => (one.name === account ? { ...one, shops } : one));
      return accounts;
    },
    account_delete: ({ name }) => {
      accounts = accounts.filter((account) => account.name !== name);
      return accounts;
    },
  });
  answerQuestions(() => true);
  const screen = new AccountsState();
  await screen.load();

  assert.equal(await screen.create("Toko Alfa"), "An account with this name is already there.");
  assert.equal(await screen.create("Toko Beta"), null);
  assert.deepEqual([screen.current, screen.names], ["Toko Beta", []], "the new account is shown, with no shop");

  screen.addShop();
  screen.names[0] = " Toko Beta ";
  await screen.saveShops();
  assert.deepEqual(calls.find((call) => call.command === "account_set_shops").args, { account: "Toko Beta", shops: ["Toko Beta"] });
  assert.equal(screen.dirty, false);

  screen.addShop();
  screen.names[1] = "Toko Rusak";
  await screen.saveShops();
  assert.equal(screen.refused, "Toko Rusak: not a shop name.");
  screen.cancelShops();
  assert.deepEqual([screen.names, screen.refused], [["Toko Beta"], ""]);

  assert.equal(await screen.rename("Toko Gamma"), null);
  assert.deepEqual([screen.current, screen.names], ["Toko Gamma", ["Toko Beta"]]);

  await screen.remove();
  assert.deepEqual([screen.current, screen.accounts.map((account) => account.name)], ["Toko Alfa", ["Toko Alfa"]]);
  await screen.remove();
  assert.deepEqual([screen.current, screen.account], [null, null], "no account left");
  screen.stop();
});

test("the dates are asked for again while the index is built", async () => {
  let loading = true;
  const calls = useBackend({
    accounts_list: () => [alfa],
    account_dates: () => ({ dates: [], loading }),
  });
  const screen = new AccountsState();
  await screen.load();
  assert.equal(screen.dates.loading, true);
  loading = false;
  await new Promise((resolve) => setTimeout(resolve, 650));
  assert.equal(screen.dates.loading, false);
  assert.equal(calls.filter((call) => call.command === "account_dates").length, 2);
  screen.stop();
});

test("the account question needs an account on every row and answers by row", () => {
  const view = {
    accounts: ["Toko Alfa"],
    shops: [{ name: "Toko Gamma", orders: 12 }],
    no_sender: [
      { input: 3, file: "labels_3.pdf", orders: 2, others: { shop: "Toko Gamma" } },
      { input: 4, file: "labels_4.pdf", orders: 1, others: { account: "Toko Alfa" } },
      { input: 5, file: "labels_5.pdf", orders: 1, others: null },
    ],
  };
  assert.equal(needsShopPrompt(view), true);
  assert.equal(needsShopPrompt({ accounts: [], shops: [], no_sender: [] }), false);

  const rows = accountRows(view);
  const [shop, withShop, withAlfa, alone] = rows;
  assert.deepEqual(shop.choice, { kind: "none" }, "with accounts a shop starts unanswered");
  assert.equal(valueOf(rows, withShop), "", "a file follows its shop, which has no account yet");
  assert.equal(valueOf(rows, withAlfa), "listed:Toko Alfa", "a file starts on its other orders' account");
  assert.equal(valueOf(rows, alone), "");
  assert.equal(canContinue(rows), false);

  shop.choice = choiceOf(shop, "new");
  assert.deepEqual(shop.choice, { kind: "new", name: "Toko Gamma" }, "a new account starts on the shop name");
  assert.equal(valueOf(rows, withShop), "as:shop:Toko Gamma", "the file follows to the new account");
  assert.deepEqual(
    listOf(rows, alone, view.accounts).map((entry) => entry.text),
    ["Toko Alfa", "New: Toko Gamma", "New account"],
    "another row's new account is offered",
  );
  alone.choice = choiceOf(alone, "as:shop:Toko Gamma");
  assert.equal(canContinue(rows), true);
  assert.deepEqual(shopAnswer(rows), {
    shops: [{ read: "Toko Gamma", account: { new: "Toko Gamma" } }],
    no_sender: { 3: { listed: "Toko Gamma" }, 4: { listed: "Toko Alfa" }, 5: { listed: "Toko Gamma" } },
  });

  shop.choice = { kind: "new", name: "  " };
  assert.equal(canContinue(rows), false, "a new account needs a name");
  alone.choice = choiceOf(alone, "new");
  assert.deepEqual(alone.choice, { kind: "new", name: "" }, "a file has no name to start on");
  alone.choice.name = " Toko Baru ";
  shop.choice = choiceOf(shop, "listed:Toko Alfa");
  assert.deepEqual(shopAnswer(rows), {
    shops: [{ read: "Toko Gamma", account: { listed: "Toko Alfa" } }],
    no_sender: { 3: { listed: "Toko Alfa" }, 4: { listed: "Toko Alfa" }, 5: { new: "Toko Baru" } },
  });

  const fresh = accountRows({ ...view, accounts: [], no_sender: [] });
  assert.deepEqual(fresh[0].choice, { kind: "new", name: "Toko Gamma" }, "no account yet: a new one");
  const only = accountRows({ ...view, shops: [], no_sender: [{ ...view.no_sender[2] }] });
  assert.equal(valueOf(only, only[0]), "listed:Toko Alfa", "one account and no shop to ask: the file starts on it");
  const two = { ...view, accounts: ["Toko Alfa", "Toko Beta"], shops: [], no_sender: [{ ...view.no_sender[2] }] };
  assert.equal(valueOf(accountRows(two), accountRows(two)[0]), "");
});

test("the accounts of a date are counted over its batches, most orders first", () => {
  const batch = (...accounts) => ({ accounts: accounts.map(([account, orders]) => ({ account, orders })) });
  const one = [batch(["Toko Alfa", 5]), batch(["Toko Alfa", 3])];
  const several = [batch(["Toko Alfa", 5], ["Toko Beta", 4]), batch(["Toko Beta", 3])];
  assert.deepEqual(accountsOf(one), ["Toko Alfa"]);
  assert.equal(accountsOf(one).length > 1, false, "one account: a plain button");
  assert.deepEqual(accountsOf(several), ["Toko Beta", "Toko Alfa"], "most orders first");
  assert.equal(accountsOf(several).length > 1, true, "several accounts: a menu");
  assert.deepEqual(accountsOf([]), []);
  assert.equal(
    texts.day.batchAccounts(several[0].accounts.length) + " | " + texts.day.batchAccounts(one[0].accounts.length),
    "2 accounts | 1 account",
  );
});

test("Account equals picks a listed account, other operators keep the typed text", () => {
  const accounts = ["Toko Alfa", "Toko Beta"];
  assert.equal(accountValue("beta", "equals", accounts), "Toko Beta");
  assert.equal(accountValue("zzz", "equals", accounts), "Toko Alfa", "no match: the first account");
  assert.equal(accountValue("Toko Beta", "equals", accounts), "Toko Beta");
  assert.equal(accountValue("Toko Beta", "contains", accounts), "Toko Beta");
  assert.equal(accountValue("beta", "equals", []), "beta", "no list: typed text");
});
