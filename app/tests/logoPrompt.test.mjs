// The logo prompt's flow and the words and counts it shows. No window is opened and nothing is
// written: the back end is a small fake that behaves like `answer_logo` (the answered logo leaves
// the list) and records what it was sent.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { LogoPrompt, needsLogoPrompt } from "../src/lib/logoPrompt.ts";
import { texts } from "../src/texts.ts";

// One read result with `count` unseen logos (`hash-1` …), 4 pages and 4 orders each, as the
// window receives it from `read_labels`.
function read(count) {
  return {
    files: 1,
    pages: 4,
    orders: 4,
    total_orders: 4,
    already_saved: 0,
    warnings: [],
    amend: null,
    couriers: ["J&T Express", "SiCepat", "J&T Cargo", "IDX", "SPX"],
    logos: Array.from({ length: count }, (_, index) => ({
      hash: `hash-${index + 1}`,
      pages: 4,
      orders: 4,
      picture: `data:image/png;base64,logo${index + 1}`,
    })),
    plan: { rows: [], has_duplicate_names: false, add_at: 0 },
  };
}

// A fake `api.answerLogo`: records every call and returns the result with the answered logo gone,
// exactly as the back end does (the answered logo is spent).
function backend(initial) {
  const calls = [];
  let result = initial;
  const answerLogo = async (hash, answer) => {
    calls.push({ hash, answer });
    result = { ...result, logos: result.logos.filter((logo) => logo.hash !== hash) };
    return result;
  };
  return { calls, answerLogo };
}

test("the prompt's words are the designed words, and its counts read singular where they should", () => {
  assert.equal(texts.logo.title, "Which courier is this logo?");
  assert.equal(texts.logo.notCourier, "Not a courier");
  assert.equal(texts.logo.dontKnow, "I don't know");
  assert.equal(texts.logo.skipAll, "Skip all");

  assert.equal(texts.logo.counts(4, 4), "4 pages · 4 orders");
  assert.equal(texts.logo.counts(1, 4), "1 page · 4 orders");
  assert.equal(texts.logo.counts(4, 1), "4 pages · 1 order");
  assert.equal(texts.logo.counts(1, 1), "1 page · 1 order");
});

test("a read with no unseen logo opens the Plan directly", () => {
  const empty = read(0);
  assert.equal(needsLogoPrompt(empty), false);
  assert.equal(needsLogoPrompt(read(2)), true);
  const prompt = new LogoPrompt(empty, async () => empty);
  assert.equal(prompt.current, null);
  assert.equal(prompt.needed, false);
});

test("an answer is sent as given, and the next unseen logo asks", async () => {
  // A courier answer names the order and moves on.
  const first = read(2);
  const { calls, answerLogo } = backend(first);
  const prompt = new LogoPrompt(first, answerLogo);
  assert.equal(prompt.current.hash, "hash-1");
  await prompt.answer({ kind: "courier", name: "SiCepat" });
  assert.deepEqual(calls, [
    { hash: "hash-1", answer: { kind: "courier", name: "SiCepat" } },
  ]);
  // The answered logo is spent; the next unseen logo asks.
  assert.equal(prompt.current.hash, "hash-2");
  await prompt.answer({ kind: "courier", name: "J&T Express" });
  assert.equal(calls.length, 2);
  assert.equal(prompt.current, null);
  assert.deepEqual(prompt.latest.logos, []);

  // *Not a courier* is sent as `not_courier` and asks the next logo.
  const notCourier = backend(read(2));
  const other = new LogoPrompt(read(2), notCourier.answerLogo);
  await other.answer({ kind: "not_courier" });
  assert.deepEqual(notCourier.calls[0], {
    hash: "hash-1",
    answer: { kind: "not_courier" },
  });
  assert.equal(other.current.hash, "hash-2");

  // *I don't know* sends `skip`, changes nothing, and asks the next logo.
  const unknown = read(2);
  unknown.warnings = ["Courier unknown for 4 orders: JY0000001234"];
  const skipping = backend(unknown);
  const third = new LogoPrompt(unknown, skipping.answerLogo);
  await third.answer({ kind: "skip" });
  assert.deepEqual(skipping.calls[0], { hash: "hash-1", answer: { kind: "skip" } });
  // Nothing was applied: the counts and warnings are the read's own.
  assert.deepEqual(third.latest.warnings, [
    "Courier unknown for 4 orders: JY0000001234",
  ]);
  assert.equal(third.latest.orders, 4);
  assert.equal(third.current.hash, "hash-2");

  // Esc is the same answer as *I don't know*, and skips only the one logo shown.
  const two = backend(read(3));
  const esc = new LogoPrompt(read(3), two.answerLogo);
  await esc.answer({ kind: "skip" });
  assert.equal(esc.current.hash, "hash-2");
  await esc.answer({ kind: "skip" });
  assert.equal(esc.current.hash, "hash-3");
  assert.equal(two.calls.length, 2);
  assert.ok(two.calls.every((call) => call.answer.kind === "skip"));
});

test("Skip all stops the prompts, sending nothing and keeping what was answered", async () => {
  // On the first logo nothing at all is sent and the draft is untouched.
  const first = read(3);
  const untouched = backend(first);
  const prompt = new LogoPrompt(first, untouched.answerLogo);
  prompt.skipAll();
  assert.equal(prompt.current, null);
  assert.equal(untouched.calls.length, 0, "nothing is sent for *Skip all*");
  assert.equal(prompt.latest, first, "the draft is untouched");

  // After one answer that answer stays, and the rest of the batch is not asked.
  const answered = backend(read(3));
  const second = new LogoPrompt(read(3), answered.answerLogo);
  await second.answer({ kind: "not_courier" });
  second.skipAll();
  assert.equal(answered.calls.length, 1, "the rest of the batch is not asked");
  assert.equal(second.current, null);
  // The rest stay in the list of the last result, unanswered.
  assert.deepEqual(
    second.latest.logos.map((logo) => logo.hash),
    ["hash-2", "hash-3"],
  );

  // A logo the back end leaves in the list is not asked twice.
  const one = read(1);
  const looping = [];
  const guarded = new LogoPrompt(one, async (hash) => {
    looping.push(hash);
    return one;
  });
  await guarded.answer({ kind: "skip" });
  assert.equal(looping.length, 1);
  assert.equal(guarded.current, null);
});
