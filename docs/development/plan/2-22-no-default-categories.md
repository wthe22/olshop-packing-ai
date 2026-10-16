# 2.22 First use writes no `categories.toml`; a missing file means one catch-all pick

A first use writes only `couriers.toml`, and the data folder is found by `couriers.toml`
([start.md](../../software-design/ui/start.md), [program.md](../../software-design/app/program.md)).
A missing `categories.toml` means one catch-all entry with no condition, so a batch saves every
order in one PDF set ([inputs.md](../../software-design/app/inputs.md),
[picks.md](../../software-design/labels/picks.md)); the owner adds picks in
[Categories](../../software-design/ui/categories.md). The line that says the list is unconfigured is
[Q-UI-4](../../software-design/questions/ui.md) (default: the Categories screen and the Plan screen).

- **Code:** `crates/engine` — a missing `categories.toml` loads as one catch-all entry (the rest)
  instead of the *no categories* error (`rules.rs`, `plan.rs`). `app/src-tauri` — `commands/day.rs`:
  *Use this folder* writes only the bundled `couriers.toml`, the program folder is found by
  `couriers.toml`, and *Choose a folder…* / Settings *Change…* write only a missing `couriers.toml`;
  `lib.rs`: the bundled `categories.toml` is no longer copied into the data folder. The screens:
  `CategoriesScreen.svelte` shows the one catch-all entry and the Q-UI-4 line, `PlanScreen.svelte`
  and `texts.ts` show the line on the Plan screen.
- **Tests:** a first start writes `couriers.toml` only; the program folder is found by its
  `couriers.toml`; a read with no `categories.toml` gives one pick holding every order, and saving it
  writes one PDF set with every order in it; the Categories screen loads the catch-all entry.
- **Result:** (not started)
