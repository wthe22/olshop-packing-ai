// The dates a search covers: the open date, a range that ends today, a range of the owner's own,
// or every date. Pure, so the choices are tested as a table.

export type RangePreset = "day" | "yesterday" | "week" | "month" | "custom" | "all";

export const RANGE_PRESETS: RangePreset[] = ["day", "yesterday", "week", "month", "custom", "all"];

// Both ends included, as `2026-10-07`.
export interface DateRange {
  from: string;
  to: string;
}

// `day` moved by `days` (negative is earlier).
export function shiftDay(day: string, days: number): string {
  const [year, month, date] = day.split("-").map(Number);
  return new Date(Date.UTC(year, month - 1, date + days)).toISOString().slice(0, 10);
}

// The range a choice stands for, or `null` for every date. *Custom range* keeps the range shown.
export function presetRange(
  preset: RangePreset,
  openDay: string,
  today: string,
  shown: DateRange,
): DateRange | null {
  switch (preset) {
    case "day":
      return { from: openDay, to: openDay };
    case "yesterday":
      return { from: shiftDay(today, -1), to: shiftDay(today, -1) };
    case "week":
      return { from: shiftDay(today, -6), to: today };
    case "month":
      return { from: shiftDay(today, -29), to: today };
    case "custom":
      return shown;
    case "all":
      return null;
  }
}
