// **Update** as it runs (start.md, *Update the data folder?*): a line for each date done and one
// for the step running, from the `migration-step` event; the dates the back end returns at the end
// are the lines kept.

import * as start from "./api/start";
import { updateTexts as words } from "../updateTexts";

export interface UpdateLine {
  text: string;
  // A batch that was not read.
  skipped: boolean;
}

export const linesOf = (date: start.DateUpdated): UpdateLine[] => [
  { text: words.updated(date.date, date.batches), skipped: false },
  ...date.skipped.map((batch) => ({ text: words.skipped(batch.batch, batch.reason), skipped: true })),
];

export class Update {
  running = $state(false);
  done = $state(false);
  error = $state("");
  lines = $state<UpdateLine[]>([]);
  // The step running; `null` between steps.
  current = $state<string | null>(null);

  run = async () => {
    if (this.running) return;
    this.running = true;
    this.error = "";
    this.lines = [];
    const stop = await start.onMigrationStep(this.step);
    try {
      this.lines = (await start.updateData()).flatMap(linesOf);
      this.done = true;
    } catch (e) {
      this.error = String(e);
    }
    stop();
    this.current = null;
    this.running = false;
  };

  step = (step: start.MigrationStep) => {
    if (step.kind === "reading") {
      this.current = words.reading(step.date, step.batch, step.page, step.pages);
      return;
    }
    this.current = null;
    this.lines = [...this.lines, ...linesOf(step)];
  };
}
