<script lang="ts">
  // Repeated orders: the batch's orders that appear more than once, by groups of files that repeat
  // each other, and the copy of each that stays. Every choice is sent at once and the back end
  // recounts; the cards start on their defaults, so *Continue to the plan* works at once.
  import * as batch from "./api/batch";
  import { sameChoice, withCopy, withGroupChoice } from "./repeats";
  import { texts } from "../texts";

  let {
    read,
    onResult,
    onContinue,
    onBack,
  }: {
    read: batch.ReadResult;
    // The read as it is after a choice.
    onResult: (result: batch.ReadResult) => void;
    onContinue: () => void;
    // *Back*: the file list with the files still listed; nothing is written.
    onBack: () => void;
  } = $props();

  const view = $derived(read.repeats);
  // The groups whose order list is open.
  let opened = $state<number[]>([]);
  let busy = $state(false);
  let error = $state("");

  async function send(choices: batch.RepeatChoices) {
    busy = true;
    error = "";
    try {
      onResult(await batch.resolveRepeats(choices));
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }

  const choose = (group: number, choice: batch.ChoiceView) =>
    view && send(withGroupChoice(view, group, choice));
  const pick = (orderId: string, copy: number) => view && send(withCopy(view, orderId, copy));
  const keeps: batch.ChoiceView[] = [{ kind: "keep_first" }, { kind: "keep_last" }];
</script>

<main>
  <div class="bar">
    <h2>{texts.repeats.title}</h2>
    <span class="grow"></span>
    <button onclick={onBack}>{texts.repeats.back}</button>
  </div>

  {#if view}
    <p class="summary">
      {texts.repeats.summary(view.orders, view.extra_copies, view.extra_pages, view.files, view.all_files)}
    </p>
    {#if error}<p class="error">{error}</p>{/if}

    {#each view.groups as group (group.number)}
      <section class="group">
        <h3>{texts.repeats.group(group.number, group.files.map((file) => file.file), group.orders.length)}</h3>
        <table>
          <thead>
            <tr>
              <th>{texts.repeats.file}</th>
              <th>{texts.repeats.pages}</th>
              <th>{texts.repeats.copies}</th>
              <th>{texts.repeats.kept}</th>
              <th>{texts.repeats.leftOut}</th>
            </tr>
          </thead>
          <tbody>
            {#each group.files as file (file.file)}
              <tr>
                <td>{file.file}&nbsp; {file.name}</td>
                <td>{file.pages}</td>
                <td>{file.copies}</td>
                <td>{file.kept}</td>
                <td>{file.left_out}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <div class="choices">
          {#each keeps as choice (choice.kind)}
            <label>
              <input
                type="radio"
                name="group-{group.number}"
                disabled={busy}
                checked={!group.by_hand && sameChoice(group.choice, choice)}
                onchange={() => choose(group.number, choice)}
              />
              {choice.kind === "keep_first" ? texts.repeats.keepFirst : texts.repeats.keepLast}
            </label>
          {/each}
          {#if group.files.length > 1}
            {#each group.take_out as option (option.file)}
              {@const choice = { kind: "take_out", file: option.file } as const}
              <label title={option.only_here > 0 ? texts.repeats.onlyHere(option.file, option.only_here) : ""}>
                <input
                  type="radio"
                  name="group-{group.number}"
                  disabled={busy || option.only_here > 0}
                  checked={!group.by_hand && sameChoice(group.choice, choice)}
                  onchange={() => choose(group.number, choice)}
                />
                {texts.repeats.takeOut(option.file)}
              </label>
            {/each}
          {/if}
          {#if group.by_hand}<span class="muted">{texts.repeats.byHand}</span>{/if}
        </div>
        {#each group.take_out.filter((option) => option.only_here > 0) as option (option.file)}
          <p class="muted">{texts.repeats.onlyHere(option.file, option.only_here)}</p>
        {/each}

        <button
          class="link"
          onclick={() =>
            (opened = opened.includes(group.number)
              ? opened.filter((n) => n !== group.number)
              : [...opened, group.number])}
        >
          {opened.includes(group.number) ? "▾" : "▸"}
          {texts.repeats.orders(group.orders.length)}
        </button>
        {#if opened.includes(group.number)}
          <table class="orders">
            <tbody>
              {#each group.orders as order (order.order_id)}
                <tr>
                  <td class="id">{order.difference ? "⚠ " : ""}{order.order_id}</td>
                  <td class="id">{order.tracking_id ?? ""}</td>
                  <td>{texts.repeats.difference(order.difference)}</td>
                  {#each order.copies as copy (copy.copy)}
                    <td>
                      <label>
                        <input
                          type="radio"
                          name="order-{order.order_id}"
                          disabled={busy}
                          checked={order.kept === copy.copy}
                          onchange={() => pick(order.order_id, copy.copy)}
                        />
                        {texts.repeats.copy(copy.file, copy.page_start)}
                      </label>
                    </td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </section>
    {/each}

    <div class="foot">
      <span>{texts.repeats.foot(view.extra_copies, read.pages - view.extra_pages, read.pages)}</span>
      <span class="grow"></span>
      <button class="primary" disabled={busy} onclick={onContinue}>{texts.repeats.continue}</button>
    </div>
  {/if}
</main>

<style>
  main {
    padding: 1rem;
  }

  .bar,
  .foot,
  .choices {
    display: flex;
    align-items: center;
    gap: 0.8rem;
  }

  .bar h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  .grow {
    flex: 1;
  }

  .summary {
    font-weight: 600;
  }

  .group {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.75rem;
    background: var(--surface);
    box-shadow: var(--shadow);
  }

  .group h3 {
    margin: 0 0 0.4rem;
    font-size: 1rem;
  }

  table {
    border-collapse: collapse;
    margin-bottom: 0.4rem;
  }

  th,
  td {
    padding: 0.2rem 0.6rem;
    text-align: left;
  }

  .choices label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .id {
    font-variant-numeric: tabular-nums;
  }

  .muted {
    color: var(--text-muted);
  }

  .foot {
    margin-top: 1rem;
  }
</style>
