// Settings › *Archive automatically* (ui/settings.md): the number of days and Off / On. Turning it on
// and a changed number ask first when they would archive any date now; **Off** removes the number.

import * as api from "./api/archive";
import type { Archiving } from "./archiving.svelte";

// The number the box starts with.
export const FIRST_DAYS = 60;

export class ArchiveAuto {
  // The number box.
  days = $state(FIRST_DAYS);
  // On: `packing-app.toml` holds the number.
  auto = $state(false);
  error = $state("");

  private readonly archiving: Archiving;

  constructor(archiving: Archiving) {
    this.archiving = archiving;
  }

  load = async () => {
    await this.attempt(async () => {
      const days = await api.archiveAfterDays();
      this.auto = days !== null;
      this.days = days ?? FIRST_DAYS;
    });
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

  // The number the box lost its focus with; written at once while on.
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
