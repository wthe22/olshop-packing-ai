// The Day screen's Reprints card: which reprints show their orders, and those orders, asked from
// the search index the first time a reprint opens (again after a moment while the index is being
// built). The markup is `ReprintsCard.svelte`.

import * as api from "./api/search";

// While the index is being built the orders are asked again after this many milliseconds.
const LOADING_RETRY_MS = 500;

export class ReprintList {
  // The reprint numbers whose orders are shown.
  open = $state<number[]>([]);
  // Each loaded reprint's orders, by its number.
  orders = $state<Record<number, api.ReprintedOrder[]>>({});
  error = $state("");

  // The date the reprints were made on.
  private readonly day: () => string;

  constructor(day: () => string) {
    this.day = day;
  }

  isOpen = (number: number) => this.open.includes(number);

  toggle = async (number: number) => {
    if (this.isOpen(number)) {
      this.open = this.open.filter((known) => known !== number);
      return;
    }
    this.open = [...this.open, number];
    if (!(number in this.orders)) await this.load(number);
  };

  private load = async (number: number) => {
    try {
      const result = await api.reprintedOrders(this.day(), number);
      if (result.loading) setTimeout(() => this.load(number), LOADING_RETRY_MS);
      else this.orders = { ...this.orders, [number]: result.orders };
    } catch (e) {
      this.error = String(e);
    }
  };
}
