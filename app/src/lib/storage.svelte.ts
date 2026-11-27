// Settings › Storage (settings.md): the data folder's size, **Archive** of a range of dates, and
// *Archive automatically* with its number of days. Turning it on, or a changed number, asks first
// when it would archive any date now, and archives at once; **Off** removes the number.

import * as api from "./api/archive";
import type { Archiving } from "./archiving.svelte";
import type { DateRange } from "./searchRange";

// The number *Archive automatically* starts with.
export const FIRST_DAYS = 60;

export class Storage {
  used = $state<api.StorageUsed | null>(null);
  range = $state<DateRange>({ from: "", to: "" });
  // What **Archive** of the range would delete; `null` while the range is not whole.
  preview = $state<api.ArchivePreview | null>(null);
  // The number box.
  days = $state(FIRST_DAYS);
  // *Archive automatically* is on: `packing-app.toml` holds the number.
  auto = $state(false);
  error = $state("");

  private readonly archiving: Archiving;

  constructor(archiving: Archiving, range: DateRange) {
    this.archiving = archiving;
    this.range = range;
  }

  // Read each time Settings opens.
  load = async () => {
    await this.attempt(async () => {
      const days = await api.archiveAfterDays();
      this.auto = days !== null;
      this.days = days ?? FIRST_DAYS;
      await this.reread();
    });
  };

  // The sizes and the range's count, as the data folder is now.
  private reread = async () => {
    this.used = await api.storage();
    const { from, to } = this.range;
    this.preview = from && to && from <= to ? await api.archivePreview({ kind: "range", from, to }) : null;
  };

  setRange = async (range: DateRange) => {
    this.range = range;
    await this.attempt(this.reread);
  };

  // **Archive** of the range, asked first.
  archiveRange = async () => {
    const preview = this.preview;
    if (!preview || preview.dates.length === 0 || !(await this.archiving.ask(preview))) return;
    await this.archiving.run({ kind: "range", ...this.range });
    await this.attempt(this.reread);
  };

  // **Off** / **On**.
  turn = async (on: boolean) => {
    if (on) {
      await this.apply(this.days);
      return;
    }
    await this.attempt(async () => {
      await api.setArchiveAfterDays(null);
      this.auto = false;
    });
  };

  // The number the box lost its focus with; written at once while *Archive automatically* is on.
  changeDays = async (typed: number) => {
    const days = Number.isFinite(typed) ? Math.max(1, Math.round(typed)) : FIRST_DAYS;
    this.days = days;
    if (this.auto) await this.apply(days);
  };

  // Write `days`, asking first when it would archive any date now, and archive those at once.
  // Cancelled, the number written before stays.
  private apply = async (days: number) => {
    await this.attempt(async () => {
      const scope: api.ArchiveScope = { kind: "older_than", days };
      const preview = await api.archivePreview(scope);
      const archives = preview.dates.length > 0;
      if (archives && !(await this.archiving.ask(preview))) {
        await this.load();
        return;
      }
      await api.setArchiveAfterDays(days);
      this.auto = true;
      if (archives) await this.archiving.run(scope);
      await this.reread();
    });
  };

  private attempt = async (job: () => Promise<void>) => {
    this.error = "";
    try {
      await job();
    } catch (e) {
      this.error = String(e);
    }
  };
}
