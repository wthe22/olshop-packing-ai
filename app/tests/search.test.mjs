// The Search screen's state against a faked back end: the boxes become filter text, the rows
// come back, a tick and *Select all results* choose rows, and **Reprint** writes them or shows why
// nothing was kept.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, useRunes } from "./tsModule.mjs";
import { searchSummary, texts } from "../src/texts.ts";
import { answerQuestions } from "./answers.mjs";

useRunes();
const { Search, detailLines, itemsText, reprintedMark, totalLine } = await import("../src/lib/search.svelte.ts");

const row = (order_id, group = "1-A") => ({
  day: "2026-10-07",
  order_id,
  tracking_id: "JY0000001234",
  courier: "jnt",
  items: [
    { name: "Sepatu", quantity: 2 },
    { name: "Spion Beat", quantity: 1 },
  ],
  batch: 1,
  group,
  file: `${group} Sepatu x2.pdf`,
  trace: [
    { text: "from labels_1.pdf p. 3 · courier jnt by logo ", code: false },
    { text: "39b9b7b9f61d0e7f", code: true },
    { text: " · batch 1 · 1-A", code: false },
  ],
  reprints: [],
});

const found = { loading: false, orders: 2, batches: 1, rows: [row("580000000000000042"), row("580000000000000057")] };

test("the result line, a row's items and the fixed texts", () => {
  assert.equal(searchSummary(1, 1), "1 order in 1 batch");
  assert.equal(searchSummary(3, 2), "3 orders in 2 batches");
  assert.equal(itemsText(row("580000000000000042")), "Sepatu ×2, Spion Beat ×1");
  assert.deepEqual(
    [texts.search.reprint, texts.search.made(1, 2, 2)],
    ["Reprint", "Reprint 1 written: 2 orders, 2 pages."],
  );
  assert.equal(texts.search.noMatch, "No saved order matches.");
  assert.equal(texts.nav.search, "Search");
  assert.deepEqual(
    [texts.search.selected(2, 3), texts.search.reprintedBefore(1), texts.search.reprintedBefore(2)],
    ["2 of 3 selected", "1 of them was reprinted before", "2 of them were reprinted before"],
  );
  assert.deepEqual(
    [texts.search.askAgain(1, 3), texts.search.askAgain(2, 3)],
    [
      { title: "Reprint 3 orders?", body: "1 of them was reprinted before.", ok: "Reprint" },
      { title: "Reprint 3 orders?", body: "2 of them were reprinted before.", ok: "Reprint" },
    ],
  );
  assert.equal(reprintedMark(row("580000000000000042")), null);
  const twice = { ...row("580000000000000042"), reprints: [{ day: "2026-10-07", number: 1 }, { day: "2026-10-08", number: 2 }] };
  assert.deepEqual(reprintedMark(twice), {
    text: "Reprinted 2×",
    title: "Reprint 1 on Wed, 7 Oct\nReprint 2 on Thu, 8 Oct",
  });
});

test("a search finds rows, ticks choose them and Reprint writes them into today", async () => {
  let reprinted = 0;
  let refused = false;
  const calls = useBackend({
    tree_to_filter: () => 'tracking_id starts_with "JY"',
    search_orders: () => found,
    reprint: (args) =>
      refused
        ? { made: null, errors: ["page 2 of the reprint is not the source page."] }
        : { made: { number: 1, file: "Reprint 1 x2.pdf", path: "C:/r.pdf", orders: args.orders.length, pages: 2 }, errors: [] },
  });
  const search = new Search(() => "2026-10-07", () => "2026-10-27", () => (reprinted += 1));
  const [first] = search.tree.root.children;
  assert.deepEqual([first.field, first.operator], ["order_id", "contains"], "the first box: Order ID contains");

  await search.run();
  assert.deepEqual(calls[1], {
    command: "search_orders",
    args: { filter: 'tracking_id starts_with "JY"', range: { from: "2026-10-07", to: "2026-10-07" } },
  });
  assert.equal(search.result.orders, 2);

  search.toggle(search.result.rows[1]);
  assert.equal(search.isTicked(search.result.rows[1]), true);
  assert.equal(search.allTicked(), false);
  search.selectAll(true);
  assert.equal(search.allTicked(), true);
  search.toggleTrace(search.result.rows[0]);
  assert.equal(search.isTraced(search.result.rows[0]), true);

  await search.reprint();
  assert.deepEqual(calls.findLast((call) => call.command === "reprint").args.orders, [
    { day: "2026-10-07", order_id: "580000000000000042" },
    { day: "2026-10-07", order_id: "580000000000000057" },
  ]);
  assert.equal(search.made.orders, 2);
  assert.equal(reprinted, 1, "the Day screen and the panel recount");
  assert.deepEqual(search.ticked, [], "the ticks clear once written");
  assert.equal(calls.at(-1).command, "search_orders", "the rows are searched again for their marks");
  assert.notEqual(search.made, null, "the written line stays");

  refused = true;
  search.selectAll(true);
  await search.reprint();
  assert.equal(search.made, null);
  assert.deepEqual(search.findings, ["page 2 of the reprint is not the source page."]);
  assert.equal(reprinted, 1, "nothing was kept, nothing to recount");

  // A change of dates clears the rows, so they always match what is shown.
  search.setPreset("week");
  assert.equal(search.result, null);
  assert.deepEqual(search.range, { from: "2026-10-21", to: "2026-10-27" }, "seven dates ending today");
  await search.reprint();
  assert.equal(calls.filter((call) => call.command === "reprint").length, 2, "no rows, no reprint");

  // *All days* sends no range; *Custom range* finds the dates shown before again.
  search.setPreset("all");
  await search.run();
  assert.equal(calls.findLast((call) => call.command === "search_orders").args.range, null);
  search.setPreset("custom");
  assert.deepEqual(search.range, { from: "2026-10-21", to: "2026-10-27" });
  // A date picked by hand makes the range the owner's own.
  search.setPreset("yesterday");
  search.setRange({ ...search.range, from: "2026-10-01" });
  assert.equal(search.preset, "custom");
  await search.run();
  assert.deepEqual(calls.findLast((call) => call.command === "search_orders").args.range, {
    from: "2026-10-01",
    to: "2026-10-26",
  });
});

test("a filter that does not parse shows its error; an index being built is asked again", async () => {
  useBackend({
    tree_to_filter: () => {
      throw "line 1, col 13: a value is missing";
    },
  });
  const broken = new Search(() => "2026-10-07", () => "2026-10-27", () => {});
  await broken.run();
  assert.equal(broken.error, "line 1, col 13: a value is missing");
  assert.equal(broken.busy, false);

  let asked = 0;
  useBackend({
    tree_to_filter: () => 'name contains "sepatu"',
    search_orders: () => {
      asked += 1;
      return asked === 1 ? { loading: true, orders: 0, batches: 0, rows: [] } : found;
    },
  });
  const waiting = new Search(() => "2026-10-07", () => "2026-10-27", () => {});
  await waiting.run();
  assert.equal(waiting.loading, true);
  assert.equal(waiting.result, null);
  await new Promise((resolve) => setTimeout(resolve, 600));
  assert.equal(waiting.loading, false);
  assert.equal(waiting.result.orders, 2);
});

test("one order found is ticked; an order reprinted before is printed again only when the owner says so", async () => {
  let answer = false;
  const asked = answerQuestions(() => answer);
  const before = { ...row("580000000000000042"), reprints: [{ day: "2026-10-07", number: 1 }] };
  const calls = useBackend({
    tree_to_filter: () => 'tracking_id starts_with "JY0000001234"',
    search_orders: () => ({ loading: false, orders: 1, batches: 1, rows: [before] }),
    reprint: () => ({ made: { number: 2, file: "Reprint 2 x1.pdf", path: "C:/r.pdf", orders: 1, pages: 1 }, errors: [] }),
  });
  const search = new Search(() => "2026-10-07", () => "2026-10-27", () => {});
  await search.run();
  assert.equal(search.isTicked(before), true, "a scanned tracking ID is one click from its reprint");
  assert.equal(search.reprintedBefore(), 1);
  assert.equal(search.someTicked(), false, "all of one is not some");

  await search.reprint();
  assert.deepEqual(asked, [
    { title: "Reprint 1 order?", body: "It was reprinted before.", ok: "Reprint" },
  ]);
  assert.equal(calls.some((call) => call.command === "reprint"), false, "Cancel prints nothing");
  assert.equal(search.isTicked(before), true, "the tick stays for another try");

  answer = true;
  await search.reprint();
  assert.equal(calls.filter((call) => call.command === "reprint").length, 1);
  assert.equal(search.made.number, 2);

  search.selectAll(true);
  search.clear();
  assert.deepEqual(search.ticked, []);
});

test("a click on a card opens its details and ticks nothing; a drag or a double click selects its text", async () => {
  useBackend({ tree_to_filter: () => "", search_orders: () => found });
  const search = new Search(() => "2026-10-07", () => "2026-10-07", () => {});
  await search.run();
  const [first, second] = search.result.rows;
  search.clickCard(first, 1, false);
  assert.equal(search.isTraced(first), true, "a click opens the details");
  assert.equal(search.isTicked(first), false, "only the tick ticks");
  search.clickCard(first, 1, true);
  assert.equal(search.isTraced(first), true, "a drag that selected text leaves them open");
  search.clickCard(second, 1, false);
  search.clickCard(second, 2, true);
  assert.equal(search.isTraced(second), false, "a double click undoes its first click");
  search.clickCard(second, 3, true);
  assert.equal(search.isTraced(second), false, "a triple click changes nothing");
  search.clickCard(first, 2, true);
  assert.equal(search.isTraced(first), true, "a second click on another card's double click undoes nothing");
  search.clickCard(first, 1, false);
  assert.equal(search.isTraced(first), false, "a click closes them again");
});

test("the Courier box lists the couriers of the rules, and a card names its page", async () => {
  const couriers = [{ code: "jnt", name: "J&T Express" }];
  useBackend({ rules_list: () => ({ rules: ["default"], text: "", couriers, draft_rule: null }) });
  const search = new Search(() => "2026-10-07", () => "2026-10-07", () => {});
  await search.loadCouriers();
  assert.deepEqual(search.couriers, couriers);
  assert.deepEqual(
    [texts.search.pages(14, 14), texts.search.pages(14, 15), texts.search.noCourier],
    ["page 14", "pages 14–15", "Unknown"],
  );
  assert.equal(texts.search.hash("e731bbe96035"), "SHA256: e731bbe96035");

  useBackend({});
  await search.loadCouriers();
  assert.match(search.error, /rules_list/, "a failed load is shown");
});

test("a card shows the Qty Total only when it says more; its details say what the head does not", () => {
  const order = {
    ...row("580000000000000042"),
    account: "Toko Alfa",
    shop: "Toko Alfa Cabang",
    items: [{ name: "Spion Beat", variant: "honda", quantity: 2, sku: "SKU-01" }],
    ship_by: null,
    total_quantity: 2,
    area: { province: "JAWA BARAT", city: "KOTA CONTOH", district: null },
    weight: "not_printed",
  };
  assert.equal(itemsText(order), "Spion Beat — honda ×2");

  assert.equal(totalLine(order), null, "one line of the whole total says it already");
  assert.equal(totalLine({ ...order, total_quantity: 3 }), 3, "a total its line does not add up to");
  assert.equal(totalLine({ ...row("580000000000000042"), total_quantity: null }), null, "no Qty Total printed");
  assert.equal(totalLine({ ...row("580000000000000042"), total_quantity: 3 }), 3, "several lines");

  const lines = (row) => detailLines(row).map((line) => [line.head, line.text, line.notRead]);
  assert.deepEqual(lines({ ...order, ship_by: "2026-10-08", weight: { grams: 1250 } }), [
    ["Address", "JAWA BARAT, KOTA CONTOH", false],
    ["Ship by", "Thu, 8 Oct 2026", false],
    ["Weight", "1,250 g", false],
    ["Account", "Toko Alfa", false],
  ]);
  const instant = { ...order, shop: "Toko Alfa", tracking_id: "12AB" };
  assert.deepEqual(
    lines(instant),
    [["Address", "JAWA BARAT, KOTA CONTOH", false]],
    "what the label does not print is left out, and the account the shop is named after is not said again",
  );
  assert.deepEqual(lines({ ...order, shop: null, area: null, weight: null }), [
    ["Address", "not read yet", true],
    ["Weight", "not read yet", true],
  ]);
});

test("Search opens again with the boxes it was left with, the dates from the viewed day; Reset starts over", () => {
  useBackend({});
  const first = new Search(() => "2026-10-07", () => "2026-10-27", () => {});
  first.reset();
  first.tree.root.children[0].value = "JY0000001234";
  first.setPreset("all");

  const again = new Search(() => "2026-10-08", () => "2026-10-27", () => {});
  assert.equal(again.tree.root.children[0].value, "JY0000001234", "the boxes are kept");
  assert.equal(again.preset, again.presets()[0], "the dates are not");
  assert.deepEqual(again.range, { from: "2026-10-08", to: "2026-10-08" });

  again.setPreset("all");
  again.reset();
  const [box] = again.tree.root.children;
  assert.deepEqual([again.tree.root.children.length, box.field, box.operator, box.value], [1, "order_id", "contains", ""]);
  assert.equal(again.preset, again.presets()[0]);
  assert.equal(again.result, null);
  const third = new Search(() => "2026-10-08", () => "2026-10-27", () => {});
  assert.equal(third.tree.root.children[0].value, "", "what Reset left is kept");
});
