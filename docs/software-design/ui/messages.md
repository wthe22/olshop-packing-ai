# Messages

Every message says what happened and what to do. Stop messages end the batch with nothing
written; warnings are listed and the batch can still be saved.

| Kind | When | Message |
|---|---|---|
| Stop | Pages without a packing slip | **These labels have no packing slip.** `<n>` of `<m>` pages are plain shipping labels (`<file>`, pages `<list>`). In the seller centre, download the labels again with **Shipping label + Packing slip**. |
| Stop | First page of a file with no Order ID | **Page `<p>` of `<file>` has no Order ID.** This does not look like a TikTok Shop label download. Check the file. |
| Stop | A file cannot be opened | **`<file>` cannot be read as a PDF.** It may be damaged or still downloading. Download it again. |
| Stop | Rules file error | **The categories have an error:** `<line, col: message>`. Fix it in Categories. [Open Categories] |
| Stop | Courier file error | **The couriers have an error:** `<line, col: message>`. Fix `couriers.toml` in the data folder. |
| Stop | Every order already saved for the day | **All `<n>` orders were already saved today** (batch `<b>`). These labels were prepared before; nothing to do. |
| Warning | Orders already saved for the day | **`<n>` orders were already saved today** and are left out: `<tracking ID>` (batch `<b>`, PDF `<k>`, `<time>`), … |
| Warning | Courier not recognised | **Courier unknown for `<n>` orders:** `<tracking IDs>`. They are sorted normally; only a condition on `courier` cannot see them. |
| Warning | `Qty Total` differs from the lines | **Slip total differs** for `<tracking ID>`: printed `<a>`, lines add up to `<b>`. Check that order's slip when packing. |
| Warning | Tracking ID not found on a page | **No tracking ID read** for Order `<order ID>`. Its label is saved as usual; only a condition on `tracking_id` cannot see it. |
| Error at save | A file is open in another program | **`<file>` is open in another program.** Close it (e.g. the PDF viewer) and press Save again. |
| Error at save | Disk full or folder not writable | **Cannot write to `<folder>`:** `<reason>`. Nothing was saved. |
