// The Accounts screen's state: the accounts and the one shown, its shop fields as edited, its dates,
// and the commands of the screen (new, rename, delete, save). A refusal of a name or of the shops
// comes back as text for the line under its field; any other failure is `error`.

import * as api from "./api/accounts";
import { confirmBox } from "./confirm.svelte";
import { listedElsewhere, shopProblems, shopsChanged } from "./shops";
import { texts } from "../texts";

// How soon the dates are asked for again while the search index is still being built.
const RELOAD_MS = 500;

export class AccountsState {
  accounts = $state<api.Account[]>([]);
  // The account shown; `null` with no account.
  current = $state<string | null>(null);
  // The shop fields as edited.
  names = $state<string[]>([]);
  dates = $state<api.AccountDates | null>(null);
  // A failed command that is not a refused name.
  error = $state("");
  // Why the shops were refused on Save, shown under the list.
  refused = $state("");
  saving = $state(false);
  private timer: ReturnType<typeof setTimeout> | undefined;

  get account(): api.Account | null {
    return this.accounts.find((account) => account.name === this.current) ?? null;
  }

  get problems() {
    return shopProblems(this.names, listedElsewhere(this.accounts, this.current));
  }

  get dirty(): boolean {
    return this.account !== null && shopsChanged(this.account.shops, this.names);
  }

  // Why Save is off, for its tooltip; empty when the shops can be saved.
  get blocked(): string {
    const problem = this.problems.find((found) => found !== null);
    return problem ? texts.accounts.problem(problem) : "";
  }

  load = async () => {
    try {
      this.accounts = await api.accountsList();
      await this.show(this.accounts[0]?.name ?? null);
    } catch (e) {
      this.error = String(e);
    }
  };

  stop = () => clearTimeout(this.timer);

  private show = async (name: string | null) => {
    this.current = name;
    this.refused = "";
    this.names = [...(this.account?.shops ?? [])];
    await this.loadDates();
  };

  private loadDates = async () => {
    clearTimeout(this.timer);
    const shown = this.current;
    if (shown === null) {
      this.dates = null;
      return;
    }
    try {
      const found = await api.accountDates(shown);
      if (shown !== this.current) return;
      this.dates = found;
      if (found.loading) this.timer = setTimeout(this.loadDates, RELOAD_MS);
    } catch (e) {
      this.error = String(e);
    }
  };

  // Unsaved shops are thrown away only when the owner agrees.
  private keepsEdits = async (): Promise<boolean> => {
    if (!this.dirty || this.current === null) return true;
    return confirmBox.ask(texts.accounts.askDiscard(this.current));
  };

  select = async (name: string) => {
    if (name === this.current || !(await this.keepsEdits())) return;
    await this.show(name);
  };

  // **+ New account** opens its name field; false when the owner kept the edits.
  startNew = (): Promise<boolean> => this.keepsEdits();

  // Resolves to why the name is refused, or `null` when the account was made and is shown.
  create = async (name: string): Promise<string | null> => {
    try {
      this.accounts = await api.accountNew(name);
    } catch (e) {
      return String(e);
    }
    await this.show(name);
    return null;
  };

  // Resolves to why the name is refused, or `null` when it was renamed (the edited shops stay).
  rename = async (to: string): Promise<string | null> => {
    if (this.current === null) return null;
    try {
      this.accounts = await api.accountRename(this.current, to);
    } catch (e) {
      return String(e);
    }
    this.current = to;
    await this.loadDates();
    return null;
  };

  remove = async () => {
    if (this.current === null || !(await confirmBox.ask(texts.accounts.askDelete(this.current)))) {
      return;
    }
    try {
      this.accounts = await api.accountDelete(this.current);
    } catch (e) {
      this.error = String(e);
      return;
    }
    await this.show(this.accounts[0]?.name ?? null);
  };

  addShop = () => {
    this.names = [...this.names, ""];
  };

  removeShop = (index: number) => {
    this.names = this.names.filter((_, at) => at !== index);
  };

  cancelShops = () => {
    this.names = [...(this.account?.shops ?? [])];
    this.refused = "";
  };

  saveShops = async () => {
    if (this.current === null || this.blocked) return;
    this.saving = true;
    try {
      this.accounts = await api.accountSetShops(
        this.current,
        this.names.map((name) => name.trim()),
      );
      this.refused = "";
      this.names = [...(this.account?.shops ?? [])];
    } catch (e) {
      this.refused = String(e);
    }
    this.saving = false;
  };
}
