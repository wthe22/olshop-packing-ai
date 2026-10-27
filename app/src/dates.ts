// The dates the screens show, derived from a `2026-10-07` date: not fixed screen texts, so they
// live apart from `texts.ts`.

const weekdayFormat = new Intl.DateTimeFormat("en-GB", { weekday: "short" });
const dayMonthFormat = new Intl.DateTimeFormat("en-GB", { day: "numeric", month: "short" });
const monthFormat = new Intl.DateTimeFormat("en-GB", { month: "long", year: "numeric" });

const dateOf = (day: string) => new Date(`${day}T00:00:00`);

// The panel's date rows and the rows' dates elsewhere, "Wed, 7 Oct".
export function formatShortDay(day: string): string {
  const date = dateOf(day);
  return `${weekdayFormat.format(date)}, ${dayMonthFormat.format(date)}`;
}

// The title bar's date, "Wed, 7 Oct 2026".
export function formatDay(day: string): string {
  return `${formatShortDay(day)} ${dateOf(day).getFullYear()}`;
}

// The panel's month headings, "October 2026" (from `2026-10`).
export function formatMonth(key: string): string {
  return monthFormat.format(new Date(`${key}-01T00:00:00`));
}
