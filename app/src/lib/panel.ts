// A list of dates by month, pure so the folding rule is tested without a window: newest month
// first, and only the month of the open date and the current month start open. The side panel and
// an account's dates both fold this way.

export interface Month<T extends { date: string }> {
  // `2026-10`.
  key: string;
  rows: T[];
  startsOpen: boolean;
}

export function months<T extends { date: string }>(
  dates: T[],
  openDay: string,
  today: string,
): Month<T>[] {
  const list: Month<T>[] = [];
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

// Whether a date's row menu offers **Archive** (archive.md): a date with a saved batch that is not
// archived, damaged or today. The back end refuses the rest too, a date with `.partial/` among them.
export function canArchive(row: {
  today: boolean;
  orders: number | null;
  damaged: boolean;
  folder: string | null;
  archived: boolean;
}): boolean {
  return row.folder !== null && !row.today && !row.damaged && !row.archived && (row.orders ?? 0) > 0;
}
