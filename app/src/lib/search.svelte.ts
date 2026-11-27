// The Search screen's state and commands: the filter boxes turned into filter text by the engine's
// printer, the search on the index over a range of dates or all of them, the ticked rows and their
// reprint. Searching changes nothing; only **Reprint** writes a file, and it asks first when a
// ticked order was printed again before. The markup is `SearchScreen.svelte`.

import * as api from "./api/search";
import { rulesList, treeToFilter, type Courier } from "./api/rules";
import { accountsList } from "./api/accounts";
import { newRow, newTree, toRustTree, type UiTree } from "./conditions";
import { texts } from "../texts";
import { formatDay, formatShortDay } from "../dates";
import { confirmBox } from "./confirm.svelte";
import { presetRange, rangePresets, type DateRange, type RangePreset } from "./searchRange";

// While the index is being built the search is asked again after this many milliseconds.
const LOADING_RETRY_MS = 500;

const key = (row: { day: string; order_id: string }) => `${row.day} ${row.order_id}`;

// The boxes as the owner left them: Search opens with them again while the program runs. The dates
// are not kept; they start from the viewed day each time.
let keptTree: UiTree | null = null;

export class Search {
  tree = $state<UiTree>(keptTree ?? firstTree());
  preset = $state<RangePreset>("viewed");
  // The range's two dates as shown; kept under *All days*, so *Custom range* finds them again.
  range = $state<DateRange>({ from: "", to: "" });
  busy = $state(false);
  loading = $state(false);
  // A filter that does not parse, or a failed command.
  error = $state("");
  // The last search's rows; `null` before the first and after any change, so the rows on screen
  // always match the boxes and the scope shown.
  result = $state<api.SearchResults | null>(null);
  ticked = $state<string[]>([]);
  // The rows whose details are open.
  traced = $state<string[]>([]);
  made = $state<api.ReprintMade | null>(null);
  // The verifier's findings of a reprint that was not kept.
  findings = $state<string[]>([]);
  // For the Courier and Account boxes' lists.
  couriers = $state<Courier[]>([]);
  accounts = $state<string[]>([]);
  // The row a single click just ticked or unticked, so the double click it starts can undo it.
  private clicked: string | null = null;

  // The viewed date and today, which the choices of dates count from; and what lets the Day screen
  // and the panel recount after a reprint.
  private readonly day: () => string;
  private readonly today: () => string;
  private readonly onReprinted: () => void;

  constructor(day: () => string, today: () => string, onReprinted: () => void) {
    this.day = day;
    this.today = today;
    this.onReprinted = onReprinted;
    keptTree = this.tree;
    this.setFirstDates();
  }

  private setFirstDates = () => {
    this.preset = this.presets()[0];
    this.range = { from: this.day(), to: this.day() };
  };

  // The Dates list.
  presets = () => rangePresets(this.day(), this.today());

  loadCouriers = async () => {
    try {
      this.couriers = (await rulesList()).couriers;
      this.accounts = (await accountsList()).map((account) => account.name);
    } catch (e) {
      this.error = String(e);
    }
  };

  invalidate = () => {
    this.error = "";
    this.result = null;
    this.ticked = [];
    this.traced = [];
  };

  setPreset = (preset: RangePreset) => {
    this.preset = preset;
    this.range = presetRange(preset, this.day(), this.today(), this.range) ?? this.range;
    this.invalidate();
  };

  // A date typed or picked: the range is the owner's own from then on.
  setRange = (range: DateRange) => {
    this.preset = "custom";
    this.range = range;
    this.invalidate();
  };

  // **Reset**: the boxes and the dates as Search first opens them.
  reset = () => {
    this.tree = firstTree();
    keptTree = this.tree;
    this.setFirstDates();
    this.made = null;
    this.findings = [];
    this.invalidate();
  };

  // **Search** (or Enter in a box): a new search, the last reprint's line gone.
  run = async () => {
    this.made = null;
    this.findings = [];
    await this.find();
  };

  // The rows for the boxes and scope shown. A search that finds one order ticks it, so a scanned
  // tracking ID is one click from its reprint.
  private find = async () => {
    this.busy = true;
    this.invalidate();
    try {
      const filter = await treeToFilter(toRustTree(this.tree));
      const result = await api.searchOrders(filter, this.preset === "all" ? null : this.range);
      this.loading = result.loading;
      if (result.loading) setTimeout(this.find, LOADING_RETRY_MS);
      else {
        this.result = result;
        if (result.rows.length === 1) this.ticked = [key(result.rows[0])];
      }
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

  // A click on a card opens or closes its details; only its tick ticks it. A drag or a double click
  // selects text instead and leaves the details as they were: `selecting` says the click ended a
  // selection, and a double click's second click undoes its first.
  clickCard = (row: api.SearchRow, clicks: number, selecting: boolean) => {
    if (clicks === 1 && !selecting) {
      this.toggleTrace(row);
      this.clicked = key(row);
    } else if (clicks === 2 && this.clicked === key(row)) {
      this.toggleTrace(row);
      this.clicked = null;
    }
  };

  toggleTrace = (row: api.SearchRow) => {
    this.traced = flip(this.traced, key(row));
  };

  // **Select all**: every row the filter found, or none.
  selectAll = (all: boolean) => {
    this.ticked = all && this.result ? this.result.rows.map(key) : [];
  };

  clear = () => {
    this.ticked = [];
  };

  allTicked = () =>
    this.result !== null && this.result.rows.length > 0 && this.ticked.length === this.result.rows.length;

  // Some rows ticked, not all: the head row's tick shows a dash.
  someTicked = () => this.ticked.length > 0 && !this.allTicked();

  tickedRows = () => this.result?.rows.filter(this.isTicked) ?? [];

  // The ticked orders that were printed again before.
  reprintedBefore = () => this.tickedRows().filter((row) => row.reprints.length > 0).length;

  // **Reprint `<n>` orders**: one PDF of the ticked orders' label pages, written into today. The
  // rows are searched again afterwards, so the orders just printed carry their mark.
  reprint = async () => {
    const rows = this.tickedRows();
    if (rows.length === 0) return;
    const before = this.reprintedBefore();
    if (before > 0) {
      if (!(await confirmBox.ask(texts.search.askAgain(before, rows.length)))) return;
    }
    this.busy = true;
    this.made = null;
    this.findings = [];
    this.error = "";
    try {
      const result = await api.reprint(rows.map((row) => ({ day: row.day, order_id: row.order_id })));
      this.made = result.made;
      this.findings = result.errors;
      if (result.made) {
        this.onReprinted();
        await this.find();
        this.ticked = [];
      }
    } catch (e) {
      this.error = String(e);
    }
    this.busy = false;
  };
}

// One row: the Order ID, the box the owner fills first; *contains*, so part of an ID finds it.
function firstTree(): UiTree {
  const tree = newTree();
  tree.root.children.push(newRow("order_id"));
  return tree;
}

function flip(list: string[], item: string): string[] {
  return list.includes(item) ? list.filter((known) => known !== item) : [...list, item];
}

// One row's items: "Sepatu ×2, Spion Beat ×1".
export function itemsText(row: api.SearchRow): string {
  return row.items.map((item) => texts.search.item(itemName(item), item.quantity)).join(", ");
}

// An item as the row names it: "Spion Beat — honda", or the name alone without a variation.
export function itemName(item: api.SearchItem): string {
  return item.variant ? `${item.name} — ${item.variant}` : item.name;
}

// The Qty Total at the foot of a card's items, when it says more than its one item line: an order of
// several lines, or a total the item lines do not add up to.
export function totalLine(row: api.SearchRow): number | null {
  const total = row.total_quantity;
  if (total === null) return null;
  const sum = row.items.reduce((all, item) => all + item.quantity, 0);
  return row.items.length > 1 || sum !== total ? total : null;
}

// One line of a card's details above its input file and group: its head and its value; `notRead`
// when the order was saved before it was read and the data folder not updated since.
export interface DetailLine {
  head: string;
  text: string;
  notRead: boolean;
}

// The details' lines above the input file and group, each only when the label prints it: the
// receiver's province, city and district; the ship-by date; the weight; the account when the card
// shows a shop of another name.
export function detailLines(row: api.SearchRow): DetailLine[] {
  const words = texts.search;
  const heads = words.details;
  const lines: DetailLine[] = [];
  const add = (head: string, text: string) => lines.push({ head, text, notRead: false });
  const notRead = (head: string) => lines.push({ head, text: words.notRead, notRead: true });
  if (row.area === null) notRead(heads.address);
  else {
    const area = [row.area.province, row.area.city, row.area.district].filter((part) => part).join(", ");
    if (area) add(heads.address, area);
  }
  if (row.ship_by) add(heads.shipBy, formatDay(row.ship_by));
  if (row.weight === null) notRead(heads.weight);
  else if (row.weight !== "not_printed") add(heads.weight, words.grams(row.weight.grams));
  if (row.shop !== null && row.shop !== row.account) add(heads.account, row.account);
  return lines;
}

// The mark of an order printed again before, and its tooltip naming each reprint:
// "Reprinted", "Reprint 1 on Wed, 7 Oct".
export function reprintedMark(row: api.SearchRow): { text: string; title: string } | null {
  if (row.reprints.length === 0) return null;
  return {
    text: texts.search.reprinted(row.reprints.length),
    title: row.reprints.map((mark) => texts.search.reprintOn(mark.number, formatShortDay(mark.day))).join("\n"),
  };
}
