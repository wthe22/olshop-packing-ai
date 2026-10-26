// The check before saving's window side: what the save result decides the findings dialog shows,
// and the stage text of a `save-progress` event. No window opens and nothing is written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { findingsOf } from "../src/lib/saveCheck.ts";
import { texts, saveStageText } from "../src/texts.ts";

// A save result, clean unless the caller asks otherwise.
function result({ saved = true, errors = [], warnings = [] } = {}) {
  return { batch: 1, pdfs: 2, packing_lists: 1, saved, errors, warnings };
}

test("a stopped save opens the dialog with its findings, and a saved one opens none", () => {
  // *Save anyway* wrote the batch: the green line and the Day card show it, no dialog.
  assert.equal(findingsOf(result()), null);
  assert.equal(findingsOf(result({ errors: ["PDF 1 is missing a page"] })), null);

  // A stopped save shows both lists, kept apart.
  assert.deepEqual(
    findingsOf(
      result({
        saved: false,
        errors: ["PDF 1 is missing a page"],
        warnings: ["PDF 2 is not in the run order"],
      }),
    ),
    {
      errors: ["PDF 1 is missing a page"],
      warnings: ["PDF 2 is not in the run order"],
    },
  );

  // A warnings-only stop still opens it: the swapped-order case is a warning only, but the batch
  // is not saved until *Save anyway*.
  assert.deepEqual(
    findingsOf(
      result({ saved: false, warnings: ["PDF 1 is not in the run order"] }),
    ),
    { errors: [], warnings: ["PDF 1 is not in the run order"] },
  );

  // A stop with no finding shows nothing.
  assert.equal(findingsOf(result({ saved: false })), null);
});

test("the dialog's words and the stage lines are the designed words", () => {
  assert.equal(texts.plan.saveAnyway, "Save anyway");
  assert.equal(texts.plan.saveCancel, "Cancel — nothing is saved");
  assert.equal(texts.plan.checkTitle, "The verifier found the outputs differ from the record.");
  assert.equal(texts.plan.checkErrors(2), "Errors: 2");
  assert.equal(texts.plan.checkWarnings(1), "Warnings: 1");

  // The four stages of the design read in plain words; *Writing* counts the files.
  assert.deepEqual(
    [
      saveStageText("CopyingInputs", 0, 2),
      saveStageText("Writing", 1, 3),
      saveStageText("Verifying", 0, 1),
      saveStageText("Saving", 0, 1),
    ],
    ["Copying inputs …", "Writing …  file 2 of 3", "Verifying the outputs …", "Saving …"],
  );
  // Before the first event the Plan screen says it is saving.
  assert.equal(texts.plan.saving, "Saving …");

  // The Day card line counts errors, singular where it should be.
  assert.equal(texts.day.savedWithErrors(1), "Saved with 1 error ▸");
  assert.equal(texts.day.savedWithErrors(2), "Saved with 2 errors ▸");
});
