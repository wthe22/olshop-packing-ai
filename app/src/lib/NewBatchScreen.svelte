<script lang="ts">
  // New batch (08 › *2. New batch*, *Reading*, *4. Messages*): choose the label PDFs, read them
  // with a progress bar and Cancel, and open the Plan screen — or show a stop message.
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import * as api from "../lib/api";
  import { texts } from "../texts";

  let {
    batchNumber,
    onPlan,
    onCancel,
  }: {
    batchNumber: number;
    onPlan: (result: api.ReadResult) => void;
    onCancel: () => void;
  } = $props();

  let files = $state<api.FileCheck[]>([]);
  let busy = $state(false);
  let reading = $state(false);
  let progress = $state<{ page: number; pages: number } | null>(null);
  let stop = $state("");
  let error = $state("");
  // Set while *Cancel* is pressed: the read then returns to the file list, not to a stop message.
  let cancelling = false;

  const totalPages = $derived(files.reduce((sum, file) => sum + file.pages, 0));
  const sorted = $derived([...files].sort((a, b) => a.name.localeCompare(b.name)));

  // The file a page falls into, from the per-file page counts (the read order is file-name
  // order). `page` counts from 1.
  const readingFile = $derived.by(() => {
    if (!progress) return { file: 1, files: sorted.length };
    let done = 0;
    for (let i = 0; i < sorted.length; i += 1) {
      done += sorted[i].pages;
      if (progress.page <= done) return { file: i + 1, files: sorted.length };
    }
    return { file: sorted.length, files: sorted.length };
  });

  const isPdf = (path: string) => path.toLowerCase().endsWith(".pdf");

  async function addPaths(paths: string[]) {
    const fresh = paths.filter(isPdf).filter(
      (path) => !files.some((file) => file.path.toLowerCase() === path.toLowerCase()),
    );
    if (fresh.length === 0) return;
    busy = true;
    error = "";
    try {
      const checks = await api.checkFiles(fresh);
      const seen = new Set(files.map((file) => file.path.toLowerCase()));
      for (const check of checks) {
        if (!seen.has(check.path.toLowerCase())) {
          files = [...files, check];
          seen.add(check.path.toLowerCase());
        }
      }
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }

  async function pickFiles() {
    const picked = await open({
      multiple: true,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!picked) return;
    await addPaths(typeof picked === "string" ? [picked] : picked);
  }

  function remove(path: string) {
    files = files.filter((file) => file.path !== path);
  }

  async function read() {
    if (sorted.length === 0) return;
    reading = true;
    cancelling = false;
    progress = null;
    stop = "";
    error = "";
    const unlisten = await listen<api.ReadProgress>("read-progress", (event) => {
      progress = event.payload;
    });
    try {
      const result = await api.readLabels(sorted.map((file) => file.path));
      onPlan(result);
    } catch (e) {
      // *Cancel* returns to the file list; any other stop shows its message with Back.
      if (!cancelling) stop = String(e);
    }
    unlisten();
    reading = false;
    progress = null;
  }

  async function cancel() {
    if (reading) {
      cancelling = true;
      await api.cancelRead();
      return;
    }
    onCancel();
  }

  onMount(() => {
    // Dropping PDFs on the window does the same as *Add files…* (08 › *2. New batch*).
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "drop") {
          addPaths(event.payload.paths);
        }
      })
      .then((fn) => (unlisten = fn));
    return () => unlisten?.();
  });
</script>

<main>
  <div class="bar">
    <h2>{texts.newBatch.title(batchNumber)}</h2>
    <span class="grow"></span>
    <button onclick={cancel}>{texts.newBatch.cancel}</button>
  </div>

  {#if stop}
    <div class="stop">
      <p>{stop}</p>
      <button onclick={() => (stop = "")}>{texts.newBatch.back}</button>
    </div>
  {:else if reading}
    <div class="reading">
      <p class="muted">
        {texts.newBatch.reading(
          progress?.page ?? 0,
          progress?.pages ?? totalPages,
          readingFile.file,
          readingFile.files,
        )}
      </p>
      <progress max={progress?.pages ?? totalPages} value={progress?.page ?? 0}></progress>
      <button onclick={cancel}>{texts.newBatch.cancel}</button>
    </div>
  {:else}
    <div class="drop">
      <span>{texts.newBatch.dropHint}</span>
      <button disabled={busy} onclick={pickFiles}>{texts.newBatch.addFiles}</button>
    </div>

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <ul class="files">
      {#each sorted as file (file.path)}
        <li>
          <span class="name">{file.name}</span>
          <span class="pages">{texts.newBatch.pages(file.pages)}</span>
          <span class="time">{file.time}</span>
          {#if file.used_in_batch !== null}
            <span class="used">{texts.newBatch.usedInBatch(file.used_in_batch)}</span>
          {/if}
          <button
            class="remove"
            title={texts.newBatch.removeTitle}
            onclick={() => remove(file.path)}>{texts.newBatch.remove}</button
          >
        </li>
      {/each}
    </ul>

    <div class="foot">
      <span class="selected">
        {#if sorted.length > 0}
          {texts.newBatch.selected(sorted.length, totalPages)}
        {/if}
      </span>
      <button class="primary" disabled={sorted.length === 0} onclick={read}>
        {texts.newBatch.read}
      </button>
    </div>
  {/if}
</main>

<style>
  main {
    padding: 1rem 1.5rem;
  }

  .bar {
    display: flex;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  .grow {
    flex: 1;
  }

  .drop {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 1.5rem;
    margin-bottom: 1rem;
    border: 2px dashed #b8b8b8;
    border-radius: 8px;
    color: #4a4a4a;
  }

  .files {
    list-style: none;
    margin: 0 0 1rem;
    padding: 0;
  }

  .files li {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0.25rem;
    border-bottom: 1px solid #ececec;
  }

  .name {
    flex: 1;
    overflow-wrap: anywhere;
  }

  .pages {
    min-width: 5rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .time {
    min-width: 3.5rem;
    text-align: right;
    color: #6a6a6a;
    font-variant-numeric: tabular-nums;
  }

  .used {
    color: #c07a00;
    min-width: 8rem;
  }

  .remove {
    padding: 0.1rem 0.5rem;
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding-top: 0.5rem;
  }

  .selected {
    flex: 1;
    font-weight: 600;
  }

  .reading {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem 0;
  }

  .reading progress {
    width: 100%;
    height: 1.1rem;
  }

  .stop {
    padding: 1rem;
    border: 1px solid #d98a8a;
    border-radius: 6px;
    background: #fdecec;
    color: #8a1f1f;
  }

  .stop p {
    margin: 0 0 0.75rem;
    white-space: pre-wrap;
  }

  .muted {
    margin: 0;
    color: #4a4a4a;
  }

  @media (prefers-color-scheme: dark) {
    .drop,
    .muted {
      color: #b0b0b0;
      border-color: #555555;
    }

    .files li {
      border-color: #3a3a3a;
    }

    .time {
      color: #b0b0b0;
    }

    .stop {
      background: #3a2020;
      border-color: #8a4a4a;
      color: #f0b8b8;
    }
  }
</style>
