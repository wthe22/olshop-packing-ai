// The Search screen's state against a faked back end: the boxes become filter text, the rows
// come back, a tick and *Select all results* choose rows, and **Reprint** writes them or shows why
// nothing was kept.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, useRunes } from "./tsModule.mjs";
import { searchSummary, texts } from "../src/texts.ts";

useRunes();
const { Search, itemsText } = await import("../src/lib/search.svelte.ts");

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
  path: `C:/Data/Toko Contoh/2026-10-07/output/${group} Sepatu x2.pdf`,
  trace: "from labels_1.pdf p. 3 · courier jnt by logo 39b9b7b9f61d0e7f · batch 1 · 1-A",
});

const found = { loading: false, orders: 2, batches: 1, rows: [row("580000000000000042"), row("580000000000000057")] };

test("the result line, a row's items and the fixed texts", () => {
  assert.equal(searchSummary(1, 1), "1 order in 1 batch");
  assert.equal(searchSummary(3, 2), "3 orders in 2 batches");
  assert.equal(itemsText(row("580000000000000042")), "Sepatu ×2, Spion Beat ×1");
  assert.deepEqual(
    [texts.search.reprint(1), texts.search.reprint(2), texts.search.made(1, 2, 2)],
    ["Reprint 1 order", "Reprint 2 orders", "Reprint 1 written: 2 orders, 2 pages."],
  );
  assert.equal(texts.search.noMatch, "No saved order matches.");
  assert.equal(texts.nav.search, "Search");
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
        : { made: { number: 1, file: "reprint 1 x2.pdf", path: "C:/r.pdf", orders: args.orders.length, pages: 2 }, errors: [] },
  });
  const search = new Search(() => "2026-10-07", () => (reprinted += 1));
  assert.equal(search.tree.root.children[0].field, "tracking_id", "the first box is the tracking ID");

  await search.run();
  assert.deepEqual(calls[1], {
    command: "search_orders",
    args: { filter: 'tracking_id starts_with "JY"', scope: "day", day: "2026-10-07" },
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
  assert.deepEqual(calls.at(-1).args.orders, [
    { day: "2026-10-07", order_id: "580000000000000042" },
    { day: "2026-10-07", order_id: "580000000000000057" },
  ]);
  assert.equal(search.made.orders, 2);
  assert.equal(reprinted, 1, "the Day screen and the panel recount");
  assert.deepEqual(search.ticked, [], "the ticks clear once written");

  refused = true;
  search.selectAll(true);
  await search.reprint();
  assert.equal(search.made, null);
  assert.deepEqual(search.findings, ["page 2 of the reprint is not the source page."]);
  assert.equal(reprinted, 1, "nothing was kept, nothing to recount");

  // A change of scope clears the rows, so they always match what is shown.
  search.setScope("all");
  assert.equal(search.result, null);
  await search.reprint();
  assert.equal(calls.filter((call) => call.command === "reprint").length, 2, "no rows, no reprint");
});

test("a filter that does not parse shows its error; an index being built is asked again", async () => {
  useBackend({
    tree_to_filter: () => {
      throw "line 1, col 13: a value is missing";
    },
  });
  const broken = new Search(() => "2026-10-07", () => {});
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
  const waiting = new Search(() => "2026-10-07", () => {});
  await waiting.run();
  assert.equal(waiting.loading, true);
  assert.equal(waiting.result, null);
  await new Promise((resolve) => setTimeout(resolve, 600));
  assert.equal(waiting.loading, false);
  assert.equal(waiting.result.orders, 2);
});
