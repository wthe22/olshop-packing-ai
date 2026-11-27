// **Update** as it runs (start.md, *Update the data folder?*): the date running with a bar for the
// whole run and the step under it, from the `migration-step` event; at the end, the batches that
// were not read.

import * as start from "./api/start";
import { updateTexts as words, type SkippedBatch } from "../updateTexts";

export class Update {
  running = $state(false);
  error = $state("");
  // *Updating 2026-11-24, 3 of 29*; `null` before the first date.
  line = $state<string | null>(null);
  // The step under it; `null` between steps.
  step = $state<string | null>(null);
  // The bar: dates done, with the running date's share of its step.
  done = $state(0);
  of = $state(0);
  // Filled when the run ended with batches not read; the card then waits for **Continue**.
  skipped = $state<SkippedBatch[]>([]);
  // The dates finished before the running one.
  private finished = 0;

  // Whether the run finished with every batch read, so the Day screen opens at once.
  run = async (): Promise<boolean> => {
    if (this.running) return false;
    this.running = true;
    this.error = "";
    this.skipped = [];
    const stop = await start.onMigrationStep(this.follow);
    let whole = false;
    try {
      this.skipped = (await start.updateData()).flatMap((date) =>
        date.skipped.map((batch) => ({
          date: date.date,
          batch: batch.batch,
          error: batch.reason.kind === "failed" ? batch.reason.error : null,
        })),
      );
      whole = this.skipped.length === 0;
    } catch (e) {
      this.error = String(e);
      this.line = null;
    }
    stop();
    this.step = null;
    this.running = false;
    return whole;
  };

  follow = (step: start.MigrationStep) => {
    if (step.kind === "starting") {
      this.line = words.updating(step.date, step.number, step.of);
      this.step = null;
      this.finished = step.number - 1;
      this.done = this.finished;
      this.of = step.of;
      return;
    }
    const [at, of] = step.kind === "moving" ? [step.file, step.files] : [step.page, step.pages];
    this.step = step.kind === "moving" ? words.moving(at, of) : words.reading(at, of);
    this.done = this.finished + at / Math.max(of, 1);
  };
}
