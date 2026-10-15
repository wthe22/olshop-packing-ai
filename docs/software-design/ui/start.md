# Start

At every start the app looks for `categories.toml` in its own folder
([the program folder](../app/program.md)):

1. **Found** → that folder is the data folder; the Day screen opens for the current day.
2. **Not found** (a first start, or a program copied without its data) → "No packing data
   next to the program (`D:\Packing\`)." [Use this folder] [Choose a folder…]
   - **Use this folder** writes the bundled `categories.toml` and `couriers.toml` (the
     repository's) there; every later start finds them. If the folder cannot be written (e.g. under `Program Files`), the
     message says so and only **Choose a folder…** is left.
   - **Choose a folder…**: a folder with a `categories.toml` is used as it is; an empty one gets
     the bundled file. It is used until the app closes, so the next start asks again.
3. If `categories.toml` has errors, Categories opens before the Day screen.
