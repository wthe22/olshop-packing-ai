// Archiving (archive.md) as the shell runs it: the ticked dates and the dates of *Automatic* from
// the Archive page, and the dates older than that number at each start. Each asks first, naming
// the dates and the size of their PDFs, except the one at start, which the owner chose on the
// page. More than one date shows its progress in the shell's dialog.

import * as archive from "./api/archive";
import type { Shell } from "./shell.svelte";
import { access } from "./readOnly.svelte";
import { confirmBox } from "./confirm.svelte";
import { formatSize } from "../size";
import { texts } from "../texts";

export class Archiving {
  private readonly shell: Shell;

  constructor(shell: Shell) {
    this.shell = shell;
  }

  // The question for archiving `dates` dates whose PDFs take `bytes`; `false` when the owner
  // cancels.
  ask = (dates: number, bytes: number): Promise<boolean> =>
    confirmBox.ask(texts.archive.ask(dates, formatSize(bytes)));

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

  // At start, after the migration's question: the dates older than *Automatic*'s
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
