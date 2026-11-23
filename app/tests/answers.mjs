// The app's own dialog, answered by the test: each question asked is kept, in order, and
// `answer(question)` answers at once, as the owner's click would: yes or no, or the button
// ("ok", "other", "cancel") of a question with two ways to do it.

import { useRunes } from "./tsModule.mjs";

useRunes();

const { confirmBox } = await import("../src/lib/confirm.svelte.ts");

export function answerQuestions(answer) {
  const asked = [];
  confirmBox.choose = async (question) => {
    asked.push(question);
    const given = answer(question);
    if (typeof given === "string") return given;
    return given ? "ok" : "cancel";
  };
  return asked;
}
