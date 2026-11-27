# How a date is saved

Every change to a date — a save, an amend, a revert, a reprint, a restore, a mark — writes the date's
[files](files.md) again whole. Crate `packing-store` owns this.

## A commit

1. Write what the change makes into `.partial/` of the date folder: the outputs (group PDFs, summary PDF,
   reprint), then `orders.csv`, `order-contents.toml` and `day.toml` as they will be. A mark (Print, the
   printed tick) writes only `day.toml`.
2. Verify the outputs there when the change has any ([verification](../app/verification.md)).
3. Files the change takes away or replaces (an amend's or a revert's outputs) move into
   `.partial/replaced/`. A file Windows refuses to move stops the change: the files moved go back, and the
   message names the file to close.
4. **Rename `.partial/day.toml` over `day.toml`.** This is the commit: before it nothing is saved, after it
   the change is.
5. Move every other file of `.partial/` into the date folder, replacing a file of the same name; delete
   `.partial/` with `replaced/`.

Each file is written to its place and `sync_all`ed before the rename that follows it. A rename never leaves
half a file, so there is no torn write to repair.

## When a date opens

No date opens at start until the owner has answered the migration's question, when there is one
([01 Start](../workflow/01-start-and-open.md#1-start)). Then, in this order (step 1 writes, and runs only
while this copy holds the data folder's lock: [data › One writer](README.md#one-writer); in read-only mode it
is skipped):

1. **`.partial/`**, if present, is settled:
   - **back** when it still holds `day.toml`: the change did not commit. The files of `replaced/` move back
     into the date folder; the rest is deleted;
   - **forward** otherwise: the change committed. Its files move into the date folder and `replaced/` is
     deleted.
2. **Hashes**: `orders.csv` or `order-contents.toml` that is missing beside `day.toml`, or whose SHA-256 is
   not the one `day.toml` holds (changed outside the program), is named on the [Day screen](../ui/day.md)
   with Read labels again, Accept (a changed file) and Read only ([D-APP-40](../decisions/date-files.md)).
   Until the owner chooses, the date is shown from what is there and takes no write.
3. **Read** the three files. A file that does not read, a `date` that is not the folder's, or an order,
   group or content that one file names and another lacks is **damage**: the date stops with a plain message
   naming the file and line ([Messages](../ui/messages.md)); it can be viewed, nothing is written to it. It is
   never a crash, and the app never repairs it. A file named in step 2 is damage only once accepted.
4. **Inputs**: every input of a live batch must exist in the date folder with its `hash_sha256`. One that is
   missing or changed is reported and never used: an amend, reprint or restore that needs it stops with
   its name.
5. **Outputs**: every output of a live batch must exist in the date folder. A missing one is shown on its
   batch card with **Restore** ([reprint](../app/reprint.md#restore)).

A date folder with none of the three files is empty: nothing is read and no file is made until its first
commit.

## Size

`orders.csv` takes about 150 bytes an order: a date of three batches and 600 orders is about 90 KB. The
search index reads every date's `orders.csv` and `order-contents.toml` at start
([reprint](../app/reprint.md#the-search-index)).
