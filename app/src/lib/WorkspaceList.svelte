<script lang="ts">
  // Settings › Workspaces: one row per workspace with its dates, **Rename…** (the name field in
  // place, a refused name told beside it) and **Delete…** (asks first; the folder goes to the
  // Recycle Bin). What the commands do is `workspaceSettings.svelte.ts`.
  import { texts } from "../texts";
  import type { WorkspaceSettings } from "./workspaceSettings.svelte";

  let { workspaces }: { workspaces: WorkspaceSettings } = $props();

  const words = texts.settings;

  // The workspace whose name is being changed, the name typed and why it was refused.
  let renaming = $state<string | null>(null);
  let name = $state("");
  let refused = $state("");

  function startRename(current: string) {
    renaming = current;
    name = current;
    refused = "";
  }

  async function rename() {
    if (renaming === null) return;
    const wanted = name.trim();
    if (wanted === renaming) {
      renaming = null;
      return;
    }
    refused = (await workspaces.rename(renaming, wanted)) ?? "";
    if (!refused) renaming = null;
  }
</script>

{#if workspaces.error}
  <p class="error">{workspaces.error}</p>
{/if}
<div class="group">
  {#each workspaces.list as row (row.name)}
    <div class="row">
      {#if renaming === row.name}
        <form
          class="naming"
          onsubmit={(event) => {
            event.preventDefault();
            if (name.trim()) rename();
          }}
        >
          <input
            bind:value={name}
            aria-label={words.renameLabel(row.name)}
            maxlength="40"
            {@attach (input) => {
              input.focus();
              input.select();
            }}
            onkeydown={(event) => {
              if (event.key === "Escape") {
                event.stopPropagation();
                renaming = null;
              }
            }}
          />
          <button type="button" onclick={() => (renaming = null)}>{words.cancel}</button>
          <button class="primary" type="submit" disabled={name.trim() === ""}>{words.save}</button>
        </form>
        {#if refused}<p class="refused">{refused}</p>{/if}
      {:else}
        <div class="what">
          <span class="name">
            {row.name}
            {#if row.open}<span class="badge">{words.openNow}</span>{/if}
          </span>
          <span class="hint">{words.dates(row.dates)}</span>
        </div>
        <div class="controls">
          <button onclick={() => startRename(row.name)}>{words.rename}</button>
          <button class="danger" onclick={() => workspaces.remove(row)}>{words.delete}</button>
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .group {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem 1rem;
    min-height: 3.4rem;
    padding: 0.6rem 1rem;
  }

  .row + .row {
    border-top: 1px solid var(--border-soft);
  }

  .what {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .name {
    font-weight: 600;
  }

  .hint {
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .badge {
    margin-left: 0.4rem;
    padding: 0 0.5rem;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--link);
    font-size: 0.78rem;
    font-weight: 600;
  }

  .controls {
    display: flex;
    gap: 0.5rem;
    flex: none;
  }

  button.danger:hover:not(:disabled) {
    border-color: var(--danger-border);
    background: var(--danger-bg);
    color: var(--danger);
  }

  .naming {
    flex: 1;
    display: flex;
    gap: 0.5rem;
  }

  .naming input {
    flex: 1;
    max-width: 24rem;
  }

  .refused {
    flex-basis: 100%;
    margin: 0;
    color: var(--danger);
    font-size: 0.88rem;
  }

  .error {
    margin: 0 0 0.6rem;
  }
</style>
