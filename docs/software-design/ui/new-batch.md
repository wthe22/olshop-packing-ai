# New batch

Choose the label files of this batch and the sort rule it uses, then read them.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  New batch 3   Sort rule: [default ▾]                                   [Cancel]       │
│  ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐       │
│     Drop label PDFs here, or  [Add files…]                                             │
│  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘       │
│  1  10-07_11-02-10_Shipping label+Packing slip_1.pdf 200 pages 11:02               [×] │
│  2  10-07_11-02-21_Shipping label+Packing slip_2.pdf  57 pages 11:02               [×] │
│  3  10-07_09-05-49_Shipping label+Packing slip_2.pdf  41 pages 09:05 used in batch 2[×]│
│     labels (1).pdf is the same file as 2: listed once.                                 │
│  Selected: 3 files · 298 pages                                         [ Read labels ] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Add files…** opens the Windows file dialog (several files at once, in the folder used last). Dropping
  files onto the window does the same.
- Each listed file shows its page count; its SHA-256 is taken at once
  ([02 Read](../workflow/02-read.md#3-choose-the-files)). A file whose bytes equal a listed file is not listed
  again: a line names both. A file that is an input of a live batch of this date shows *used in batch n*
  in orange: it can be read, its orders are then found as already saved. **×** takes a file off the list.
- Files are read in the order of the list, sorted by file name (= download time and part number). **Read
  labels** needs at least one file.
- After a read, a file that holds an order appearing more than once is marked with its group chip (▌G1) and
  **Review the repeats** opens [Repeated orders](duplicates.md).
- **Read labels** opens the Plan screen ([Plan](plan.md)), or Repeated orders first when the batch holds a
  repeat. **Cancel** leaves the batch and returns to the Day screen; **Back** on the Plan screen returns here
  with the files still listed, and **Read labels** reads them again from the start (plan changes are lost).

## The sort rule

**Sort rule** lists the rules of `packing-workflows.toml` ([rules file](../rules-file/README.md)). The rule
of the owner's newest batch is preselected, so an owner who always uses one rule never touches it
([D-RULES-1](../decisions/rules.md)). The batch's record keeps a snapshot of the rule it was saved with.

## The amend's file list

**Back** on an amend's Plan screen opens this page, titled *Redo batch 2*, so the batch's files can change:

- The batch's own inputs are listed first, from `input/`, in their recorded order, then what the owner added.
- The sort rule is shown, not chosen: *Sort rule: default (as now)*, or *default (as saved)* when no rule
  has that name any more ([D-RULES-2](../decisions/rules.md)).
- **Add files…** and a drop add any PDF on the PC; **×** takes a file off, and its orders stay unsaved for a
  later batch ([D-APP-19](../decisions/app-features.md)).
- **Read labels** reads the list as an amend and opens the Plan screen again. **Cancel** returns to the Day
  screen and writes nothing.

## Reading

```
│  Reading labels …  page 143 of 257 (file 1 of 2)                                       │
│  ███████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░         [Cancel]    │
```

**Cancel** stops reading and returns to the file list; nothing is written. A batch that cannot be used ends
in a stop message ([Messages](messages.md)) with **Back**.

## The logo prompt

A header image the app has never seen is asked about once, before the Plan screen. Only images in the
page's top 40 pt are asked about ([D-LABEL-9](../decisions/label.md)), so the frame, the barcodes and the
QR never are. The picture is the image as printed, and the count says how many pages carry it:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Which courier is this logo?                                                           │
│    [ SICEPAT EKSPRES ]                                 4 pages · 4 orders               │
│    [J&T Express] [SiCepat] [J&T Cargo] [IDX] [SPX]   [Not a courier] [I don't know] [Skip all] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **A courier name** (the couriers of `packing-workflows.toml`) names those orders at once, and the hash is
  added to that courier's `logos` ([D-LABEL-10](../decisions/label.md)): it is never asked about again.
- **Not a courier** answers for a buying channel's strip (`tokopedia | TikTok`, `TikTok Shop`, `Shopee`, …):
  the hash goes to `not_courier.logos` and the image is never asked about again.
- **I don't know** writes nothing: those orders save with no courier and are in the warning, and the image
  is asked about again in the next batch. **Skip all** answers *I don't know* for the rest of this batch.
  Esc skips that one image. The prompt never blocks the work.
- An instant-service label's **tag** that names no courier is asked the same way, *Which courier serves
  instant labels tagged `GRB`?*, without *Not a courier*; the answer goes to that courier's `instant_tags`
  ([D-LABEL-7](../decisions/label.md)).
