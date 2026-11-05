// **Print** and the *Sent* mark on the Day screen's outputs, against the faked back end: the output
// is made ready, its bytes go to the preview (a fake here, as there is no window) and a group PDF
// is then marked *Sent*; its ✕ takes the mark back.
//
// Run in `app/`: `npm test`.

import { test } from "node:test";
import assert from "node:assert/strict";

import { useBackend } from "./tsModule.mjs";
import { Shell, backend } from "./shellKit.mjs";
import { texts } from "../src/texts.ts";

const BYTES = new Uint8Array([37, 80, 68, 70]).buffer;
const NOT_KEPT = ["page 3 of labels_1.pdf is in no output."];

function printing(fail = "") {
  const calls = useBackend({
    ...backend(),
    print_output: (args) =>
      args.file === "1 summary.pdf"
        ? { kind: "written_again" }
        : args.file === "1-B Spion x2.pdf"
          ? { kind: "not_kept", errors: NOT_KEPT }
          : { kind: "opened" },
    output_pdf: () => BYTES,
    set_sent: () => {
      if (fail === "set_sent") throw "the batch is gone";
    },
  });
  const shell = new Shell();
  shell.day = "2026-10-07";
  const previews = [];
  shell.outputs.preview = async (bytes) => {
    if (fail === "preview") throw new Error("no printer");
    previews.push(bytes);
  };
  const sent = () => calls.filter((call) => call.command === "set_sent").map((call) => call.args);
  const recounts = () => calls.filter((call) => call.command === "day_overview").length;
  return { calls, shell, previews, sent, recounts };
}

test("**Print** shows a group PDF's preview, then marks it Sent and recounts the day", async () => {
  const { shell, previews, sent, recounts } = printing();
  await shell.outputs.print("2026-10-07", "1-A Sepatu x2.pdf", { batch: 1, group: "A" });
  assert.deepEqual(previews, [BYTES]);
  assert.deepEqual(sent(), [{ day: "2026-10-07", batch: 1, group: "A", sent: true }]);
  assert.equal(recounts(), 1, "the Sent time comes from the journal");
  assert.equal(shell.error, "");
});

test("a summary PDF or reprint marks nothing; one written again recounts its own date only", async () => {
  const { shell, previews, sent, recounts } = printing();
  await shell.outputs.print("2026-10-07", "R1 x3.pdf", null);
  assert.equal(recounts(), 0, "nothing changed");
  await shell.outputs.print("2026-10-07", "1 summary.pdf", null);
  assert.equal(recounts(), 1);
  await shell.outputs.print("2026-10-06", "1 summary.pdf", null);
  assert.equal(recounts(), 1, "another date's Day screen is not shown");
  assert.equal(previews.length, 3);
  assert.deepEqual(sent(), []);
});

test("an output not kept is told and not printed; a failed preview marks nothing", async () => {
  const kept = printing();
  await kept.shell.outputs.print("2026-10-07", "1-B Spion x2.pdf", { batch: 1, group: "B" });
  assert.equal(kept.shell.error, texts.day.restoreFailed("1-B Spion x2.pdf", NOT_KEPT));
  assert.deepEqual([kept.previews, kept.sent()], [[], []]);

  const failed = printing("preview");
  await failed.shell.outputs.print("2026-10-07", "1-A Sepatu x2.pdf", { batch: 1, group: "A" });
  assert.equal(failed.shell.error, "Error: no printer");
  assert.deepEqual(failed.sent(), []);
});

test("the Sent mark's ✕ takes it back in place; a failed write is shown and keeps it", async () => {
  const { shell, sent } = printing();
  const pdf = { group: "A", file: "1-A Sepatu x2.pdf", sent: "09:14", printed: false };
  const row = { batch: 1, pdfs: [pdf] };
  await shell.outputs.unsend(row, pdf);
  assert.equal(pdf.sent, null);
  assert.deepEqual(sent(), [{ day: "2026-10-07", batch: 1, group: "A", sent: false }]);

  const failing = printing("set_sent");
  const kept = { ...pdf, sent: "09:14" };
  await failing.shell.outputs.unsend({ batch: 1, pdfs: [kept] }, kept);
  assert.equal(kept.sent, "09:14");
  assert.equal(failing.shell.error, "the batch is gone");
});
