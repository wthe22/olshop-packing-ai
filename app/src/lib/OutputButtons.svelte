<script lang="ts">
  // The buttons of one output file on the Day screen: **Print** (not on a summary PDF) and **Open**,
  // or **Restore** for a file found changed, with the open-time check's warning before them.
  import { texts } from "../texts";
  import type { FileProblem } from "./api/day";
  import type { SentMark } from "./dayOutputs.svelte";

  let {
    file,
    problem,
    mark,
    printable,
    onRestore,
    onPrint,
    onOpenOutput,
  }: {
    file: string;
    problem: FileProblem | null;
    // The group a **Print** marks *Sent*; `null` marks nothing.
    mark: SentMark | null;
    printable: boolean;
    onRestore: (file: string) => void;
    onPrint: (file: string, mark: SentMark | null) => void;
    onOpenOutput: (file: string) => void;
  } = $props();
</script>

{#if problem}
  <span class="problem">{texts.day.problem(problem)}</span>
{/if}
{#if problem === "changed"}
  <button onclick={() => onRestore(file)}>{texts.day.restore}</button>
{:else}
  {#if printable}
    <button title={texts.day.printTitle(file)} onclick={() => onPrint(file, mark)}
      >{texts.day.print}</button
    >
  {/if}
  <button onclick={() => onOpenOutput(file)}>{texts.day.open}</button>
{/if}
