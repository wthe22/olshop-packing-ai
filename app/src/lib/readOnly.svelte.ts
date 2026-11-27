// Read-only mode (start.md, *Read-only*): chosen at start instead of the update; the screens
// offer nothing that writes, and the back end refuses a write. An archived date offers nothing that
// needs its PDFs.

class Access {
  readOnly = $state(false);
  // The open date was made read only by the owner: a file of it was changed outside the program.
  date = $state(false);
  // The open date is archived: its PDFs were removed, so nothing that needs one is offered.
  archived = $state(false);

  // Nothing of the open date is written.
  get dateLocked(): boolean {
    return this.readOnly || this.date;
  }
}

export const access = new Access();
