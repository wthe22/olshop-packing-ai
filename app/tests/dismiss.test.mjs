// A menu closes on Escape and on a press outside it, and listens only while it is open.
//
// Run in `app/`: `npm test`.

import { test } from "node:test";
import assert from "node:assert/strict";

import "./tsModule.mjs";
const { dismiss } = await import("../src/lib/dismiss.ts");

// A document that keeps its listeners, so a test presses and types into it.
function fakeDocument() {
  const listeners = new Map();
  globalThis.document = {
    addEventListener: (type, listener) => listeners.set(type, listener),
    removeEventListener: (type, listener) => {
      if (listeners.get(type) === listener) listeners.delete(type);
    },
  };
  return listeners;
}

test("a press outside the menu or Escape closes it; a press inside or another key does not", () => {
  const listeners = fakeDocument();
  const inside = {};
  const menu = { contains: (target) => target === inside };
  let closed = 0;
  const stop = dismiss(() => (closed += 1))(menu);

  listeners.get("pointerdown")({ target: inside });
  listeners.get("keydown")({ key: "Enter" });
  assert.equal(closed, 0);
  listeners.get("pointerdown")({ target: {} });
  assert.equal(closed, 1);
  listeners.get("keydown")({ key: "Escape" });
  assert.equal(closed, 2);

  stop();
  assert.equal(listeners.size, 0, "closed, it listens no more");
});
