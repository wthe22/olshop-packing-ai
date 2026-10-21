# Sessions

A **session** is one working space of the owner: a date and an optional name. It holds its own
batches, numbers and `state.json`. A day can have several sessions — one per shop, or one for
testing a setup — and each is listed in the side panel ([Sessions panel](../ui/sessions.md)).

| Word | Meaning |
|---|---|
| **Day** | A calendar date (`2026-10-07`). A day holds one or more sessions |
| **Session** | A day plus a name. The session with no name is **Main**; its folder has no name part |

## Where a session lives

Every session is one folder under `labels/` ([data-folder.md](data-folder.md)); everything inside the
folder is as it is today.

| Session | Folder |
|---|---|
| Main of 2026-10-07 | `labels/2026-10-07/` |
| `Shop B` of 2026-10-07 | `labels/2026-10-07 Shop B/` |

- The folder name starts with the date, so a sorted list is in date order. The date is the first
  ten characters; the name is whatever follows a space.
- **Main** is a folder named by the date alone: `labels/2026-10-07/` is the Main session of that day.
- A name is 1–40 characters. Characters Windows does not allow in a file name (`\ / : * ? " < > |`)
  become `-`. Two sessions of one day cannot have the same name (upper and lower case count as the
  same); `Main` is not a name — it is the session with no name.
- The folder name is the only home of the name: `state.json` keeps its `day` field and gets no new
  one. The default is [Q-APP-4](../questions/sessions.md).

## What a session holds on its own

Everything the day held until now belongs to the session, so two sessions never touch each other:

- the batches, numbered `1, 2, 3, …` from 1 in each session;
- the saved-PDF numbers, counted from 1 in each session;
- the **already saved** guard ([picks.md](../labels/picks.md)): an order saved in `Shop B` is not
  left out of a batch in Main. A tested setup can therefore read the same labels again without
  disturbing the real session. Whether the app should warn across sessions is
  [Q-APP-5](../questions/sessions.md);
- the *used in batch n* hint of the file list;
- revert and amend: each session has its own last batch ([batch.md](batch.md)).

What is shared is the data folder: the groupings, `couriers.toml` and `settings.json`
([data-folder.md](data-folder.md)). The grouping used last stays one setting for the whole app.

## Listing, opening and creating

- **The list** is every folder of `labels/` whose name starts with a date, read when the app starts
  and again whenever the window regains focus, so a session added or removed in Explorer shows up.
- **Today's Main** is always in the list, even before a folder exists: it is created when the first
  batch is saved ([batch.md](batch.md)). The app opens on it at every start.
- **A new session** (**+ New session**) creates the folder at once, empty, so the session stays in
  the list after a restart even before anything is saved in it. An empty session has no
  `state.json`; it is written at the first save.
- **A rename** moves the folder. It is refused with the *file in use* message
  ([Messages](../ui/messages.md)) while Windows holds a file of it open, and while a batch is being
  prepared in that session ("Save or cancel the batch first.").
- **A session is never deleted by the app**; the owner removes the folder in Explorer
  ([Q-UI-6](../questions/sessions.md)).

## What the rest of the design reads as

The words *day* in the other documents mean the session when they speak of batches, numbers, the
already-saved guard, revert, amend, the packing-list header and `summary.txt`. The packing-list
header and the summary's first line carry the session name when it is not Main
(`Packing list · 2026-10-07 · Shop B · batch 1 · 2  B  Spion & Knalpot`).
