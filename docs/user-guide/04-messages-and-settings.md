# Messages and settings

## The messages you will meet

A **stop** ends the step and writes nothing. A **warning** is listed under **Warnings ▸**; the batch
can still be saved. The full list is in [the screens' messages](../software-design/ui/messages.md).

| Message | What to do |
|---|---|
| **These labels have no packing slip.** | You downloaded plain labels. In the seller centre, download again with **Shipping label + Packing slip**. |
| **Page `<p>` of `<file>` has no Order ID.** | The file does not look like a label download. Take it off the list and use the right file. |
| **`<file>` cannot be read as a PDF.** | It may be damaged or still downloading: download it again. |
| **All `<n>` orders were already saved today** | These labels were done in an earlier batch of this date. Nothing to do. |
| **`<n>` orders were already saved today** and are left out | A warning: those orders are skipped. Take the file off the list if you did not mean to read it again. |
| **`<n>` orders are in the files more than once** | A warning: one copy of each order is kept ([Repeated orders](02-a-day.md#repeated-orders)). |
| **Courier unknown** for some orders | A warning: they are sorted normally; only a filter on the courier cannot see them. |
| **Slip total differs** | A warning: check that order's packing slip when you pack it. |
| **`<file>` is open in another program.** | Close the PDF viewer, then press the button again. |
| **The rules file has an error:** `<line, col: message>` | Fix `packing-workflows.toml` beside the program, or the rule in **Sort rules**. |
| **Packing is already open on `<computer>`** | Close the other copy, then **Try again**. |
| **This folder also holds data of Packing v0.1** | The data of the old version stays where it is and is not read. |

When the checker finds something while saving, a dialog lists the errors and the warnings apart, with
**Save anyway** and **Cancel — nothing is saved** ([Saving and the check](02-a-day.md#saving-and-the-check)).
A reprint or a restore that the checker refuses keeps nothing and says why.

## Settings

Open **Settings** in the title bar.

- **Data folder** — where the namespaces and their dates are kept (the default is `data` beside the
  program). **Change…** asks "Use `<folder>` for the data from now on? The data in `<old>` stays where it
  is." and opens that folder; nothing is moved. **Open** shows it in Explorer.
- **About** — the program's version (**Version v0.2**), the PDFium version and the data's schema
  version.

There are no other settings: the sort rule of your newest batch is preselected, the summary PDF is
always written, and the couriers and sort rules live in `packing-workflows.toml` beside the program.

The app is in English. Every screen text is in one file, so another language can be added.
