// Read-only mode (start.md, *Read-only*): chosen at start instead of the update; the screens
// offer nothing that writes, and the back end refuses a write.

class Access {
  readOnly = $state(false);
  // The open date was made read only by the owner: a file of it was changed outside the program.
  date = $state(false);

  // Nothing of the open date is written.
  get dateLocked(): boolean {
    return this.readOnly || this.date;
  }
}

export const access = new Access();
