// The check before saving's window side (07 › *Check before saving*): what *Save batch* returns
// decides whether the findings dialog opens and what it lists. Pure, so the flow is tested
// without a window (the stage text lives with the other texts, in `texts.ts`).
//
//   saved: true                → the Day screen shows the new batch; no dialog
//   saved: false, findings     → the dialog with the two lists, *Save anyway* / *Cancel*
//   saved: false, no findings  → nothing to show (the save failed for another reason)

import type { SaveResult } from "./api/batch";

// The findings the dialog lists when a save stopped: errors and warnings, kept apart.
export interface Findings {
  errors: string[];
  warnings: string[];
}

// What the dialog shows for a save result, or `null` when there is nothing to show: a batch that
// was saved (cleanly or with *Save anyway*), or a stop that listed no finding.
export function findingsOf(result: SaveResult): Findings | null {
  if (result.saved) return null;
  if (result.errors.length === 0 && result.warnings.length === 0) return null;
  return { errors: result.errors, warnings: result.warnings };
}
