<script lang="ts">
  // New batch (08 › *2. New batch*, *Reading*, *4. Messages*): choose the label PDFs — the shell
  // keeps the list, so the Plan screen's *Back* shows it again with the files still there — read
  // them with a progress bar and Cancel, and open the Plan screen; the top *Cancel* discards the
  // draft. Or show a stop message.
  //
  // The same screen is the amend's selection page (08 › *The amend's selection page*), reached
  // from the amend's Plan screen's *Back*: titled *Redo batch `<b>`*, with the current selection
  // listed — the batch's own `download/` files in their `state.json` order at first, then what the
  // owner chose — and *Read labels* reading that selection as an amend (`amend`).
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import * as api from "../lib/api";
  import { preselectSet } from "../lib/sets";
  import { LogoPrompt, needsLogoPrompt } from "../lib/logoPrompt";
  import LogoPromptDialog from "./LogoPrompt.svelte";
  import { texts } from "../texts";

  let {
    batchNumber,
    amend,
    amendSet,
    files,
    onFiles,
    onPlan,
    onCancel,
  }: {
    batchNumber: number;
    /// The batch being redone on an amend's selection page (08 › *The amend's selection page*),
    /// or `null` for a new batch's file list. Decides the title, the order the files are listed
    /// and read in, and that *Read labels* reads an amend.
    amend: number | null;
    /// The batch's own picks set on the amend's selection page (04 › *Sets*), shown fixed — an
    /// amend reads that set again, whatever set was used last; `null` for a new batch's file
    /// list (the *Set* control is shown there instead).
    amendSet: string | null;
    /// The chosen files. The shell keeps them, so *Back* on the Plan screen shows this list again
    /// with the files still there (08 › *3. Plan*).
    files: api.FileCheck[];
    onFiles: (files: api.FileCheck[]) => void;
    onPlan: (result: api.ReadResult) => void;
    onCancel: () => void;
  } = $props();

  let busy = $state(false);
  let reading = $state(false);
  let progress = $state<{ page: number; pages: number } | null>(null);
  let stop = $state("");
  let error = $state("");
  // Set while *Cancel* is pressed: the read then returns to the file list, not to a stop message.
  let cancelling = false;

  // The *Set* control (08 › *2. New batch* → *The set*): the sets of the data folder, with the
  // set used last preselected, so a shop that always uses one set never touches it. The chosen
  // set is passed to `read_labels` and remembered as the last used there.
  let sets = $state<api.SetView[]>([]);
  let chosenSet = $state(api.DEFAULT_SET);

  // The logo prompt (08 › *The logo prompt*): after a read, before the Plan screen, each unseen
  // logo is asked about once. `asking` is the logo shown now; the flow itself is in logoPrompt.ts.
  let prompt = $state<LogoPrompt | null>(null);
  let asking = $state<api.LogoView | null>(null);
  let couriers = $state<string[]>([]);
  let answering = $state(false);

  const totalPages = $derived(files.reduce((sum, file) => sum + file.pages, 0));
  // A new batch lists (and reads) its files by name = download time and part number; an amend
  // keeps the list's order, which is the batch's `download/` copy in the `state.json` order
  // (08 › *2. New batch*).
  const listed = $derived(
    amend === null ? [...files].sort((a, b) => a.name.localeCompare(b.name)) : [...files],
  );

  // The file a page falls into, from the per-file page counts (the read order is the list's).
  // `page` counts from 1.
  const readingFile = $derived.by(() => {
    if (!progress) return { file: 1, files: listed.length };
    let done = 0;
    for (let i = 0; i < listed.length; i += 1) {
      done += listed[i].pages;
      if (progress.page <= done) return { file: i + 1, files: listed.length };
    }
    return { file: listed.length, files: listed.length };
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
      const added = checks.filter((check) => {
        if (seen.has(check.path.toLowerCase())) return false;
        seen.add(check.path.toLowerCase());
        return true;
      });
      if (added.length > 0) onFiles([...files, ...added]);
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
    onFiles(files.filter((file) => file.path !== path));
  }

  async function read() {
    if (listed.length === 0) return;
    reading = true;
    cancelling = false;
    progress = null;
    stop = "";
    error = "";
    const unlisten = await listen<api.ReadProgress>("read-progress", (event) => {
      progress = event.payload;
    });
    let result: api.ReadResult | null = null;
    try {
      // The batch's picks come from the set chosen above (the amend's selection page reads the
      // batch's own set, so its `set` is ignored by the back end; 04 › *Sets*).
      result = await api.readLabels(
        listed.map((file) => file.path),
        chosenSet,
        amend,
      );
    } catch (e) {
      // *Cancel* returns to the file list; any other stop shows its message with Back.
      if (!cancelling) stop = String(e);
    }
    unlisten();
    reading = false;
    progress = null;
    if (result) begin(result);
  }

  // A read ended: ask about the unseen logos first, or open the Plan screen at once.
  function begin(result: api.ReadResult) {
    if (!needsLogoPrompt(result)) {
      onPlan(result);
      return;
    }
    prompt = new LogoPrompt(result, api.answerLogo);
    couriers = result.couriers;
    asking = prompt.current;
  }

  // One answer (a courier name, *Not a courier*, or *I don't know*/Esc, which both send `skip`).
  async function answer(reply: api.LogoAnswer) {
    if (!prompt || answering) return;
    answering = true;
    error = "";
    try {
      await prompt.answer(reply);
    } catch (e) {
      error = String(e);
    }
    const next = prompt.current;
    asking = next;
    if (next === null) {
      // The list is done: the Plan screen opens with the latest result.
      const done = prompt.latest;
      prompt = null;
      onPlan(done);
    }
    answering = false;
  }

  // *Skip all*: stop the prompts for the rest of this batch; nothing is sent, the last result
  // opens the Plan screen (08 › *The logo prompt*).
  function skipAll() {
    if (!prompt) return;
    prompt.skipAll();
    const done = prompt.latest;
    prompt = null;
    asking = null;
    onPlan(done);
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
    // The *Set* control's sets, with the set used last preselected (08 › *2. New batch* →
    // *The set*). Only a new batch's file list has the control; the amend's selection page shows
    // the batch's own set. A data folder whose sets cannot be listed keeps the set `default`.
    if (amend === null) {
      api
        .setsList()
        .then((list) => {
          sets = list.sets;
          chosenSet = preselectSet(list.sets, list.last_used);
        })
        .catch((e) => (error = String(e)));
    }
    // Dropping PDFs on the window does the same as *Add files…* (08 › *2. New batch*).
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "drop") {
          addPaths(event.payload.paths);
        }
      })
      .then((fn) => (unlisten = fn));
    // Esc skips only the logo shown: the next unseen logo still asks (08 › *The logo prompt*).
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !asking) return;
      event.preventDefault();
      answer({ kind: "skip" });
    };
    window.addEventListener("keydown", onKey);
    return () => {
      unlisten?.();
      window.removeEventListener("keydown", onKey);
    };
  });
</script>

<main>
  <div class="bar">
    <h2>{amend === null ? texts.newBatch.title(batchNumber) : texts.newBatch.amendTitle(amend)}</h2>
    {#if amend === null}
      <!-- 08 › *2. New batch* → *The set*: chosen before *Read labels*. -->
      <label class="set">
        {texts.newBatch.set}:
        <select bind:value={chosenSet}>
          {#each sets as option (option.name)}
            <option value={option.name}>{option.name}</option>
          {/each}
        </select>
      </label>
    {:else if amendSet !== null}
      <!-- The amend's selection page: the batch's own set, shown and not chosen. -->
      <span class="set-fixed">
        {texts.newBatch.set}: {texts.newBatch.setFixed(amendSet)}
      </span>
    {/if}
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
      {#each listed as file (file.path)}
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
        {#if listed.length > 0}
          {texts.newBatch.selected(listed.length, totalPages)}
        {/if}
      </span>
      <button class="primary" disabled={listed.length === 0} onclick={read}>
        {texts.newBatch.read}
      </button>
    </div>
  {/if}

  {#if asking}
    <LogoPromptDialog
      logo={asking}
      {couriers}
      busy={answering}
      onAnswer={answer}
      onSkipAll={skipAll}
    />
  {/if}

  {#if error && asking}
    <p class="answer-error">{error}</p>
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

  /* 08 › *2. New batch* → *The set*: the Set control before *Read labels*, and the amend's
     selection page's fixed set line. */
  .set,
  .set-fixed {
    margin-left: 1rem;
    color: #4a4a4a;
  }

  .set select {
    max-width: 14rem;
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

  /* An answer to the logo prompt that failed: shown over the prompt's backdrop. */
  .answer-error {
    position: fixed;
    left: 50%;
    bottom: 2rem;
    transform: translateX(-50%);
    margin: 0;
    padding: 0.5rem 1rem;
    background: #fdecec;
    border: 1px solid #d98a8a;
    border-radius: 6px;
    color: #8a1f1f;
    z-index: 12;
  }

  @media (prefers-color-scheme: dark) {
    .drop,
    .muted,
    .set,
    .set-fixed {
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

    .answer-error {
      background: #3a2020;
      border-color: #8a4a4a;
      color: #f0b8b8;
    }
  }
</style>
