# A workspace's shops; Print in the app

The owner's words in chat (2026-11-05), written into the design as
[D-APP-25](../../software-design/decisions/app-features.md) and
[D-APP-26](../../software-design/decisions/app-features.md). Two tasks, each its own commit; either may go
first. [code-rules.md](../code-rules.md) is binding for every step and every subagent; each report states
*size gate passed, no new lint suppression, coverage not lower*. Tests: known-answer fixtures through
read → plan → save → verify as the backbone, tables for fixed truths, broken outputs the verifier must catch.

## S1. The shop check

Read first: [workspaces › Its shops](../../software-design/app/workspaces.md#its-shops),
[reading › The sender](../../software-design/labels/reading.md#the-sender),
[02 Read step 6](../../software-design/workflow/02-read.md),
[New batch › The shop prompt](../../software-design/ui/new-batch.md#the-shop-prompt),
[Settings](../../software-design/ui/settings.md), [program](../../software-design/app/program.md).

| Step | What | Done when |
|---|---|---|
| S1.1 | `packing-fixtures` prints a `Pengirim` caption and a made-up shop name in each of the four places of the sender table, one fixture file of another shop and one page with no caption | The fixtures and their answer are written by `cargo run -p packing-fixtures` |
| S1.2 | The engine: the sender text of a copy's first page from `PageText` (its line and the runs beside and under the caption), and the match against a list of names | A table test per layout, with stray spaces and a digit stuck to the name; *no sender* on a page with no caption |
| S1.3 | `crates/store`: `{workspace}/settings.toml` read and written under the lock; `read_labels` returns, per file, the orders not of a listed shop and the workspace that lists the sender found | Known-answer test: the other shop's fixture is named per file; a workspace with no file checks nothing |
| S1.4 | The window: the shop prompt after the logo prompt (**Back** the default), Settings › Workspaces › **Shops** | Checked on screenshots of the built program in both themes, beside the logo prompt and Settings ([UI consistency](../../software-design/decisions/ui.md)) |

- Before S1.2, run the reading over `samples/` locally and check that every copy of the owner's files finds
  the sender (the slip-only download `10-26_…_slip.pdf` prints no `Pengirim` on its first page: find out
  whether its copies start on a page that does). Nothing of `samples/` goes into a fixture or a commit.

## P1. Print, Sent and Printed

Read first: [Day](../../software-design/ui/day.md),
[05 After the save › 16](../../software-design/workflow/05-after-save.md#16-print-sent-and-printed),
[journal](../../software-design/data/journal.md), [records](../../software-design/data/records.md),
[views](../../software-design/data/views.md), [program](../../software-design/app/program.md).

| Step | What | Done when |
|---|---|---|
| P1.1 | Spike, kept out of `main`: a second `WebviewWindow` showing an output through the asset protocol (WebView2's PDF viewer), and `ICoreWebView2_16::ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_BROWSER)` through `with_webview` (`webview2-com`) once the PDF has loaded | The preview opens on one click with Chrome's options; it is known whether WebView2 reports Print versus Cancel (if it does, *Sent* follows the real print and the design is changed in the same commit) |
| P1.2 | The `print_sent` record: engine, the fold's rule, the views' `is_marked_as_sent`, `crates/batch` | Fold table test; save → mark sent → revert/amend clears it; reopen shows the same marks |
| P1.3 | `print_output` and `set_sent`; the Day screen's **Print** (group PDFs, summary PDFs, reprints), *Sent hh:mm* ✕ and the tick | App tests; checked on screenshots of the built program in both themes and at 125 % |

- An older copy of the program cannot read a journal that holds `print_sent`: the handoff tells the owner
  to update every copy on a shared data folder together. Check [D-APP-3](../../software-design/decisions/app.md)
  for what a new record type means for the journal's `schema`.
- The version is raised with the first of the two tasks to land (`0.4.0`), in its one place
  ([Settings › The version](../../software-design/ui/settings.md#the-version)).
