#!/usr/bin/env node
// Coverage compare (code-rules.md, "Coverage is the measure"): records and compares the coverage
// baseline per language, tools/baselines/coverage.json.
//
// Rust: `cargo llvm-cov --workspace --json --summary-only`, lines/regions/functions per crate and in
// total. The instrumented build is large: the script honours CARGO_LLVM_COV_TARGET_DIR when it is set
// (setup.md says to point it at a drive with room).
// TypeScript: `node --test --experimental-test-coverage` from `app/`. Node counts only the files a
// test loads, so the run uses `--test-coverage-include-all` over `app/src` to count a `.ts` file no
// test loads as uncovered; the `.svelte` files are listed separately, as no test renders them.
//
// Compare mode (default) fails when a compared number dropped by more than 0.1 point. The compared
// numbers are the Rust totals and per-crate figures and the TypeScript totals; the per-file
// TypeScript detail is recorded for reading only, since splitting a file changes it without
// changing the coverage. `--record` writes the baseline and keeps an existing entry when the new
// number is not higher.

import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const BASELINE_PATH = join(ROOT, "tools", "baselines", "coverage.json");
const TOLERANCE = 0.1;

// Crates holding test fixtures rather than product code: left out of the report and the totals.
const IGNORED_CRATES = new Set(["crates/testkit"]);

const RECORD = process.argv.includes("--record");

/** A percentage with two decimals, from a covered/count pair.*/
function percent(covered, count) {
  if (count === 0) return 0;
  return Math.round((covered / count) * 10000) / 100;
}

function repoPath(fileName) {
  const posix = fileName.split("\\").join("/");
  if (!isAbsolute(fileName)) return posix;
  return relative(ROOT, fileName).split("\\").join("/");
}

/** `crates/engine/src/rules.rs` → `crates/engine`; `app/src-tauri/src/lib.rs` → `app/src-tauri`.*/
function crateOf(repoPathName) {
  const parts = repoPathName.split("/");
  if (parts[0] === "crates") return `crates/${parts[1]}`;
  if (parts[0] === "app" && parts[1] === "src-tauri") return "app/src-tauri";
  return parts.slice(0, 2).join("/");
}

function rustCoverage() {
  const scratch = mkdtempSync(join(tmpdir(), "packing-cov-"));
  const report = join(scratch, "rust.json");
  try {
    const run = spawnSync(
      "cargo",
      ["llvm-cov", "--workspace", "--json", "--summary-only", "--output-path", report],
      { cwd: ROOT, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 },
    );
    if (run.error) throw run.error;
    if (run.status !== 0) {
      console.error(run.stdout?.slice(-4000) ?? "");
      console.error(run.stderr?.slice(-4000) ?? "");
      throw new Error(`cargo llvm-cov failed with exit code ${run.status}`);
    }
    const data = JSON.parse(readFileSync(report, "utf8")).data[0];

    const crates = {};
    for (const file of data.files) {
      const crate = crateOf(repoPath(file.filename));
      if (IGNORED_CRATES.has(crate)) continue;
      const into = (crates[crate] ??= { lines: { c: 0, n: 0 }, regions: { c: 0, n: 0 }, functions: { c: 0, n: 0 } });
      for (const key of ["lines", "regions", "functions"]) {
        into[key].c += file.summary[key].covered;
        into[key].n += file.summary[key].count;
      }
    }
    const cratesOut = {};
    const totals = { lines: { c: 0, n: 0 }, regions: { c: 0, n: 0 }, functions: { c: 0, n: 0 } };
    for (const crate of Object.keys(crates).sort()) {
      const v = crates[crate];
      for (const key of ["lines", "regions", "functions"]) {
        totals[key].c += v[key].c;
        totals[key].n += v[key].n;
      }
      cratesOut[crate] = {
        lines: percent(v.lines.c, v.lines.n),
        regions: percent(v.regions.c, v.regions.n),
        functions: percent(v.functions.c, v.functions.n),
      };
    }
    return {
      totals: {
        lines: percent(totals.lines.c, totals.lines.n),
        regions: percent(totals.regions.c, totals.regions.n),
        functions: percent(totals.functions.c, totals.functions.n),
      },
      crates: cratesOut,
    };
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

/** The `app/src` source files a rendered test never loads: the `.svelte` components.*/
function svelteFiles(dirRel = "app/src", out = []) {
  for (const entry of readdirSync(join(ROOT, dirRel), { withFileTypes: true })) {
    const rel = `${dirRel}/${entry.name}`;
    if (entry.isDirectory()) svelteFiles(rel, out);
    else if (entry.name.endsWith(".svelte")) out.push(rel);
  }
  return out.sort();
}

function typescriptCoverage() {
  const run = spawnSync(
    process.execPath,
    [
      "--import=./tests/tsModule.mjs",
      "--test",
      "--experimental-test-coverage",
      "--test-coverage-include-all",
      "--test-coverage-include=src/**",
      "--test-reporter=tap",
      "tests/**/*.test.mjs",
    ],
    { cwd: join(ROOT, "app"), encoding: "utf8", maxBuffer: 256 * 1024 * 1024 },
  );
  if (run.error) throw run.error;
  if (run.status !== 0) {
    console.error(run.stdout?.slice(-4000) ?? "");
    console.error(run.stderr?.slice(-4000) ?? "");
    throw new Error(`the front-end test run failed with exit code ${run.status}`);
  }

  const lines = run.stdout.split(/\r?\n/);
  const start = lines.findIndex((l) => l.includes("start of coverage report"));
  const end = lines.findIndex((l) => l.includes("end of coverage report"));
  if (start === -1 || end === -1) throw new Error("no coverage report in the front-end test output");

  const files = {};
  const stack = [];
  let totals = null;
  for (const raw of lines.slice(start + 1, end)) {
    const line = raw.replace(/^#\s?/, "");
    const cells = line.split("|").map((c) => c.trim());
    if (cells.length < 4) continue;
    const indent = line.length - line.trimStart().length;
    const name = cells[0];
    if (name === "") continue;
    if (name === "all files") {
      totals = { lines: Number(cells[1]), branches: Number(cells[2]), functions: Number(cells[3]) };
      continue;
    }
    if (cells[1] === "") {
      stack[indent] = name;
      stack.length = indent + 1;
      continue;
    }
    const depth = indent;
    const path = [...stack.slice(0, depth), name].join("/");
    if (/\.(ts|mts|js|mjs|cjs)$/.test(name)) files[`app/${path}`] = Number(cells[1]);
  }
  if (!totals) throw new Error("no 'all files' row in the front-end coverage report");

  return { totals, files, not_reached: svelteFiles() };
}

function readBaseline() {
  return existsSync(BASELINE_PATH) ? JSON.parse(readFileSync(BASELINE_PATH, "utf8")) : null;
}

function record(rust, typescript) {
  const previous = readBaseline();
  const keepLower = (before, now) => (typeof before === "number" && before < now ? before : now);
  const merge = (before, now) => {
    if (Array.isArray(now)) return now;
    const out = {};
    for (const [k, v] of Object.entries(now)) {
      if (v !== null && typeof v === "object") out[k] = merge(before?.[k] ?? {}, v);
      else out[k] = previous ? keepLower(before?.[k], v) : v;
    }
    return out;
  };
  const commit = spawnSync("git", ["rev-parse", "--short", "HEAD"], { cwd: ROOT, encoding: "utf8" });
  const body = {
    note: "Coverage BEFORE the refactor (R0). The gate compares the Rust totals and per-crate figures and the TypeScript totals; a compared number must not drop by more than 0.1 point. tools/check-coverage.mjs reads this file; only tools/check-coverage.mjs --record writes it.",
    measured_time: new Date().toISOString(),
    commit: (commit.stdout ?? "").trim(),
    rust: merge(previous?.rust ?? {}, rust),
    typescript: merge(previous?.typescript ?? {}, typescript),
  };
  const ordered = {
    note: body.note,
    measured_time: previous?.measured_time ?? body.measured_time,
    commit: previous?.commit ?? body.commit,
    rust: { totals: body.rust.totals, crates: body.rust.crates },
    typescript: { totals: body.typescript.totals, files: body.typescript.files, not_reached: body.typescript.not_reached },
  };
  writeFileSync(BASELINE_PATH, `${JSON.stringify(ordered, null, 2)}\n`);
  console.log(`coverage baseline written (${body.commit || "no commit"}, ${ordered.measured_time})`);
}

function compare(baseline, rust, typescript) {
  const failures = [];
  const check = (label, before, now) => {
    const drop = Math.round((before - now) * 100) / 100;
    console.log(`${label}: ${now.toFixed(2)} % (before ${before.toFixed(2)} %)`);
    if (drop > TOLERANCE) failures.push(`${label} dropped ${before.toFixed(2)} → ${now.toFixed(2)} %`);
  };

  console.log("Rust");
  for (const key of ["lines", "regions", "functions"]) check(`  total ${key}`, baseline.rust.totals[key], rust.totals[key]);
  for (const crate of Object.keys(baseline.rust.crates).sort()) {
    const now = rust.crates[crate];
    if (!now) {
      failures.push(`${crate} is gone from the coverage report`);
      continue;
    }
    for (const key of ["lines", "regions", "functions"]) check(`  ${crate} ${key}`, baseline.rust.crates[crate][key], now[key]);
  }
  for (const crate of Object.keys(rust.crates).sort()) {
    if (!(crate in baseline.rust.crates)) console.log(`  ${crate} is new (not compared)`);
  }

  console.log("TypeScript");
  for (const key of ["lines", "branches", "functions"]) {
    check(`  total ${key}`, baseline.typescript.totals[key], typescript.totals[key]);
  }
  for (const file of typescript.not_reached) {
    if (!baseline.typescript.not_reached.includes(file)) console.log(`  new file no test loads: ${file}`);
  }
  console.log(`  ${typescript.not_reached.length} source file(s) no test loads (listed in the baseline)`);

  if (failures.length > 0) {
    console.error("coverage gate failed:");
    for (const f of failures) console.error(`  ${f}`);
    process.exit(1);
  }
  console.log("coverage gate passed: no compared number is lower than the baseline");
}

function main() {
  mkdirSync(dirname(BASELINE_PATH), { recursive: true });
  const rust = rustCoverage();
  const typescript = typescriptCoverage();
  if (RECORD) record(rust, typescript);
  else {
    const baseline = readBaseline();
    if (!baseline) throw new Error("tools/baselines/coverage.json is missing: record it with --record");
    compare(baseline, rust, typescript);
  }
}

main();
