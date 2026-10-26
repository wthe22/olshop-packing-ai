# Messages

Every message says what happened and what to do. A **stop** ends the step with nothing written; a
**warning** is listed and the batch can still be saved.

## Reading a batch

| Kind | When | Message |
|---|---|---|
| Stop | Pages without a packing slip | **These labels have no packing slip.** `<n>` of `<m>` pages are plain shipping labels (`<file>`, pages `<list>`). In the seller centre, download the labels again with **Shipping label + Packing slip**. |
| Stop | First page of a file with no Order ID and no copy to continue | **Page `<p>` of `<file>` has no Order ID.** This does not look like a label download. Check the file. |
| Stop | A continuation page away from its label | **Page `<p>` of `<file>` continues order `<id>`, whose label is not right before it.** The pages of one order must be together. Download the labels again. |
| Stop | A file cannot be opened | **`<file>` cannot be read as a PDF.** It may be damaged or still downloading. Download it again. |
| Stop | Every order already saved on this date | **All `<n>` orders were already saved today** (batch `<b>`). These labels were prepared before; nothing to do. |
| Warning | Orders already saved on this date | **`<n>` orders were already saved today** and are left out: `<tracking ID>` (batch `<b>`, `<b>-<code>`, `<time>`), … |
| Warning | Orders that appear more than once | **`<n>` orders appear more than once in this batch:** `<m>` copies are left out (`<file>` page `<p>`, kept: `<file>` page `<p>`), … |
| Warning | Courier not recognised | **Courier unknown for `<n>` orders:** `<tracking IDs>`. They are sorted normally; only a filter on `courier` cannot see them. |
| Warning | `Qty Total` differs from the lines | **Slip total differs** for `<tracking ID>`: printed `<a>`, lines add up to `<b>`. Check that order's slip when packing. |
| Warning | No tracking ID on a label | **No tracking ID read** for order `<order ID>`. Its label is saved as usual; only a filter on `tracking_id` cannot see it. |

## Saving, reverting, reprinting

| Kind | When | Message |
|---|---|---|
| Stop | A file to replace or delete is open in another program | **`<file>` is open in another program.** Close it (e.g. the PDF viewer) and press `<button>` again. Nothing was changed. |
| Stop | Disk full or folder not writable | **Cannot write to `<folder>`:** `<reason>`. Nothing was saved. |
| Stop | The verifier found something | The dialog of [Plan › Saving](plan.md#saving): **Save anyway** or **Cancel — nothing is saved**. A reprint or a restore has no *anyway*: **The reprint does not match its orders:** `<finding>`. Nothing was kept. |
| Note | Saved, but a file could not be moved into place | **Batch `<b>` is saved, but `<file>` is open in another program.** Close it; the file is put in place when this day is opened again. |
| Stop | An input an amend, reprint or restore needs is missing or changed | **`<file>` (batch `<b>`) is missing from the day's inputs, or has changed.** It cannot be used. |

## Opening

| Kind | When | Message |
|---|---|---|
| Stop | Another copy of the app holds the data folder | **Packing is already open on `<computer>` (since `<time>`).** Close it there, then **Try again**. |
| Stop | `packing-workflows.toml` does not parse | **The rules file has an error:** `<line, col: message>`. Fix `packing-workflows.toml` beside the program. [Open folder] |
| Stop | A sort rule's filter has an error | **Sort rule `<name>` has an error:** `<line, col: message>`. The Sort rules screen opens on it. |
| Damage | The journal of a date is damaged, or names another namespace or date | **`<date>` of `<namespace>` cannot be changed:** line `<n>` of its journal `<is damaged / names another namespace>`. It can be viewed; nothing is written to it. |
| Warning | An input of a live batch is missing or changed | **`<file>`, an input of batch `<b>`, is missing or has changed.** Amend, reprint and restore cannot use it. |
| Warning | An output is missing or changed | On its batch card: **⚠ missing** / **⚠ changed** and **Restore**. |
| Note | A version 0.1 data folder | **This folder also holds data of Packing v0.1 (`labels\`). It is not read.** [Open folder] |
