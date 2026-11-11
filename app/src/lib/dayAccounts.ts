// The accounts of a date's live batches: the Day summary button is a menu when there are several.

import type { BatchView } from "./api/day";

// The accounts that have orders in `batches`, most orders first.
export function accountsOf(batches: BatchView[]): string[] {
  const orders = new Map<string, number>();
  for (const batch of batches) {
    for (const { account, orders: n } of batch.accounts) {
      orders.set(account, (orders.get(account) ?? 0) + n);
    }
  }
  return [...orders].sort((a, b) => b[1] - a[1]).map(([account]) => account);
}
