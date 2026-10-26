// The batch flow's buttons and screens, pure so a test reads them without a window.

import { texts } from "../texts";

// The Plan screen's two ways out, from the kind of batch. The top button is
// *Back* on both — it returns to the PDF selection page the batch came from, with the files still
// listed, and is not a discard; the bottom discard button beside *Save batch* exists on an amend
// only and goes to the Day screen. Which
// screen each button leads to is part of the helper, so a test reads it without a window.

// The screen a Plan-screen button leads to: the PDF selection page (a new
// batch's file list, or the amend's selection page) or the Day screen.
export type PlanTarget = "selection" | "day";

// The Plan screen's buttons: their texts and the screen each one leads to. `discard` is `null`
// when there is no bottom discard button (a new batch).
export interface PlanButtons {
  top: string;
  topTarget: PlanTarget;
  discard: string | null;
  discardTarget: PlanTarget | null;
}

// The Plan screen's top button and bottom discard button, with their targets. `amend` is the
// batch being redone, or `null` for a new batch: an amend's top *Back* returns
// to its selection page and its *Cancel amend* to the Day screen; a new batch has neither a
// *Cancel amend* nor a button that leaves for the Day screen (only its file list's *Cancel*
// does, and it is not on this screen).
export function planButtons(amend: number | null): PlanButtons {
  return {
    top: texts.plan.back,
    topTarget: "selection",
    discard: amend === null ? null : texts.plan.cancelAmend,
    discardTarget: amend === null ? null : "day",
  };
}

// **Amend** on the newest batch reads the batch's own `download/` files
// again and opens the Plan screen (the picks) at once, titled *Redo batch `<b>`*; the PDF
// selection page comes only from that Plan screen's *Back*.
// Pure, so the flow is tested without a
// window.

// The batch-flow screens, named as the shell's own `Screen` is: *Day*, the
// PDF selection page (a new batch's file list, or the amend's selection page) or the Plan screen.
export type FlowScreen = "day" | "new-batch" | "plan";

// The screen **Amend** opens: the Plan screen — the picks first, read from the
// batch's own `download/` files — never the PDF selection page.
export function amendOpens(): FlowScreen {
  return "plan";
}
