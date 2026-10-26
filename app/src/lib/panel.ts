// The panel's dates by month, pure so the folding rule is tested without a window: newest month
// first, and only the month of the open date and the current month start open.

import type { DateRow } from "./api/panel";

export interface Month {
  // `2026-10`.
  key: string;
  rows: DateRow[];
  startsOpen: boolean;
}

export function months(dates: DateRow[], openDay: string, today: string): Month[] {
  const list: Month[] = [];
  for (const row of dates) {
    const key = row.date.slice(0, 7);
    const last = list[list.length - 1];
    if (last?.key === key) {
      last.rows.push(row);
    } else {
      list.push({
        key,
        rows: [row],
        startsOpen: key === openDay.slice(0, 7) || key === today.slice(0, 7),
      });
    }
  }
  return list;
}
