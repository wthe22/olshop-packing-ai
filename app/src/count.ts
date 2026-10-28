// "1 order", "3 orders": a count with its word, for the screen texts.
export function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}
