// The Day screen's Reprints card against a faked back end: a reprint's orders are asked for the
// first time it opens, asked again while the search index is being built, and kept when it closes.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend, useRunes } from "./tsModule.mjs";

useRunes();
const { ReprintList } = await import("../src/lib/reprintList.svelte.ts");

const order = { order_id: "580000000000000042", day: "2026-10-07", batch: 1, found: null };

test("a reprint's orders load when it opens, after the index is built, once", async () => {
  let asked = 0;
  const calls = useBackend({
    reprint_orders_of: () => {
      asked += 1;
      return asked === 1 ? { loading: true, orders: [] } : { loading: false, orders: [order] };
    },
  });
  const list = new ReprintList(() => "2026-10-08");
  await list.toggle(1);
  assert.equal(list.isOpen(1), true);
  assert.equal(list.orders[1], undefined, "the index is still being built");
  await new Promise((resolve) => setTimeout(resolve, 600));
  assert.deepEqual(list.orders[1], [order]);
  assert.deepEqual(calls[0], { command: "reprint_orders_of", args: { day: "2026-10-08", number: 1 } });

  await list.toggle(1);
  assert.equal(list.isOpen(1), false);
  await list.toggle(1);
  assert.equal(asked, 2, "opened again, the orders are not asked again");
});

test("a failed ask shows its error", async () => {
  useBackend({
    reprint_orders_of: () => {
      throw "2026-10-08 has no reprint 3.";
    },
  });
  const list = new ReprintList(() => "2026-10-08");
  await list.toggle(3);
  assert.equal(list.error, "2026-10-08 has no reprint 3.");
});
