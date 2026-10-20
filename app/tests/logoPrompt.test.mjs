// The logo prompt's flow and the counts it
// shows. No window is opened and nothing is written: the back end is a small fake that behaves
// like `answer_logo` (the answered logo leaves the list) and records what it was sent.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { LogoPrompt, needsLogoPrompt } from "../src/lib/logoPrompt.ts";
import { texts } from "../src/texts.ts";

/// One read result with `count` unseen logos (`hash-1` …), 4 pages and 4 orders each, as the
/// window receives it from `read_labels`.
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

/// A fake `api.answerLogo`: records every call and returns the result with the answered logo gone,
/// exactly as the back end does (the answered logo is spent).
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

test("the prompt's texts are the designed words", () => {
  assert.equal(texts.logo.title, "Which courier is this logo?");
  assert.equal(texts.logo.notCourier, "Not a courier");
  assert.equal(texts.logo.dontKnow, "I don't know");
  assert.equal(texts.logo.skipAll, "Skip all");
});

test("the counts line is '<n> pages · <m> orders', singular where it should be", () => {
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

test("a courier answer asks the next logo and applies the answer", async () => {
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
});

test("Not a courier is sent as not_courier and asks the next logo", async () => {
  const first = read(2);
  const { calls, answerLogo } = backend(first);
  const prompt = new LogoPrompt(first, answerLogo);

  await prompt.answer({ kind: "not_courier" });
  assert.deepEqual(calls[0], { hash: "hash-1", answer: { kind: "not_courier" } });
  assert.equal(prompt.current.hash, "hash-2");
});

test("I don't know sends skip, changes nothing, and asks the next logo", async () => {
  const first = read(2);
  first.warnings = ["Courier unknown for 4 orders: JY0000001234"];
  const { calls, answerLogo } = backend(first);
  const prompt = new LogoPrompt(first, answerLogo);

  await prompt.answer({ kind: "skip" });
  assert.deepEqual(calls[0], { hash: "hash-1", answer: { kind: "skip" } });
  // Nothing was applied: the counts and warnings are the read's own.
  assert.deepEqual(prompt.latest.warnings, ["Courier unknown for 4 orders: JY0000001234"]);
  assert.equal(prompt.latest.orders, 4);
  assert.equal(prompt.current.hash, "hash-2");
});

test("Esc skips only the one logo shown: the next one still asks", async () => {
  const first = read(3);
  const { calls, answerLogo } = backend(first);
  const prompt = new LogoPrompt(first, answerLogo);

  // Esc and *I don't know* are the same answer.
  await prompt.answer({ kind: "skip" });
  assert.equal(prompt.current.hash, "hash-2");
  await prompt.answer({ kind: "skip" });
  assert.equal(prompt.current.hash, "hash-3");
  assert.equal(calls.length, 2);
  assert.ok(calls.every((call) => call.answer.kind === "skip"));
});

test("Skip all on the first logo stops the prompts and sends nothing", async () => {
  const first = read(3);
  const { calls, answerLogo } = backend(first);
  const prompt = new LogoPrompt(first, answerLogo);

  prompt.skipAll();
  assert.equal(prompt.current, null);
  assert.equal(calls.length, 0, "nothing is sent for *Skip all*");
  assert.equal(prompt.latest, first, "the draft is untouched");
});

test("Skip all after one answer keeps that answer and stops the rest", async () => {
  const first = read(3);
  const { calls, answerLogo } = backend(first);
  const prompt = new LogoPrompt(first, answerLogo);

  await prompt.answer({ kind: "not_courier" });
  prompt.skipAll();
  assert.equal(calls.length, 1, "the rest of the batch is not asked");
  assert.equal(prompt.current, null);
  // The rest stay in the list of the last result, unanswered.
  assert.deepEqual(
    prompt.latest.logos.map((logo) => logo.hash),
    ["hash-2", "hash-3"],
  );
});

test("a logo left in the list by the back end is not asked twice", async () => {
  const first = read(1);
  const calls = [];
  const prompt = new LogoPrompt(first, async (hash) => {
    calls.push(hash);
    return first; // the logo stays: a guard against asking it again
  });

  await prompt.answer({ kind: "skip" });
  assert.equal(calls.length, 1);
  assert.equal(prompt.current, null);
});
