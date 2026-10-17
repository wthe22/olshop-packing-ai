// The **Printed** tick's rule (07 › *`state.json`*; 08 › *1. Day*): the Day screen sends one
// saved-PDF row's tick to the `set_printed` command and updates the batch's rows in place, without
// reloading the whole overview. The rule is pure, so it is tested without a window; the write
// itself is the command, which stores the batch's ticked saved-PDF numbers the same way
// (`state.json`'s `batches[].printed`).

/// The batch's ticked saved-PDF numbers after ticking (`ticked: true`) or unticking row `pdf`
/// (08 › *1. Day*): ticking adds the number once, unticking removes it, and every other row's
/// number is left as it is. The numbers are one batch's own, so another batch's list is untouched.
export function togglePrinted(
  printed: number[],
  pdf: number,
  ticked: boolean,
): number[] {
  if (ticked) return printed.includes(pdf) ? printed : [...printed, pdf];
  return printed.filter((number) => number !== pdf);
}
