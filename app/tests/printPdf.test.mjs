// **Print**'s preview frame, against a fake page: the PDF's bytes go into a hidden frame whose
// `print()` is called once it has loaded, and the frame of the print before is removed.
//
// Run in `app/`: `npm test`.

import { test } from "node:test";
import assert from "node:assert/strict";

import { printPdf } from "../src/lib/printPdf.ts";

// A page whose frames load at once; `fail` makes the frame's `print()` throw.
function fakePage(fail = false) {
  const frames = [];
  globalThis.document = {
    createElement: (tag) => {
      const frame = {
        tag,
        style: {},
        printed: 0,
        removed: false,
        remove() {
          this.removed = true;
        },
        contentWindow: {
          print() {
            if (fail) throw new Error("no printer");
            frame.printed += 1;
          },
        },
      };
      frames.push(frame);
      return frame;
    },
    body: { append: (frame) => frame.onload() },
  };
  return frames;
}

test("the bytes are printed from a hidden frame, which the next print replaces", async () => {
  const frames = fakePage();
  const bytes = new Uint8Array([37, 80, 68, 70]).buffer;
  await printPdf(bytes);
  await printPdf(bytes);
  assert.deepEqual(
    frames.map((frame) => [frame.tag, frame.printed, frame.removed]),
    [
      ["iframe", 1, true],
      ["iframe", 1, false],
    ],
  );
  assert.match(frames[1].src, /^blob:/);
  assert.match(frames[1].style.cssText, /visibility: hidden/);
});

test("a print that throws rejects", async () => {
  fakePage(true);
  await assert.rejects(printPdf(new ArrayBuffer(4)), /no printer/);
});
