#!/usr/bin/env node
// Lint ratchet (code-rules.md, "Lint ratchet"): per file, counts the two ratcheted clippy warnings
// (`too_many_arguments`, `cognitive_complexity`, thresholds in the repository's `clippy.toml`) and
// every lint suppression (`#[allow]`/`#![allow]`, `svelte-ignore`, `@ts-ignore`/`@ts-expect-error`,
// `eslint-disable`). A file with a count not in the baseline, or above it, fails the gate.
//
// `tools/check.sh` runs the plain clippy gate with `-A clippy::too_many_arguments
// -A clippy::cognitive_complexity`, so those two warnings reach this script instead of `-D warnings`.
//
// Run from anywhere; paths are taken from this file's own location. `--update` rewrites the baseline
// and may only lower a count or remove an entry: a new suppression is never added.

import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, isAbsolute, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const BASELINE_PATH = join(ROOT, "tools", "baselines", "lint.json");

const UPDATE = process.argv.includes("--update");

// The clippy warnings the ratchet counts, and the source patterns that count as a suppression.
const CLIPPY_KINDS = {
  "clippy::too_many_arguments": "too_many_arguments",
  "clippy::cognitive_complexity": "cognitive_complexity",
};
const SUPPRESSION_PATTERNS = {
  allow: /#!?\[allow\b/g,
  svelte_ignore: /svelte-ignore/g,
  ts_ignore: /@ts-(?:ignore|expect-error)/g,
  eslint_disable: /eslint-disable/g,
};
const SUPPRESSION_EXTENSIONS = /\.(rs|svelte|ts|mts|mjs|js|cjs)$/;

// Folders never scanned: build output, dependencies, downloaded binaries and confidential data.
// `tools` holds the gate scripts themselves, whose own patterns would count as suppressions; the
// frozen Python reference `tools/analysis` is out of scope of the code rules.
const SKIP_DIRS = new Set([
  ".git",
  ".svelte-kit",
  ".venv",
  "build",
  "docs",
  "node_modules",
  "samples",
  "spike",
  "target",
  "tmp",
  "tools",
  "vendor",
  "work",
]);

function isSkippedDir(parentRel, name) {
  return SKIP_DIRS.has(name);
}

function walk(dirRel, out = []) {
  for (const entry of readdirSync(join(ROOT, dirRel), { withFileTypes: true })) {
    const rel = dirRel === "" ? entry.name : `${dirRel}/${entry.name}`;
    if (entry.isDirectory()) {
      if (!isSkippedDir(dirRel, entry.name)) walk(rel, out);
    } else {
      out.push(rel);
    }
  }
  return out;
}

/** The repository-relative POSIX path of a clippy span file name. */
function repoPath(fileName) {
  if (!isAbsolute(fileName)) return fileName.split("\\").join("/");
  return relative(ROOT, fileName).split("\\").join("/");
}

function bump(counts, file, kind) {
  const kinds = counts.get(file) ?? new Map();
  kinds.set(kind, (kinds.get(kind) ?? 0) + 1);
  counts.set(file, kinds);
}

function clippyCounts() {
  const run = spawnSync(
    "cargo",
    [
      "clippy",
      "--workspace",
      "--all-targets",
      "--message-format=json",
      "--",
      "-W",
      "clippy::too_many_arguments",
      "-W",
      "clippy::cognitive_complexity",
    ],
    { cwd: ROOT, encoding: "utf8", maxBuffer: 512 * 1024 * 1024 },
  );
  if (run.error) throw run.error;

  const counts = new Map();
  const seen = new Set();
  for (const line of run.stdout.split(/\r?\n/)) {
    if (!line.startsWith("{")) continue;
    let message;
    try {
      message = JSON.parse(line);
    } catch {
      continue;
    }
    if (message.reason !== "compiler-message") continue;
    const kind = CLIPPY_KINDS[message.message?.code?.code];
    if (!kind) continue;
    const spans = message.message.spans ?? [];
    const span = spans.find((s) => s.is_primary) ?? spans[0];
    if (!span) continue;
    // Clippy reports the same warning once per target that compiles the file (the crate's lib and
    // its test target), so a warning is counted once per place.
    const key = `${kind}|${span.file_name}|${span.line_start}|${span.column_start}`;
    if (seen.has(key)) continue;
    seen.add(key);
    bump(counts, repoPath(span.file_name), kind);
  }
  if (run.status !== 0 && counts.size === 0) {
    console.error(run.stderr.slice(-4000));
    throw new Error(`cargo clippy failed with exit code ${run.status}`);
  }
  return counts;
}

function suppressionCounts() {
  const counts = new Map();
  for (const rel of walk("")) {
    if (!SUPPRESSION_EXTENSIONS.test(rel)) continue;
    const text = readFileSync(join(ROOT, rel), "utf8");
    for (const [kind, pattern] of Object.entries(SUPPRESSION_PATTERNS)) {
      const found = text.match(pattern);
      if (found) for (let i = 0; i < found.length; i += 1) bump(counts, rel, kind);
    }
  }
  return counts;
}

function plain(counts) {
  const out = {};
  for (const file of [...counts.keys()].sort()) {
    out[file] = Object.fromEntries([...counts.get(file).entries()].sort(([a], [b]) => (a < b ? -1 : 1)));
  }
  return out;
}

function readBaseline() {
  if (!existsSync(BASELINE_PATH)) return {};
  const parsed = JSON.parse(readFileSync(BASELINE_PATH, "utf8"));
  return parsed.files ?? {};
}

function writeBaseline(files) {
  mkdirSync(dirname(BASELINE_PATH), { recursive: true });
  const body = {
    note: "Per file: the counts of the ratcheted clippy warnings (tools/check-lint.mjs) and of every lint suppression. Measured by tools/check-lint.mjs. Only shrinks: a new entry or suppression is never added.",
    files,
  };
  writeFileSync(BASELINE_PATH, `${JSON.stringify(body, null, 2)}\n`);
}

function main() {
  const current = { ...plain(clippyCounts()) };
  for (const [file, kinds] of Object.entries(plain(suppressionCounts()))) {
    current[file] = { ...(current[file] ?? {}), ...kinds };
  }
  const baseline = readBaseline();

  if (UPDATE) {
    const first = !existsSync(BASELINE_PATH);
    const forbidden = [];
    const next = {};
    for (const [file, kinds] of Object.entries(current)) {
      const lower = {};
      for (const [kind, count] of Object.entries(kinds)) {
        const before = baseline[file]?.[kind] ?? 0;
        if (first) {
          lower[kind] = count;
        } else if (count > before) {
          forbidden.push(
            before === 0 ? `${file}: new ${kind}` : `${file}: ${kind} grew ${before} → ${count}`,
          );
        } else {
          lower[kind] = count;
        }
      }
      if (Object.keys(lower).length > 0) next[file] = lower;
    }
    if (forbidden.length > 0) {
      console.error("lint baseline not written:");
      for (const f of forbidden) console.error(`  - ${f}`);
      process.exit(1);
    }
    const removed = Object.keys(baseline).filter((file) => !(file in next));
    writeBaseline(next);
    if (first) console.log(`wrote the first lint baseline with ${Object.keys(next).length} entries (today's counts)`);
    for (const file of removed) console.log(`removed ${file} (no count left)`);
    if (!first) console.log(`lint baseline written with ${Object.keys(next).length} entries`);
    return;
  }

  const findings = [];
  for (const [file, kinds] of Object.entries(current)) {
    for (const [kind, count] of Object.entries(kinds)) {
      const before = baseline[file]?.[kind] ?? 0;
      if (count > before) {
        findings.push(
          before === 0
            ? `${file}: new ${kind} (${count}), a new suppression is never added`
            : `${file}: ${kind} grew ${before} → ${count}`,
        );
      } else if (count < before) {
        console.log(`lower the baseline: ${file} ${kind} ${before} → ${count}`);
      }
    }
  }
  const gone = [];
  for (const [file, kinds] of Object.entries(baseline)) {
    for (const [kind, count] of Object.entries(kinds)) {
      if ((current[file]?.[kind] ?? 0) === 0) gone.push(`${file}: ${kind} is gone (${count})`);
    }
  }
  for (const g of gone) console.log(`remove from the baseline: ${g}`);

  const entries = Object.entries(current);
  for (const [file, kinds] of entries) {
    console.log(`${file} ${JSON.stringify(kinds)}`);
  }
  if (findings.length > 0) {
    console.error("lint gate failed:");
    for (const f of findings) console.error(`  ${f}`);
    process.exit(1);
  }
  console.log(`lint gate passed: ${entries.length} file(s) with a counted warning or suppression, all in the baseline`);
}

main();
