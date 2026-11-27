// Which months of a list of dates are open (panel.ts): a month the owner folded or unfolded stays
// so; the others follow `startsOpen`.

export class Folds {
  private toggled = $state<Record<string, boolean>>({});

  isOpen = (month: { key: string; startsOpen: boolean }) => this.toggled[month.key] ?? month.startsOpen;

  toggle = (month: { key: string; startsOpen: boolean }) => {
    this.toggled[month.key] = !this.isOpen(month);
  };
}
