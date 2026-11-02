// Other copies of the app, on this PC or another, may use the same data folder. Every few seconds
// the back end looks for what they wrote, and the shell reads again what it shows: the Day screen,
// the panel, or the workspace opened instead of one another copy renamed or deleted.

import { dataChanges } from "./api/changes";
import { texts } from "../texts";
import type { Shell } from "./shell.svelte";

const LOOK_EVERY_MS = 3000;

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

// Look every few seconds while a workspace is open, until the returned function is called. A
// look that fails (the folder out of reach for a moment) is tried again at the next.
export const followChanges = (shell: Shell): (() => void) => {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let stopped = false;
  const look = async () => {
    if (shell.workspace !== null) {
      try {
        await lookForChanges(shell);
      } catch {
        // Nothing to do: the next look tries again.
      }
    }
    if (!stopped) timer = setTimeout(look, LOOK_EVERY_MS);
  };
  timer = setTimeout(look, LOOK_EVERY_MS);
  return () => {
    stopped = true;
    clearTimeout(timer);
  };
};
