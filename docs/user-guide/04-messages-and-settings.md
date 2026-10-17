# Messages and settings

## Every message and what to do

**Stop** messages end the batch and write nothing. **Warnings** are listed (click **Warnings ▸**);
the batch can still be saved.

| Message | What to do |
|---|---|
| **These labels have no packing slip.** `<n>` of `<m>` pages are plain shipping labels (`<file>`, pages `<list>`). | You downloaded plain labels. In the seller centre, download again with **Shipping label + Packing slip**. |
| **`<file>`: page `<p>` has no Order ID** | The file does not look like a TikTok Shop label download (or is damaged). Remove it and use the right file. |
| **`<file>` cannot be read as a PDF.** It may be damaged or still downloading. Download it again. | Download the file again, then add it. |
| **`<line, col: message>`** — a category error, e.g. `line 18, col 8: field "product_category" needs the orders CSV, which the PC app does not read` | Fix the condition in Categories (the app opens it for you). |
| **The couriers have an error:** `<line, col: message>` | Fix `couriers.toml` in the data folder. |
| **All `<n>` orders were already saved today** (batch `<b>`). These labels were prepared before; nothing to do. | These labels were done in an earlier batch today. Nothing to do. |
| **`<n>` orders were already saved today** and are left out: `<tracking ID>` (batch `<b>`, PDF `<k>`, `<time>`), … | A warning: those orders are skipped. Remove the file if you did not mean to read it again. |
| **no courier could be deduced for:** `<order IDs>` | A warning: those orders are sorted normally; only a category that uses `courier` cannot see them. Check those labels by eye if you use one. |
| **`<order ID>`: Qty Total `<a>` differs from the sum of Qty `<b>`** | A warning: check that order's packing slip when you pack it. |
| **no tracking ID for:** `<order IDs>` | A warning: those labels are saved as usual; only a category that uses `tracking_id` cannot see them. |
| **`<file>` is open in another program.** Close it (e.g. the PDF viewer) and press Save again. | Close the PDF viewer, then press **Save batch** again. |
| **Cannot write to `<folder>`:** `<reason>`. Nothing was saved. | Check the disk is not full and the folder is writable, then save again. |

Before the batch is saved, the app reads its own saved PDFs and packing lists back and compares
them with the plan. When it finds something, a dialog names it in plain words — the errors and the
warnings in two lists — with **Save anyway** and **Cancel — nothing is saved**:

| Text | What to do |
|---|---|
| **The saved files do not match the plan.** | the check found something: read the two lists |
| **Errors:** `<n>` / **Warnings:** `<n>` | the findings, in plain words |
| **Save anyway** | save the batch anyway; the two lists stay on its card and in its `state.json` |
| **Cancel — nothing is saved** | look the batch over again; the Plan screen is still there |

A warning stops the batch too: one whose pages are all there but in another order is saved only
after **Save anyway**.

## Settings

Open **Settings** in the top bar.

- **Data folder** — where `couriers.toml`, your picks and the `labels` folder live. Your picks are
  the grouping `default` in `categories.toml` (once you have saved some) and every other grouping in the
  `categories` folder, one file each ([Groupings](03-changes.md#groupings)).
  **Change…** switches to
  another folder until the app closes ("Use `<folder>` until the app closes? Existing days stay in
  `<old>`."); **Open** opens it. It does not move existing files.
- **Packing list default** — the scope and layouts the Plan screen starts with (default: one for
  the batch, layout *pick*). **Save** stores it.
- **About** — the program's version (**Version v0.1**) and the PDFium version.

The grouping of categories you used last is remembered too (in `settings.json`), and the next
**New batch** already has it chosen; there is nothing to set for it.

The app is in English. Every screen text is in one file, so another language can be added.
