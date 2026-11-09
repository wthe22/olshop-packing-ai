// Other copies of the app, on this PC or another, may use the same data folder. Each time the
// window gets the focus the back end looks for what they wrote, and the shell reads again what it
// shows: the Day screen, the panel, or the workspace opened instead of one another copy renamed or
// deleted.

import { dataChanges } from "./api/changes";
import { texts } from "../texts";
import type { Shell } from "./shell.svelte";

// One look, and what the shell reads again.
export const lookForChanges = async (shell: Shell): Promise<void> => {
  const found = await dataChanges();
  if (found.gone) {
    await shell.cancelBatch();
    shell.error = texts.day.workspaceGone(found.gone.workspace);
    await shell.workspaceDeleted(found.gone.now_open);
    return;
  }
  if (found.day) await shell.refreshOverview();
  if (found.panel) await shell.refreshPanel();
};

// Look each time the window gets the focus while a workspace is open, until the returned function
// is called. A look that fails (the folder out of reach for a moment) is tried again at the next
// focus; a focus during a look does not start a second.
export const followChanges = (shell: Shell): (() => void) => {
  let looking = false;
  const look = async () => {
    if (looking || shell.workspace === null) return;
    looking = true;
    try {
      await lookForChanges(shell);
    } catch {
      // Nothing to do: the next focus looks again.
    } finally {
      looking = false;
    }
  };
  window.addEventListener("focus", look);
  return () => window.removeEventListener("focus", look);
};
