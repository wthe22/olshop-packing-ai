// The Repeated orders screen's choices, pure so they are tested without a window: the choices the
// back end holds, read from its view, and one changed — a card's choice clears that card's orders
// chosen by hand; an order's radio sets its copy by hand.

import type { ChoiceView, RepeatChoices, RepeatsView } from "./api/batch";
import { texts } from "../texts";

export function choicesOf(view: RepeatsView): RepeatChoices {
  return {
    groups: view.groups.map((group) => ({ group: group.number, choice: group.choice })),
    by_hand: view.groups.flatMap((group) =>
      group.orders.filter((order) => order.by_hand).map((order) => ({ order_id: order.order_id, copy: order.kept })),
    ),
  };
}

export function withGroupChoice(view: RepeatsView, number: number, choice: ChoiceView): RepeatChoices {
  const now = choicesOf(view);
  const group = view.groups.find((item) => item.number === number);
  const own = new Set(group?.orders.map((order) => order.order_id) ?? []);
  return {
    groups: now.groups.map((item) => (item.group === number ? { group: number, choice } : item)),
    by_hand: now.by_hand.filter((order) => !own.has(order.order_id)),
  };
}

export function withCopy(view: RepeatsView, orderId: string, copy: number): RepeatChoices {
  const now = choicesOf(view);
  return {
    groups: now.groups,
    by_hand: [...now.by_hand.filter((order) => order.order_id !== orderId), { order_id: orderId, copy }],
  };
}

export function sameChoice(a: ChoiceView, b: ChoiceView): boolean {
  return a.kind === b.kind && (a.kind !== "take_out" || b.kind !== "take_out" || a.file === b.file);
}

// The Plan screen's list of the copies left out: the order, where the copy is, the copy kept.
export function leftOutLines(view: RepeatsView): string[] {
  return view.groups.flatMap((group) =>
    group.orders.flatMap((order) => {
      const kept = order.copies.find((copy) => copy.copy === order.kept) ?? order.copies[0];
      return order.copies
        .filter((copy) => copy.copy !== order.kept)
        .map((copy) =>
          texts.repeats.leftOutLine(order.order_id, copy.file, copy.page_start, kept.file, kept.page_start),
        );
    }),
  );
}

// The file list's chip for each file that holds a repeat, by its number in the batch: its group
// and how many of its orders are also in which other files.
export function fileChips(view: RepeatsView): Map<number, { chip: string; line: string }> {
  const chips = new Map<number, { chip: string; line: string }>();
  for (const group of view.groups) {
    for (const file of group.files) {
      const orders = group.orders.filter((order) => order.copies.some((copy) => copy.file === file.file));
      const others = [
        ...new Set(orders.flatMap((order) => order.copies.map((copy) => copy.file)).filter((n) => n !== file.file)),
      ].sort((a, b) => a - b);
      chips.set(file.file, { chip: texts.repeats.chip(group.number), line: texts.repeats.alsoIn(orders.length, others) });
    }
  }
  return chips;
}
