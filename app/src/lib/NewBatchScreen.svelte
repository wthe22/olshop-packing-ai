<script lang="ts">
  // New batch: choose the label PDFs — the shell
  // keeps the list, so the Plan screen's *Back* shows it again with the files still there — read
  // them with a progress bar and Cancel, and open the Plan screen; the top *Cancel* discards the
  // draft. Or show a stop message.
  //
  // The same screen is the amend's selection page, reached
  // from the amend's Plan screen's *Back*: titled *Redo batch `<b>`*, with the current selection
  // listed — the batch's own `download/` files in their `state.json` order at first, then what the
  // owner chose — and *Read labels* reading that selection as an amend (`amend`).
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import * as batch from "../lib/api/batch";
  import { sortRules } from "./api/rules";
  import { preselectRule } from "./sortRules";
  import { LogoPrompt, needsLogoPrompt } from "../lib/logoPrompt";
  import LogoPromptDialog from "./LogoPrompt.svelte";
  import BatchFiles from "./BatchFiles.svelte";
  import { fileChips } from "./repeats";
  import SortRuleControl from "./SortRuleControl.svelte";
  import { texts } from "../texts";

  let {
    batchNumber,
    amend,
    amendRule,
    files,
    onFiles,
    repeats,
    onPlan,
    onReviewRepeats,
    onCancel,
  }: {
    batchNumber: number;
    // The batch being redone on an amend's selection page,
    // or `null` for a new batch's file list. Decides the title, the order the files are listed
    // and read in, and that *Read labels* reads an amend.
    amend: number | null;
    // The batch's own sort rule on the amend's selection page, shown fixed; `null` for a new
    // batch's file list, where the rule is chosen.
    amendRule: string | null;
    // The chosen files. The shell keeps them, so *Back* on the Plan screen shows this list again
    // with the files still there.
    files: batch.FileCheck[];
    onFiles: (files: batch.FileCheck[]) => void;
    // The repeats of the read still open: each file that holds one gets its group chip.
    repeats: batch.RepeatsView | null;
    onPlan: (result: batch.ReadResult) => void;
    onReviewRepeats: () => void;
    onCancel: () => void;
  } = $props();

  const chips = $derived(repeats ? fileChips(repeats) : new Map());

  let busy = $state(false);
  let reading = $state(false);
  let progress = $state<{ page: number; pages: number } | null>(null);
  let stop = $state("");
  let error = $state("");
  // The files of the last *Add files…* or drop already listed by their bytes.
  let note = $state("");
  // Set while *Cancel* is pressed: the read then returns to the file list, not to a stop message.
  let cancelling = false;

  // The sort rules of `packing-workflows.toml`, the newest batch's preselected, so an owner who
  // always uses one rule never touches the choice.
  let rules = $state<string[]>([]);
  let chosenRule = $state("");

  // The logo prompt: after a read, before the Plan screen, each unseen
  // logo is asked about once. `asking` is the logo shown now; the flow itself is in logoPrompt.ts.
  let prompt = $state<LogoPrompt | null>(null);
  let asking = $state<batch.LogoView | null>(null);
  let couriers = $state<string[]>([]);
  let answering = $state(false);

  const totalPages = $derived(files.reduce((sum, file) => sum + file.pages, 0));
  // A new batch lists (and reads) its files by name = download time and part number; an amend
  // keeps the list's order, which is the batch's `download/` copy in the `state.json` order.
  //
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
    note = "";
    try {
      const checks = await batch.checkFiles(fresh, amend);
      // One file by its bytes: the same download chosen again from another folder — on an amend,
      // a file the batch already reads from `input\` — is listed once, and a line names both.
      const listedNames = new Map(files.map((file) => [file.sha256, file.name]));
      const added: batch.FileCheck[] = [];
      const same: string[] = [];
      for (const check of checks) {
        const listed = listedNames.get(check.sha256);
        if (listed === undefined) {
          listedNames.set(check.sha256, check.name);
          added.push(check);
        } else {
          same.push(texts.newBatch.sameFile(check.name, listed));
        }
      }
      note = same.join(" ");
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
    const unlisten = await listen<batch.ReadProgress>("read-progress", (event) => {
      progress = event.payload;
    });
    let result: batch.ReadResult | null = null;
    try {
      result = await batch.readLabels(
        listed.map((file) => file.path),
        chosenRule,
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
  function begin(result: batch.ReadResult) {
    if (!needsLogoPrompt(result)) {
      onPlan(result);
      return;
    }
    prompt = new LogoPrompt(result, batch.answerLogo);
    couriers = result.couriers;
    asking = prompt.current;
  }

  // One answer (a courier name, *Not a courier*, or *I don't know*/Esc, which both send `skip`).
  async function answer(reply: batch.LogoAnswer) {
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
  // opens the Plan screen.
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
      await batch.cancelRead();
      return;
    }
    onCancel();
  }

  onMount(() => {
    // Only a new batch's file list chooses a rule; the amend's selection page shows the batch's.
    if (amend === null) {
      sortRules()
        .then((list) => {
          rules = list.rules;
          chosenRule = preselectRule(list.rules, list.preselected);
        })
        .catch((e) => (error = String(e)));
    }
    // Dropping PDFs on the window does the same as *Add files…*.
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "drop") {
          addPaths(event.payload.paths);
        }
      })
      .then((fn) => (unlisten = fn));
    // Esc skips only the logo shown: the next unseen logo still asks.
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
    <SortRuleControl
      {amend}
      {amendRule}
      {rules}
      chosen={chosenRule}
      onChosen={(value) => (chosenRule = value)}
    />
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
    {#if repeats}
      <p class="repeats">
        <button onclick={onReviewRepeats}>{texts.repeats.review}</button>
      </p>
    {/if}
    <BatchFiles
      {listed}
      {totalPages}
      {busy}
      {error}
      {note}
      {chips}
      onAdd={pickFiles}
      onRemove={remove}
      onRead={read}
    />
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
    border: 1px solid var(--danger-border);
    border-radius: var(--radius-lg);
    background: var(--danger-bg);
    color: var(--danger);
  }

  .stop p {
    margin: 0 0 0.75rem;
    white-space: pre-wrap;
  }

  .muted {
    margin: 0;
    color: var(--text-muted);
  }

  /* An answer to the logo prompt that failed: shown over the prompt's backdrop. */
  .answer-error {
    position: fixed;
    left: 50%;
    bottom: 2rem;
    transform: translateX(-50%);
    margin: 0;
    padding: 0.5rem 1rem;
    background: var(--danger-bg);
    border: 1px solid var(--danger-border);
    border-radius: 6px;
    color: var(--danger);
    z-index: 12;
  }
</style>
