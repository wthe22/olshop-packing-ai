// What the Day screen does with a date's outputs: **Print** (the preview over the app's window; a
// group PDF is marked *Sent* as it opens), the *Sent* mark's ✕, the **Printed** tick, *Open* in the
// PC's PDF viewer and **Restore**. An output that is not there any more is written again first,
// checked like a **Restore**, with the shell's dialog.

import * as dayData from "./api/day";
import type { Shell } from "./shell.svelte";
import { printPdf } from "./printPdf";
import { togglePrinted } from "./printed";
import { texts } from "../texts";

// The label group a **Print** marks *Sent*; a summary PDF or a reprint marks nothing.
export interface SentMark {
  batch: number;
  group: string;
}

export class DayOutputs {
  // The preview itself; the tests put a fake in its place, as they have no window.
  preview: (bytes: ArrayBuffer) => Promise<void> = printPdf;

  private readonly shell: Shell;

  constructor(shell: Shell) {
    this.shell = shell;
  }

  // **Print** output `file` of `day`. *Sent* is written once the preview is open.
  print = async (day: string, file: string, mark: SentMark | null) => {
    const ready = await this.shell.waitFor(texts.day.writingAgain(file), () =>
      dayData.printOutput(day, file),
    );
    if (!ready) return;
    if (ready.kind === "not_kept") {
      this.shell.error = texts.day.restoreFailed(file, ready.errors);
    } else {
      try {
        await this.preview(await dayData.outputPdf(day, file));
        if (mark) await dayData.setSent(day, mark.batch, mark.group, true);
      } catch (e) {
        this.shell.error = String(e);
      }
    }
    if ((mark || ready.kind !== "opened") && day === this.shell.day) {
      await this.shell.refreshOverview();
    }
  };

  // The *Sent* mark's ✕: the print was cancelled or failed.
  unsend = async (row: dayData.BatchView, pdf: dayData.GroupPdfView) => {
    try {
      await dayData.setSent(this.shell.day, row.batch, pdf.group, false);
      pdf.sent = null;
    } catch (e) {
      this.shell.error = String(e);
    }
  };

  // The **Printed** tick of one group PDF: write it and update the batch's rows in place, without
  // reloading the whole overview. The tick rule is the pure [`togglePrinted`].
  setPrinted = async (row: dayData.BatchView, pdf: dayData.GroupPdfView, printed: boolean) => {
    try {
      await dayData.setPrinted(this.shell.day, row.batch, pdf.group, printed);
      const ticked = row.pdfs.filter((item) => item.printed).map((item) => item.group);
      const groups = togglePrinted(ticked, pdf.group, printed);
      for (const item of row.pdfs) item.printed = groups.includes(item.group);
    } catch (e) {
      this.shell.error = String(e);
    }
  };

  // **Restore** an output found changed: written again from the stored inputs and checked like
  // the original; a finding keeps nothing and is shown.
  restore = async (file: string) => {
    const result = await this.shell.waitFor(texts.day.writingAgain(file), () =>
      dayData.restoreOutput(this.shell.day, file),
    );
    if (!result) return;
    if (!result.restored) this.shell.error = texts.day.restoreFailed(file, result.errors);
    await this.shell.refreshOverview();
  };

  // *Open* on a group PDF or a summary PDF of `day`, in the PC's PDF viewer; a summary PDF written
  // again has today's layout.
  open = async (day: string, file: string) => {
    const opened = await this.shell.waitFor(texts.day.writingAgain(file), () =>
      dayData.openOutput(day, file),
    );
    if (!opened) return;
    if (opened.kind === "not_kept") this.shell.error = texts.day.restoreFailed(file, opened.errors);
    if (opened.kind !== "opened" && day === this.shell.day) await this.shell.refreshOverview();
  };
}
