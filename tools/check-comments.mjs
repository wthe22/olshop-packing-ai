#!/usr/bin/env node
// Comment gate (code-rules.md, rule 6): fails on a comment that carries a decision/question/review ID,
// a task number, a pointer to a numbered design file or a banner line. There is no baseline: the count
// is zero. A line counts as a comment when it starts with a comment marker or has a trailing `//`.

import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const SCANNED = ["crates", "app/src", "app/src-tauri/src", "app/tests", "tools"];
const EXTENSIONS = /\.(rs|ts|mts|mjs|js|svelte|css|sh|toml)$/;
const SKIPPED_DIRS = new Set(["target", "node_modules", ".svelte-kit", "build", "analysis", "baselines"]);

const COMMENT_LINE = /^\s*(\/\/|\/\*|\*|#|<!--)|\s\/\/\s/;
const FORBIDDEN = {
  "an ID": /\b[DQRF]-(LABEL|RULES|APP|UI)-\d+/,
  "a task number": /\btasks? \d+\.\d+|\(\d\.\d+\)/,
  "a numbered design pointer": /\b0\d ›/,
  "a banner": /(\/\/|#)\s*[-=]{5,}/,
};

function files(dir, out = []) {
  for (const entry of readdirSync(join(ROOT, dir), { withFileTypes: true })) {
    const rel = `${dir}/${entry.name}`;
    if (entry.isDirectory()) {
      if (!SKIPPED_DIRS.has(entry.name)) files(rel, out);
    } else if (EXTENSIONS.test(entry.name) && entry.name !== "check-comments.mjs") {
      out.push(rel);
    }
  }
  return out;
}

const found = [];
for (const dir of SCANNED) {
  for (const file of files(dir)) {
    readFileSync(join(ROOT, file), "utf8").split(/\r?\n/).forEach((line, index) => {
      if (!COMMENT_LINE.test(line)) return;
      for (const [what, pattern] of Object.entries(FORBIDDEN)) {
        if (pattern.test(line)) found.push(`${file}:${index + 1}: comment with ${what}`);
      }
    });
  }
}

if (found.length > 0) {
  console.error(found.join("\n"));
  console.error(`comment gate failed: ${found.length} comment line(s)`);
  process.exit(1);
}
console.log("comment gate passed: no ID, task number, numbered pointer or banner in a comment");
