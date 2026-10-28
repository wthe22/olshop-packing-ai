// Settings › Workspaces: the list with each workspace's dates, renaming and deleting one. Renaming
// or deleting the open workspace asks about an open draft first; the shell then follows: the same
// date under the new name, or today's date of the workspace the back end opened (none left: the
// first workspace's name is asked).

import * as api from "./api/workspaces";
import type { Shell } from "./shell.svelte";
import { confirmBox } from "./confirm.svelte";
import { texts } from "../texts";

export class WorkspaceSettings {
  list = $state<api.WorkspaceView[]>([]);
  // A failed command; a refused name is shown beside its field instead.
  error = $state("");

  private readonly shell: Shell;

  constructor(shell: Shell) {
    this.shell = shell;
  }

  load = async () => {
    try {
      this.list = await api.workspacesOverview();
      this.error = "";
    } catch (e) {
      this.error = String(e);
    }
  };

  // Resolves to why the name is refused, or `null` when it was renamed or nothing was done.
  rename = async (name: string, newName: string): Promise<string | null> => {
    const open = name === this.shell.workspace;
    if (open && !(await this.shell.leaveBatch())) return null;
    try {
      await api.workspaceRename(name, newName);
    } catch (e) {
      return String(e);
    }
    if (open) await this.shell.workspaceRenamed(newName);
    else await this.shell.refreshPanel();
    await this.load();
    return null;
  };

  remove = async (row: api.WorkspaceView) => {
    if (!(await confirmBox.ask(texts.settings.askDelete(row.name, row.dates)))) return;
    if (row.open && !(await this.shell.leaveBatch())) return;
    try {
      const nowOpen = await api.workspaceDelete(row.name);
      if (row.open) await this.shell.workspaceDeleted(nowOpen);
      else await this.shell.refreshPanel();
    } catch (e) {
      this.error = String(e);
      return;
    }
    await this.load();
  };
}
