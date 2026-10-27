// The window's one yes-or-no question, asked in the app's own dialog (`ConfirmDialog.svelte`)
// instead of the system's message box: `ask` shows it and resolves with the answer.

export interface Question {
  title: string;
  // One short line under the title; `null` when the title says it all.
  body: string | null;
  // The button that does it, named for what it does ("Undo batch 2").
  ok: string;
}

class ConfirmBox {
  question = $state<Question | null>(null);
  private resolve: ((yes: boolean) => void) | null = null;

  // A question asked while another is open answers the first one *no*.
  ask = (question: Question): Promise<boolean> => {
    this.resolve?.(false);
    this.question = question;
    return new Promise((resolve) => (this.resolve = resolve));
  };

  answer = (yes: boolean) => {
    const resolve = this.resolve;
    this.resolve = null;
    this.question = null;
    resolve?.(yes);
  };
}

export const confirmBox = new ConfirmBox();
