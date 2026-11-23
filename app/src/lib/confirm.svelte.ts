// The window's one question, asked in the app's own dialog (`ConfirmDialog.svelte`) instead of the
// system's message box: `ask` shows a yes-or-no question and resolves with the answer; `choose` shows
// a question with two ways to do it and resolves with the button pressed.

export interface Question {
  title: string;
  // One short line under the title; `null` when the title says it all.
  body: string | null;
  // The button that does it, named for what it does ("Undo batch 2").
  ok: string;
}

// A question with a second way to do it, a button beside `ok`.
export interface Choice extends Question {
  other: string;
}

export type Answer = "ok" | "other" | "cancel";

class ConfirmBox {
  question = $state<Question | Choice | null>(null);
  private resolve: ((answer: Answer) => void) | null = null;

  ask = async (question: Question): Promise<boolean> => (await this.choose(question)) === "ok";

  // A question asked while another is open answers the first one *cancel*.
  choose = (question: Question | Choice): Promise<Answer> => {
    this.resolve?.("cancel");
    this.question = question;
    return new Promise((resolve) => (this.resolve = resolve));
  };

  answer = (answer: Answer) => {
    const resolve = this.resolve;
    this.resolve = null;
    this.question = null;
    resolve?.(answer);
  };
}

export const confirmBox = new ConfirmBox();
