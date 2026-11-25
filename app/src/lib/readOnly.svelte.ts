// Read-only mode (start.md, *Read-only*): chosen at start instead of the update; the screens
// offer nothing that writes, and the back end refuses a write.

class Access {
  readOnly = $state(false);
}

export const access = new Access();
