// The app's own yes-or-no dialog, answered by the test: each question asked is kept, in order, and
// `answer(question)` says yes or no at once, as the owner's click would.

import { useRunes } from "./tsModule.mjs";

useRunes();

const { confirmBox } = await import("../src/lib/confirm.svelte.ts");

export function answerQuestions(answer) {
  const asked = [];
  confirmBox.ask = async (question) => {
    asked.push(question);
    return answer(question);
  };
  return asked;
}
