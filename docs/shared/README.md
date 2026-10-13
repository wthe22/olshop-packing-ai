# Shared

What the PC tools ([`../desktop/`](../desktop/README.md)) and the Android app
([`../mobile/`](../mobile/README.md)) have in common.

| File | Content |
|---|---|
| [04-rules-file.md](04-rules-file.md) | `categories.toml` and the condition language: used by the PC app and the Android app. A file written by one must load unchanged in the other |

## How the tools relate

- The PC side does all PDF work: reading the shipping labels and packing slips, the picks, the
  saved label PDFs and the A4 packing lists. The Android app never opens the label PDFs.
- The Android app reads the orders CSV; the PC app does not (all its data comes from the label and
  its packing slip).
- They share one rules file, `categories.toml`.
- There is no network connection between them; files are moved by hand (USB, cloud folder, chat to
  self).

## Conventions

Apply to every design, doc and code of both tools.

- Timestamps are named `*_time` (`create_time`, `import_time`) and stored as ISO 8601 with the
  UTC offset (`2026-10-06T14:26:00+07:00`).
- Items are identified only by the platform `SKU ID`. Product name, variation and Seller SKU are
  display values that can change between two exports of the same day.
- Orders are identified by `Order ID`.
- Requirement IDs (`P3`, `U1`, `A2`) are stable references; code comments and tests may cite them.
- Where a design disagrees with [`../business-process/`](../business-process/README.md), reality
  wins and the design is fixed.
