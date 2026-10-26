// The **Printed** tick's rule: the Day screen sends one group PDF's tick to the `set_printed`
// command and updates the batch's rows in place, without reloading the whole overview. The rule is
// pure, so it is tested without a window; the write itself is the command, a `printed_marked`
// record in the date's journal.

// The batch's ticked label groups after ticking (`ticked: true`) or unticking `group`: ticking
// adds the group once, unticking removes it, and every other group is left as it is. The groups
// are one batch's own, so another batch's list is untouched.
export function togglePrinted(
  printed: string[],
  group: string,
  ticked: boolean,
): string[] {
  if (ticked) return printed.includes(group) ? printed : [...printed, group];
  return printed.filter((code) => code !== group);
}
