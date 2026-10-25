#!/usr/bin/env node
// Size ratchet (code-rules.md, "File size"): a file over its ceiling is only allowed when the
// baseline already lists it and it has not grown. Run from anywhere; paths are taken from this
// file's own location, so the gate works from any folder.
//
// Ceilings: Rust production lines (the lines before the first `#[cfg(test)]`) and a Rust test file
// (a `.rs` file under a `tests/` folder, all its lines) 800; a `.svelte`, `.ts` or `.mjs` file under
// `app/src` or `app/tests` 500.
//
// Exit 0 when every file is at or under its ceiling or listed in the baseline without having grown.
// `--update` rewrites the baseline and may only lower a count or remove an entry; a new offender is
// never added, so the run fails instead.

import { readdirSync, readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const BASELINE_PATH = join(ROOT, "tools", "baselines", "size.json");

const RUST_CEILING = 800;
const UI_CEILING = 500;

// Folders that are never measured: build output, dependencies, downloaded binaries and
// confidential data.
const SKIP_DIRS = new Set([
  ".git",
  ".svelte-kit",
  "build",
  "docs",
  "node_modules",
  "samples",
  "target",
  "tmp",
  "vendor",
  "work",
]);

const UPDATE = process.argv.includes("--update");

/** Every measured source file under `dir`, as repository-relative POSIX paths.*/
function walk(dirRel, out = []) {
  for (const entry of readdirSync(join(ROOT, dirRel), { withFileTypes: true })) {
    const rel = dirRel === "" ? entry.name : `${dirRel}/${entry.name}`;
    if (entry.isDirectory()) {
      if (!SKIP_DIRS.has(entry.name)) walk(rel, out);
    } else {
      out.push(rel);
    }
  }
  return out;
}

/** Line count without the empty element a trailing newline produces.*/
function lineCount(text) {
  const lines = text.split(/\r?\n/);
  if (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();
  return lines.length;
}

/** The ceiling and the count for one file, or null when the file is not measured.*/
function measure(rel) {
  const underAppSource = rel.startsWith("app/src/") || rel.startsWith("app/tests/");
  if (/\.(svelte|ts|mjs)$/.test(rel)) {
    if (!underAppSource) return null;
    return { lines: lineCount(readFileSync(join(ROOT, rel), "utf8")), ceiling: UI_CEILING };
  }
  if (!rel.endsWith(".rs")) return null;
  const text = readFileSync(join(ROOT, rel), "utf8");
  const lines = text.split(/\r?\n/);
  if (/\/tests\//.test(rel) || rel.startsWith("tests/")) {
    return { lines: lineCount(text), ceiling: RUST_CEILING };
  }
  const testBlock = lines.findIndex((l) => l.trim().startsWith("#[cfg(test)]"));
  return { lines: testBlock === -1 ? lineCount(text) : testBlock, ceiling: RUST_CEILING };
}

function readBaseline() {
  if (!existsSync(BASELINE_PATH)) return {};
  const parsed = JSON.parse(readFileSync(BASELINE_PATH, "utf8"));
  return parsed.files ?? {};
}

function writeBaseline(files) {
  mkdirSync(dirname(BASELINE_PATH), { recursive: true });
  const sorted = Object.fromEntries(Object.entries(files).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)));
  const body = {
    note: "Files over their ceiling (Rust production 800, Rust test 800, Svelte/TypeScript 500), measured by tools/check-size.mjs. Only shrinks: a new entry is never added.",
    files: sorted,
  };
  writeFileSync(BASELINE_PATH, `${JSON.stringify(body, null, 2)}\n`);
}

function main() {
  const current = {};
  for (const rel of walk("")) {
    const m = measure(rel);
    if (m && m.lines > m.ceiling) current[rel] = m.lines;
  }
  const baseline = readBaseline();

  if (UPDATE) {
    const first = !existsSync(BASELINE_PATH);
    const forbidden = [];
    const next = {};
    for (const [rel, lines] of Object.entries(current)) {
      if (first) next[rel] = lines;
      else if (!(rel in baseline)) forbidden.push(`new offender ${rel} (${lines} lines) is never added`);
      else if (lines > baseline[rel]) forbidden.push(`${rel} grew ${baseline[rel]} → ${lines}, a baseline is never raised`);
      else next[rel] = lines;
    }
    if (forbidden.length > 0) {
      console.error("size baseline not written:");
      for (const f of forbidden) console.error(`  - ${f}`);
      process.exit(1);
    }
    const lowered = Object.entries(next).filter(([rel, lines]) => !first && lines < baseline[rel]);
    const removed = Object.keys(baseline).filter((rel) => !(rel in next));
    writeBaseline(next);
    if (first) console.log(`wrote the first size baseline with ${Object.keys(next).length} entries (today's offenders)`);
    for (const [rel, lines] of lowered) console.log(`lowered ${rel} ${baseline[rel]} → ${lines}`);
    for (const rel of removed) console.log(`removed ${rel} (no longer over its ceiling)`);
    if (!first) console.log(`size baseline written with ${Object.keys(next).length} entries`);
    return;
  }

  const newOffenders = Object.entries(current).filter(([rel]) => !(rel in baseline));
  const grown = Object.entries(current).filter(([rel, lines]) => rel in baseline && lines > baseline[rel]);
  const shrank = Object.entries(current).filter(([rel, lines]) => rel in baseline && lines < baseline[rel]);
  const gone = Object.keys(baseline).filter((rel) => !(rel in current));

  for (const [rel, lines] of Object.entries(current)) {
    const allowed = rel in baseline ? ` (baseline ${baseline[rel]})` : " (not in the baseline)";
    console.log(`over ceiling: ${rel} ${lines} lines${allowed}`);
  }
  for (const [rel, lines] of shrank) console.log(`lower the baseline: ${rel} ${baseline[rel]} → ${lines}`);
  for (const rel of gone) console.log(`remove from the baseline: ${rel} is no longer over its ceiling`);

  if (newOffenders.length > 0 || grown.length > 0) {
    console.error("size gate failed:");
    for (const [rel, lines] of newOffenders) console.error(`  ${rel} is over its ceiling (${lines} lines) and not in the baseline: split it`);
    for (const [rel, lines] of grown) console.error(`  ${rel} grew from ${baseline[rel]} to ${lines} lines`);
    process.exit(1);
  }
  console.log(`size gate passed: ${Object.keys(current).length} file(s) over a ceiling, all in the baseline`);
}

main();
