# Open questions: sessions

Questions: Q-APP-4, Q-APP-5, Q-UI-5, Q-UI-6, Q-UI-7. The design is [sessions](../app/sessions.md) and
the [Sessions panel](../ui/sessions.md); each default is what they describe.

## Q-APP-4 — Where a session lives on disk

**Words.** A session is a day plus an optional name ([sessions](../app/sessions.md)). Today a day is
one folder, `labels/2026-10-07/`, holding its `state.json` and `batch n` folders.

**Example.** On 7 Oct the owner works Main and a second shop, `Shop B`.

| Option | What the owner sees on disk |
|---|---|
| **1. One folder per session, the name after the date** (**Default**) | `labels/2026-10-07/` and `labels/2026-10-07 Shop B/`. Existing folders are the Main sessions: nothing moves |
| 2. A folder per day, a folder per session inside | `labels/2026-10-07/Main/` and `labels/2026-10-07/Shop B/`. Cleaner to browse; every existing day folder is moved into `Main` once (a migration) |
| 3. Opaque folders, the name kept in `state.json` | `labels/2026-10-07-2/` with `"name": "Shop B"` inside. Explorer shows no name |

**What the answer changes:** the list and open commands (`days_in`, `AppState.day_dir`), the
session rules in [sessions.md](../app/sessions.md), [data-folder.md](../app/data-folder.md) and, for
option 2 or 3, a migration of the stored data.

## Q-APP-5 — What two sessions of one day share

**Words.** The *already saved* guard ([picks.md](../labels/picks.md)) leaves out an order saved earlier
in the session. Each session has its own guard, numbers and last batch.

**Example.** `Shop B` is a test of a new grouping, read from the labels Main saved an hour ago.

| Option | What the owner sees |
|---|---|
| **1. Nothing is shared** (**Default**) | The labels read again in `Shop B` are all new orders; no warning anywhere |
| 2. A note across sessions of the same day | The same, plus a read warning "14 orders are also saved in Main (batch 2)"; nothing is left out |
| 3. The guard spans the day | The 14 orders are left out of `Shop B` as already saved: a test cannot reuse real labels |

**What the answer changes:** the guard input of `drop_saved` (`crates/engine/src/day/guard.rs`), one
read warning ([Messages](../ui/messages.md)) and the wording in [sessions.md](../app/sessions.md).

## Q-UI-5 — A long list of sessions

**Words.** The panel grows by at least one row a day: about 365 days after a year.

| Option | What the owner sees |
|---|---|
| **1. Months fold; the current and the open one start open** (**Default**) | `▾ October 2026` open, `▸ September 2026` closed, each with its days under it |
| 2. One flat list | Every day in a single scrolling list, newest on top; nothing is hidden |
| 3. The last 30 days, then **Show older** | A short list; older days load on a click |

**What the answer changes:** the panel's list code and [Sessions panel](../ui/sessions.md) only.

## Q-UI-6 — What a session row can do

**Words.** A row opens its session on a click. The row menu (**⋯**) can offer more.

| Option | What the owner sees |
|---|---|
| **1. Rename… and Open folder** (**Default**) | A test session is renamed or finished in Explorer; the app never deletes a folder of saved labels |
| 2. Also **Delete session…** | A confirmation names the batches and orders lost; a session with batches needs the name typed |
| 3. Open only | No menu; rename and delete are done in Explorer |

**What the answer changes:** the row menu and the rename rules in [sessions.md](../app/sessions.md).

## Q-UI-7 — The title bar's day stepper

**Words.** The title bar has ◀ ▶ **Today** to step through days ([Window frame](../ui/README.md)). With
the panel listing every day and session, they say the same thing a second time.

| Option | What the owner sees |
|---|---|
| **1. The panel replaces them** (**Default**) | The title bar names the day and the session and has the ☰ button |
| 2. They stay beside the name | ◀ ▶ step to the previous and next **session**; **Today** opens today's Main |

**What the answer changes:** the window frame in [ui/README.md](../ui/README.md) and
[sessions.md](../ui/sessions.md).
