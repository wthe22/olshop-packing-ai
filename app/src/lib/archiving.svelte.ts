// Archiving (archive.md) as the shell runs it: one date from the panel's row menu, the dates of a
// range or of *Archive automatically* from Settings, and the dates older than that number at each
// start. Each asks first, naming the dates, the PDFs and their size, except the one at start, which
// the owner chose in Settings. More than one date shows its progress in the shell's dialog.

import * as archive from "./api/archive";
import type { Shell } from "./shell.svelte";
import { access } from "./readOnly.svelte";
import { confirmBox } from "./confirm.svelte";
import { formatDay } from "../dates";
import { formatSize } from "../size";
import { texts } from "../texts";

export class Archiving {
  private readonly shell: Shell;

  constructor(shell: Shell) {
    this.shell = shell;
  }

  // The question for what `preview` would delete; `false` when the owner cancels.
  ask = (preview: archive.ArchivePreview): Promise<boolean> =>
    confirmBox.ask(
      texts.archive.ask(preview.dates.map(formatDay), preview.pdfs, formatSize(preview.bytes)),
    );

  // Archive what `scope` covers; when anything changed, the panel and the open date are shown
  // again. A PDF that was not deleted is named.
  run = async (scope: archive.ArchiveScope): Promise<archive.ArchiveOutcome | null> => {
    if (this.shell.writing) return null;
    this.shell.writing = true;
    const stop = await archive.onArchiveStep((step) => {
      if (step.of > 1) this.shell.working = texts.archive.progress(step.date, step.number, step.of);
    });
    let outcome: archive.ArchiveOutcome | null = null;
    try {
      outcome = await archive.archiveScope(scope);
      if (outcome.left.length > 0) {
        this.shell.error = texts.archive.left(outcome.left.map((pdf) => `${pdf.date}\\${pdf.file}`));
      }
    } catch (e) {
      this.shell.error = String(e);
    }
    stop();
    this.shell.working = null;
    this.shell.writing = false;
    if (outcome === null || outcome.archived.length > 0 || outcome.left.length > 0) {
      await Promise.all([this.shell.refreshPanel(), this.shell.refreshOverview()]);
    }
    return outcome;
  };

  // **Archive** in a date's row menu.
  date = async (day: string) => {
    const scope: archive.ArchiveScope = { kind: "range", from: day, to: day };
    try {
      const preview = await archive.archivePreview(scope);
      if (preview.dates.length === 0) {
        this.shell.error = texts.archive.nothing;
        return;
      }
      if (!(await this.ask(preview))) return;
    } catch (e) {
      this.shell.error = String(e);
      return;
    }
    await this.run(scope);
  };

  // At start, after the migration's question: the dates older than *Archive automatically*'s
  // number, when it is on. Read-only, nothing.
  atStart = async () => {
    if (access.readOnly) return;
    try {
      const days = await archive.archiveAfterDays();
      if (days !== null) await this.run({ kind: "older_than", days });
    } catch (e) {
      this.shell.error = String(e);
    }
  };
}
