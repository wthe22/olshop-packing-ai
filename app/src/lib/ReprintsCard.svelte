<script lang="ts">
  // The Day screen's Reprints card: one row per reprint of the date, like a group PDF's row —
  // its number, when it was made, its counts and **Open** — and *Orders ▸*, which lists its orders
  // as Search shows them. The state is `reprintList.svelte.ts`.
  import Count from "./Count.svelte";
  import { ReprintList } from "./reprintList.svelte";
  import { itemsText } from "./search.svelte";
  import type { ReprintRow } from "./api/day";
  import { texts } from "../texts";
  import { formatShortDay } from "../dates";

  let {
    day,
    reprints,
    onOpen,
  }: {
    // The date the reprints were made on.
    day: string;
    reprints: ReprintRow[];
    onOpen: (path: string) => void;
  } = $props();

  const list = new ReprintList(() => day);

  const orders = $derived(reprints.reduce((sum, reprint) => sum + reprint.orders, 0));
</script>

<section class="card">
  <div class="head">{texts.day.reprints(reprints.length, orders)}</div>
  {#each reprints as reprint (reprint.number)}
    <div class="row">
      <span class="code">{texts.day.reprintCode(reprint.number)}</span>
      <span class="time">{texts.day.reprintMade(reprint.time)}</span>
      <Count n={reprint.orders} word={texts.day.orderWord(reprint.orders)} />
      <Count n={reprint.pages} word={texts.day.pageWord(reprint.pages)} />
      <span class="grow"></span>
      <button
        class="link"
        aria-expanded={list.isOpen(reprint.number)}
        onclick={() => list.toggle(reprint.number)}
        >{texts.day.reprintOrders} {list.isOpen(reprint.number) ? "▾" : "▸"}</button
      >
      <button onclick={() => onOpen(reprint.path)}>{texts.day.open}</button>
    </div>
    {#if list.isOpen(reprint.number)}
      {@const printed = list.orders[reprint.number]}
      {#if !printed}
        <p class="muted orders">{texts.day.loading}</p>
      {:else}
        <table class="orders">
          <tbody>
            {#each printed as order (order.day + order.order_id)}
              <tr>
                <td class="id">{order.order_id}</td>
                {#if order.found}
                  <td class="id">{order.found.tracking_id ?? ""}</td>
                  <td class="muted">{order.found.courier ?? ""}</td>
                  <td class="muted">{itemsText(order.found)}</td>
                  <td class="muted where">{formatShortDay(order.day)} · {order.found.group}</td>
                {:else}
                  <td class="muted" colspan="4">
                    {texts.day.savedIn(formatShortDay(order.day), order.batch)}
                  </td>
                {/if}
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/if}
  {/each}
  {#if list.error}
    <p class="error">{list.error}</p>
  {/if}
</section>

<style>
  .card {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    margin-bottom: 0.75rem;
  }

  .head {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-soft);
    font-weight: 600;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0.75rem;
  }

  .row + .row,
  .orders + .row {
    border-top: 1px solid var(--border-soft);
  }

  /* The reprint's number, as wide as a group PDF's code on the batch cards. */
  .code {
    min-width: 3.5rem;
    white-space: nowrap;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .time {
    min-width: 6rem;
  }

  .muted {
    color: var(--text-muted);
  }

  .grow {
    flex: 1;
  }

  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--link);
    cursor: pointer;
  }

  .orders {
    width: calc(100% - 4.25rem);
    margin: 0 0.75rem 0.5rem 4.25rem;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  p.orders {
    margin-top: 0;
  }

  .orders td {
    padding: 0.2rem 0.4rem 0.2rem 0;
    border-top: 1px solid var(--border-soft);
    vertical-align: top;
  }

  .id {
    font-variant-numeric: tabular-nums;
  }

  .where {
    white-space: nowrap;
  }
</style>
