// A size on disk as the panel and Settings show it.
//
// Run in `app/`: `npm test`.

import { test } from "node:test";
import assert from "node:assert/strict";

import "./tsModule.mjs";
const { formatSize } = await import("../src/size.ts");

test("a size is in KB, MB or GB, one decimal under 10", () => {
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;
  for (const [bytes, shown] of [
    [0, "0.0 KB"],
    [512, "0.5 KB"],
    [12 * KB, "12 KB"],
    [100 * KB, "100 KB"],
    [0.1 * MB, "0.1 MB"],
    [2.25 * MB, "2.3 MB"],
    [9.96 * MB, "10 MB"],
    [84 * MB, "84 MB"],
    [1023 * MB, "1023 MB"],
    [GB, "1.0 GB"],
    [1.2 * GB, "1.2 GB"],
    [42 * GB, "42 GB"],
  ]) {
    assert.equal(formatSize(bytes), shown, String(bytes));
  }
});
