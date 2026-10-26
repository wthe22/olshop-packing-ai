// The faked back end and the fixtures the shell's own tests share: the commands every screen
// needs, a day's overview and a read result. No window opens and nothing is written.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { useRunes } from "./tsModule.mjs";

useRunes();

export const { Shell } = await import("../src/lib/shell.svelte.ts");

// The day's overview the faked `day_overview` answers with.
export const overview = (batches = 2) => ({
  day: "2026-10-07",
  totals: { batches, orders: 7, files: 3, pdfs: 4, packing_lists: 4, warnings: 0 },
  batches: [],
  days: [],
});

// A read result, clean unless the caller says otherwise.
export function read(overrides = {}) {
  return {
    files: 2,
    pages: 24,
    orders: 23,
    total_orders: 23,
    already_saved: 0,
    warnings: [],
    amend: null,
    couriers: ["J&T Express"],
    logos: [],
    repeats: null,
    plan: { rows: [], has_duplicate_names: false, add_at: 0, no_groups: false, sort_rule: "default" },
    ...overrides,
  };
}

// The commands every shell test needs; a test adds its own or asks this one to fail. A confirm
// dialog answers with the label of the button it was given: *Ok*, or the caller's own label.
export function backend(extra = {}) {
  return {
    workspaces_list: () => ({ dates: [{ date: "2026-10-07" }] }),
    day_overview: () => overview(),
    about: () => ({ version: "0.2", pdfium: "140.0", schema: "1.0" }),
    "plugin:event|listen": () => 1,
    "plugin:event|unlisten": () => undefined,
    // A confirm dialog answers with the label of the *Ok* button it was given: the caller's own
    // label, or *Ok*. Rust names the two labels `OkCancelCustom`.
    "plugin:dialog|message": (args) => args.buttons?.OkCancelCustom?.[0] ?? "Ok",
    ...extra,
  };
}
