<script lang="ts">
  // One group PDF of a batch card: its code, name and counts, the *Sent* mark, the **Printed** tick
  // and its buttons.
  import { texts } from "../texts";
  import type { BatchView, GroupPdfView } from "./api/day";
  import type { SentMark } from "./dayOutputs.svelte";
  import Icon from "./Icon.svelte";
  import Count from "./Count.svelte";
  import OutputButtons from "./OutputButtons.svelte";

  let {
    batch,
    pdf,
    onRestore,
    onPrint,
    onOpenOutput,
    onUnsend,
    onSetPrinted,
  }: {
    batch: BatchView;
    pdf: GroupPdfView;
    onRestore: (file: string) => void;
    onPrint: (file: string, mark: SentMark | null) => void;
    onOpenOutput: (file: string) => void;
    // The *Sent* mark's ✕.
    onUnsend: (batch: BatchView, pdf: GroupPdfView) => void;
    // The **Printed** tick: written to the date's journal the moment it is clicked.
    onSetPrinted: (batch: BatchView, pdf: GroupPdfView, printed: boolean) => void;
  } = $props();
</script>

<div class="row">
  <span class="code">{batch.batch}-{pdf.group}</span>
  <span class="name">{pdf.name}</span>
  <Count n={pdf.orders} word={texts.day.orderWord(pdf.orders)} />
  <Count n={pdf.contents} word={texts.day.contentWord(pdf.contents)} />
  <span class="sent">
    {#if pdf.sent && !pdf.printed}
      {texts.day.sent(pdf.sent)}
      <button
        class="icon"
        title={texts.day.unsend(pdf.file)}
        aria-label={texts.day.unsend(pdf.file)}
        onclick={() => onUnsend(batch, pdf)}><Icon name="close" size={16} /></button
      >
    {/if}
  </span>
  <label class="printed">
    <input
      type="checkbox"
      checked={pdf.printed}
      aria-label={texts.day.printedLabel(pdf.file)}
      title={texts.day.printedLabel(pdf.file)}
      onchange={(event) => onSetPrinted(batch, pdf, event.currentTarget.checked)}
    />
  </label>
  <OutputButtons
    file={pdf.file}
    problem={pdf.problem}
    mark={{ batch: batch.batch, group: pdf.group }}
    printable={true}
    {onRestore}
    {onPrint}
    {onOpenOutput}
  />
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0.75rem;
    border-bottom: 1px solid var(--border-soft);
    /* The row's *Open* button and the **Printed** tick share this height, so the tick is as large
       as the button beside it. The value is the button's own box: its text line,
       its padding and its border. */
    --control-height: calc(1.45rem + 0.7rem + 2px);
  }

  .row :global(button),
  .printed input {
    height: var(--control-height);
  }

  /* The batch and group code (`2-SN`) on one line, codes of several letters included. */
  .code {
    min-width: 3.5rem;
    padding: 0.05rem 0.5rem;
    border-radius: var(--radius);
    background: var(--chip);
    color: var(--chip-text);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    text-align: center;
    white-space: nowrap;
  }

  .name {
    flex: 1;
    font-weight: 600;
  }

  /* The **Printed** tick beside *Open*: a note only. It carries no word — the tick
     alone says the saved PDF is printed — and it is as large as the button beside it. */
  .printed {
    display: flex;
    align-items: center;
  }

  /* *Sent 09:14* and its ✕ before the tick: the preview opened, the labels not yet ticked. Every
     group row keeps its room, so the counts of a card stay under each other. */
  .sent {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    min-width: 6.5rem;
    gap: 0.2rem;
    color: var(--text-muted);
    font-size: 0.9rem;
    font-variant-numeric: tabular-nums;
  }

  .printed input {
    width: var(--control-height);
    border-radius: var(--radius);
  }

  .printed input:checked {
    border-color: var(--ok-solid);
    background-color: var(--ok-solid);
  }
</style>
