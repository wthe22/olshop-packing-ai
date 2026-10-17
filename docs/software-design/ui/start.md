# Start

At every start the app looks for `couriers.toml` in its own folder
([the program folder](../app/program.md)):

1. **Found** → that folder is the data folder; the Day screen opens for the current day.
2. **Not found** (a first start, or a program copied without its data) → "No packing data
   next to the program (`D:\Packing\`)." [Use this folder] [Choose a folder…]
   - **Use this folder** writes the bundled `couriers.toml` (the repository's) there; every
     later start finds it. No picks are written: the picks start as one catch-all
     entry that takes every order — the grouping `default`
     ([rules file](../rules-file/README.md), [picks](../labels/picks.md), [Q-UI-4](../questions/ui.md)).
     If the folder cannot be written (e.g. under `Program Files`), the
     message says so and only **Choose a folder…** is left.
   - **Choose a folder…**: the folder is used as it is; a `couriers.toml`
     it does not have is written from the bundled file, and its own groupings (`categories.toml` and
     `categories/`) are kept. It
     is used until the app closes, so the next start asks again.
3. If a grouping has an error, Categories opens before the Day screen, on that grouping; a data folder
   without a grouping file is not an error.
