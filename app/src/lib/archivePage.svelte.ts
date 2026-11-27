// The Archive page's state (ui/archive.md): every date with its size, the ticked dates and what
// archiving them deletes, and *Automatic* with its number of days. **Archive**, turning *Automatic*
// on and a changed number ask first when they would archive any date; **Off** removes the number.
// Read-only, nothing is ticked or written. The markup is `ArchiveScreen.svelte`.

import * as api from "./api/archive";
import type { Archiving } from "./archiving.svelte";
import { access } from "./readOnly.svelte";

// The number *Automatic* starts with.
export const FIRST_DAYS = 60;
// While the search index is being built the page is read again after this many milliseconds.
const LOADING_RETRY_MS = 500;

export class ArchivePage {
  view = $state<api.ArchivePage | null>(null);
  ticked = $state<string[]>([]);
  // The number box.
  days = $state(FIRST_DAYS);
  // *Automatic* is on: `packing-app.toml` holds the number.
  auto = $state(false);
  error = $state("");
  // The date last ticked or cleared by a click without Shift: where a Shift+click's range starts.
  private anchor: string | null = null;
  private closed = false;

  private readonly archiving: Archiving;

  constructor(archiving: Archiving) {
    this.archiving = archiving;
  }

  load = async () => {
    await this.attempt(async () => {
      const days = await api.archiveAfterDays();
      this.auto = days !== null;
      this.days = days ?? FIRST_DAYS;
      await this.reread();
    });
  };

  // The page stopped showing: no more reads while the index is built.
  close = () => {
    this.closed = true;
  };

  // The dates as the data folder is now; a ticked date that can no longer be archived is cleared.
  private reread = async () => {
    if (this.closed) return;
    const view = await api.archivePage();
    this.view = view;
    this.ticked = this.ticked.filter((date) => this.canTick(date));
    if (view.loading) setTimeout(() => void this.attempt(this.reread), LOADING_RETRY_MS);
  };

  private rows = () => this.view?.dates ?? [];
  private canTick = (date: string) => this.rows().some((row) => row.date === date && row.archive !== null);
  isTicked = (date: string) => this.ticked.includes(date);

  // A click on a date's tick; with Shift, every date between the last click and this one is ticked.
  tick = (date: string, shift: boolean) => {
    if (access.readOnly || !this.canTick(date)) return;
    const dates = this.rows().map((row) => row.date);
    const from = this.anchor === null ? -1 : dates.indexOf(this.anchor);
    if (shift && from >= 0) {
      const to = dates.indexOf(date);
      const between = dates.slice(Math.min(from, to), Math.max(from, to) + 1);
      this.ticked = [...new Set([...this.ticked, ...between.filter(this.canTick)])];
      return;
    }
    this.ticked = this.isTicked(date) ? this.ticked.filter((one) => one !== date) : [...this.ticked, date];
    this.anchor = date;
  };

  // The dates of `rows` that can be archived.
  tickable = (rows: api.ArchiveRow[]) => rows.filter((row) => row.archive !== null).map((row) => row.date);

  // A month's tick is on when every date of it that can be archived is ticked.
  monthTicked = (rows: api.ArchiveRow[]) => {
    const dates = this.tickable(rows);
    return dates.length > 0 && dates.every(this.isTicked);
  };

  // A click on a month's tick ticks its dates that can be archived, or clears them when all are.
  tickMonth = (rows: api.ArchiveRow[]) => {
    if (access.readOnly) return;
    const dates = this.tickable(rows);
    this.ticked = this.monthTicked(rows)
      ? this.ticked.filter((date) => !dates.includes(date))
      : [...new Set([...this.ticked, ...dates])];
  };

  // What the foot shows: the ticked dates and their PDFs' size.
  bytes = () =>
    this.rows()
      .filter((row) => this.isTicked(row.date))
      .reduce((sum, row) => sum + (row.archive?.bytes ?? 0), 0);

  // **Archive**, asked first.
  archive = async () => {
    const dates = [...this.ticked];
    if (dates.length === 0 || !(await this.archiving.ask(dates.length, this.bytes()))) return;
    await this.archiving.run({ kind: "dates", dates });
    this.ticked = [];
    this.anchor = null;
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

  // The number the box lost its focus with; written at once while *Automatic* is on.
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
      if (archives && !(await this.archiving.ask(preview.dates.length, preview.bytes))) {
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
