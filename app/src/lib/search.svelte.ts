// The Search screen's state and commands: the filter boxes turned into filter text by the engine's
// printer, the search on the index over the open date or all dates, the ticked rows and their
// reprint. Searching changes nothing; only **Reprint** writes a file. The markup is
// `SearchScreen.svelte`.

import * as api from "./api/search";
import { treeToFilter } from "./api/rules";
import { newRow, newTree, toRustTree, type UiTree } from "./conditions";
import { texts } from "../texts";

// While the index is being built the search is asked again after this many milliseconds.
const LOADING_RETRY_MS = 500;

const key = (row: { day: string; order_id: string }) => `${row.day} ${row.order_id}`;

export class Search {
  tree = $state<UiTree>(firstTree());
  scope = $state<api.SearchScope>("day");
  busy = $state(false);
  loading = $state(false);
  // A filter that does not parse, or a failed command.
  error = $state("");
  // The last search's rows; `null` before the first and after any change, so the rows on screen
  // always match the boxes and the scope shown.
  result = $state<api.SearchResults | null>(null);
  ticked = $state<string[]>([]);
  // The rows whose trace line is open.
  traced = $state<string[]>([]);
  made = $state<api.ReprintMade | null>(null);
  // The verifier's findings of a reprint that was not kept.
  findings = $state<string[]>([]);

  // The open date, read when a search runs; and what lets the Day screen and the panel recount
  // after a reprint.
  private readonly day: () => string;
  private readonly onReprinted: () => void;

  constructor(day: () => string, onReprinted: () => void) {
    this.day = day;
    this.onReprinted = onReprinted;
  }

  invalidate = () => {
    this.error = "";
    this.result = null;
    this.ticked = [];
    this.traced = [];
  };

  setScope = (scope: api.SearchScope) => {
    this.scope = scope;
    this.invalidate();
  };

  run = async () => {
    this.busy = true;
    this.invalidate();
    this.made = null;
    this.findings = [];
    try {
      const filter = await treeToFilter(toRustTree(this.tree));
      const result = await api.searchOrders(filter, this.scope, this.day());
      this.loading = result.loading;
      if (result.loading) setTimeout(this.run, LOADING_RETRY_MS);
      else this.result = result;
    } catch (e) {
      this.error = String(e);
    }
    this.busy = false;
  };

  isTicked = (row: api.SearchRow) => this.ticked.includes(key(row));
  isTraced = (row: api.SearchRow) => this.traced.includes(key(row));

  toggle = (row: api.SearchRow) => {
    this.ticked = flip(this.ticked, key(row));
  };

  toggleTrace = (row: api.SearchRow) => {
    this.traced = flip(this.traced, key(row));
  };

  // **Select all results**: every row the filter found, or none.
  selectAll = (all: boolean) => {
    this.ticked = all && this.result ? this.result.rows.map(key) : [];
  };

  allTicked = () =>
    this.result !== null && this.result.rows.length > 0 && this.ticked.length === this.result.rows.length;

  // **Reprint `<n>` orders**: one PDF of the ticked orders' label pages, written into today.
  reprint = async () => {
    const rows = this.result?.rows.filter(this.isTicked) ?? [];
    if (rows.length === 0) return;
    this.busy = true;
    this.made = null;
    this.findings = [];
    this.error = "";
    try {
      const result = await api.reprint(rows.map((row) => ({ day: row.day, order_id: row.order_id })));
      this.made = result.made;
      this.findings = result.errors;
      if (result.made) {
        this.ticked = [];
        this.onReprinted();
      }
    } catch (e) {
      this.error = String(e);
    }
    this.busy = false;
  };
}

// One row: the tracking ID, the box the owner fills first.
function firstTree(): UiTree {
  const tree = newTree();
  const row = newRow("tracking_id");
  row.operator = "starts_with";
  tree.root.children.push(row);
  return tree;
}

function flip(list: string[], item: string): string[] {
  return list.includes(item) ? list.filter((known) => known !== item) : [...list, item];
}

// One row's items: "Sepatu ×2, Spion Beat ×1".
export function itemsText(row: api.SearchRow): string {
  return row.items.map((item) => texts.search.item(item.name, item.quantity)).join(", ");
}
