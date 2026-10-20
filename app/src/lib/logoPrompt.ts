// The logo prompt's flow (08 › *2. New batch* → *The logo prompt*, D-LABEL-4): after a read and
// before the Plan screen, each unseen logo of the batch is asked about once. The screen renders
// `current`; this class decides what happens next, so the flow is tested without a window.
//
//   a courier name    → the back end names the orders whose tracking-ID form agrees
//   Not a courier     → the back end names nothing
//   I don't know, Esc → `skip`: nothing is named, and the logo asks again in the next batch
//   Skip all          → stop asking for the rest of this batch; nothing is sent
//
// The answers go straight to the open draft of the running app; *Skip all* is this module's
// alone (nothing is written for it).

import type { LogoAnswer, LogoView, ReadResult } from "./api/batch";

// Sends one answer to the back end and returns the refreshed read result (`answerLogo` in
// `api/batch.ts`).
export type AnswerLogo = (hash: string, answer: LogoAnswer) => Promise<ReadResult>;

// Whether a read needs the prompt before the Plan screen: false when the courier data knows
// every logo of the batch, and the Plan opens at once.
export function needsLogoPrompt(result: ReadResult): boolean {
  return result.logos.length > 0;
}

// One batch's logo prompt: the unseen logos in first-seen order, the one being asked about, and
// the latest read result (for the Plan screen).
export class LogoPrompt {
  private readonly answered = new Set<string>();
  private stopped = false;
  private result: ReadResult;
  private readonly answerLogo: AnswerLogo;

  constructor(result: ReadResult, answerLogo: AnswerLogo) {
    this.result = result;
    this.answerLogo = answerLogo;
  }

  // Whether the read needs the prompt at all.
  get needed(): boolean {
    return needsLogoPrompt(this.result);
  }

  // The logo to ask about now, or `null` when the batch has no more prompts (every logo
  // answered, or *Skip all*).
  get current(): LogoView | null {
    if (this.stopped) return null;
    return this.result.logos.find((logo) => !this.answered.has(logo.hash)) ?? null;
  }

  // The latest read result, for the Plan screen.
  get latest(): ReadResult {
    return this.result;
  }

  // Answer the logo shown now and move on to the next; nothing happens when the list is done.
  async answer(answer: LogoAnswer): Promise<void> {
    const logo = this.current;
    if (!logo) return;
    // Marked before the call: an answer is spent even if the request fails, so the same logo is
    // never asked twice.
    this.answered.add(logo.hash);
    this.result = await this.answerLogo(logo.hash, answer);
  }

  // *Skip all*: stop asking for the rest of this batch. Nothing is sent; what was answered
  // before stays applied.
  skipAll(): void {
    this.stopped = true;
  }
}
