# Open questions: the rules file

Questions: Q-RULES-1.

## Q-RULES-1 — Several sets of categories: which set a batch uses, and where the sets are kept

**Words.** A **category** is one `[[category]]` entry of a picks file: a code, a name, and (except on
the last entry) a condition ([rules file](../rules-file/README.md)). It is the word the screens use
for a pick, and its name is the heading of the packing list and of the saved label PDF it fills
([Concepts](../requirements.md#concepts)). A **set** is a named collection of categories in the data
folder; a batch is packed with exactly one set, whose categories are applied in file order — each one
takes the matching orders from what is left ([picks](../labels/picks.md)). Today there is one
file, `categories.toml`, so the shop has one set of categories.

**Now.** The app reads one `categories.toml` in the data folder, so there is one set of picks
([rules file](../rules-file/README.md)). A data folder with no picks file at all has one catch-all
entry that takes every order, so every order goes into one PDF set
([inputs](../app/inputs.md)); with sets, that is the set `default`, and the set list always holds it.

**Wanted.** The owner can define several named sets of categories and use the set that fits. Two
choices come with it: **which set a batch uses**, and **where the sets are kept**. Both are the
owner's; the app follows one default of each until the owner decides. Whatever the answer, a batch
records the set it was packed with, so **Amend** (redo the batch) reads that same set again, not
whichever set happens to be in use afterwards ([revert and amend](../app/batch.md)).

**Example.** The shop keeps a normal set (A Sepatu, B Spion & Knalpot, Z Lainnya) and, for a day that
is nearly all shoes, a set of its own (A Sepatu pria, B Sepatu wanita, C Tas, Z Lainnya). The owner
downloads that day's labels, opens **New batch**, chooses the shoes set, reads the labels and saves;
a week later both sets are still there, unchanged, and the next batch starts on the set used last.
A week after that the owner redoes an old batch: it is packed with the set that batch used, even
though another set has been used since.

### (a) Which set a batch uses

| Option | What the owner sees |
|---|---|
| **1. Chosen per batch on the New batch screen, the last-used set preselected** (**Default**) | **New batch 3** shows *Set: [default ▾]*, chosen before **Read labels**; the plan is that set's categories. The choice is remembered as the last used, so a shop that always uses one set never touches the control, and a week of the same set costs nothing. The batch records its set (a `set` field in the day's `state.json`), so **Amend** reads the batch's own set again. Cost: one control on the screen and the stored field |
| 2. One active set chosen in Settings | One set is *the* set ([Settings](../ui/settings.md)); every batch uses it until it is switched there. The fewest clicks a day, and a day that needs another set means a trip to Settings first; a batch prepared with the wrong set is redone by an amend |
| 3. Chosen per day folder | The set belongs to the day: it is chosen when the day's first batch starts and every batch of that day uses it. One choice a day; a day with two kinds of download cannot use two sets |

**What the answer changes:** the New batch screen (the control, or none), `settings.json` (which set
was used last, or the active set), `state.json`'s `batches[]` (the set each batch records), the amend,
and the Settings screen for option 2.

### (b) Where the sets are kept

| Option | What the owner sees |
|---|---|
| **1. One file per set, the existing `categories.toml` being the set `default`** (**Default**) | `<data folder>/categories/ramadhan.toml` holds the set *ramadhan* — the file name is the set name, so a set can be copied, renamed or written by hand. The set `default` stays the file `categories.toml` the app already writes: nothing is moved and today's data folder works as it does now. The set list is `default` plus every file in `categories/` |
| 2. Named sections inside the one `categories.toml` | One file: e.g. a `[sets.ramadhan]` heading over the `[[sets.ramadhan.category]]` entries. One file to copy and back up; the file grows with every set, and editing the wrong section by hand is easy to miss |
| 3. One file per set beside `categories.toml` | Every set is one file in the data folder itself (`default.toml`, `ramadhan.toml`, …), next to `couriers.toml`; the sets gather in the folder the owner already opens, and today's `categories.toml` has to be renamed to `default.toml` or read under that name |

**What the answer changes:** which files the app reads and writes, what the Categories screen lists,
and whether today's `categories.toml` keeps its name.

Recommended: **1** in both. A batch chooses its set where the batch is prepared, the sets are files
the owner can copy or correct by hand, and an existing data folder keeps its `categories.toml` where
it is. Task 2.27 in [development/plan/](../../development/plan/README.md) builds this default; an
answer is written into the design files and into that task file before it is built.
