// The dates the screens show, derived from a `2026-10-07` date: not fixed screen texts, so they
// live apart from `texts.ts`.

// The title bar's date, "Wed 7 Oct 2026".
const dayFormat = new Intl.DateTimeFormat("en-GB", {
  weekday: "short",
  day: "numeric",
  month: "short",
  year: "numeric",
});

export function formatDay(day: string): string {
  return dayFormat.format(new Date(`${day}T00:00:00`));
}

// The panel's date rows and the rows' dates elsewhere, "Wed 7 Oct", and the panel's month
// headings, "October 2026" (from `2026-10`).
const shortDayFormat = new Intl.DateTimeFormat("en-GB", {
  weekday: "short",
  day: "numeric",
  month: "short",
});

const monthFormat = new Intl.DateTimeFormat("en-GB", { month: "long", year: "numeric" });

export function formatShortDay(day: string): string {
  return shortDayFormat.format(new Date(`${day}T00:00:00`));
}

export function formatMonth(key: string): string {
  return monthFormat.format(new Date(`${key}-01T00:00:00`));
}
