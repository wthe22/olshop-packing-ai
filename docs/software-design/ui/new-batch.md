# New batch

Choose the label files of this batch and the grouping of picks it uses, then read them.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  New batch 3   Grouping: [default ▾]                                    [Cancel]       │
│                                                                                        │
│  ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐       │
│     Drop label PDFs here, or  [Add files…]                                             │
│  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘       │
│                                                                                        │
│  10-07_11-02-10_Shipping label+Packing slip_1.pdf 200 pages 11:02                  [×] │
│  10-07_11-02-21_Shipping label+Packing slip_2.pdf  57 pages 11:02                  [×] │
│  10-07_09-05-49_Shipping label+Packing slip_2.pdf  41 pages 09:05  used in batch 2 [×] │
│                                                                                        │
│  Selected: 3 files · 298 pages                                         [ Read labels ] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Add files…** opens the Windows file dialog (several files at once; it opens in the folder
  used last time). Dropping files onto the window does the same.
- Each listed file shows its page count (quick: only the page count is read). A file read in an
  earlier batch of the session shows *used in batch n* in orange: it can still be read, its orders are
  then found as already saved and left out. **×** takes a file off the list; a file added twice
  is listed once.
- Files are read in the order of the list, sorted by file name (= download time and part
  number). **Read labels** needs at least one file.
- A file whose bytes equal another listed file shows **same file as `n`** at once, before any read
  (the file fingerprint, [data-folder](../app/data-folder.md)); **×** takes it off. After a read, a
  file that holds an order which appears more than once in the batch is marked with its group chip
  (▌G1) and a **Review the repeats** button opens [Repeated orders](duplicates.md#the-file-list-marks-the-files).
- **Read labels** opens the Plan screen ([Plan](plan.md)), or [Repeated orders](duplicates.md) first
  when the batch holds a repeat. The top **Cancel** leaves the batch
  and returns to the Day screen, discarding the draft; **Back** on the Plan screen returns here
  with these files still listed, and **Read labels** then reads them again from the start (the
  Plan changes are lost).

## The grouping

The picks of the batch come from one **grouping** of categories
([the rules file](../rules-file/README.md)), chosen here before **Read labels**:

- **Grouping** lists the groupings of the data folder: `default` always, plus every grouping file in its
  `categories/` folder. The last-used grouping is preselected, so a shop that always uses one grouping never
  touches the control. The choice is remembered as the last used and written into the batch at save
  (`state.json`), so **Amend** reads the batch's own grouping again
  ([revert and amend](../app/batch.md)).
- The chosen grouping's categories are applied in order as the plan ([Plan](plan.md)). A grouping with no file
  is the one catch-all entry, so every order of the batch goes into one PDF set
  ([inputs](../app/inputs.md), [Q-UI-4](../questions/ui.md)).
- Which grouping a batch uses is an open choice of the owner ([Q-RULES-1](../questions/rules.md)); this
  control is its default — chosen per batch, the last-used grouping preselected — and is built until the
  owner decides.

## The amend's selection page

**Back** on the amend's Plan screen ([Plan](plan.md)) opens this page, titled *Redo batch 2*, so the
batch's label PDFs can be changed ([Amend last batch](../app/batch.md)):
```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Redo batch 2   Grouping: default (fixed)                               [Cancel]       │
│  ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐       │
│     Drop label PDFs here, or  [Add files…]                                             │
│  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘       │
│  10-07_11-02-10_Shipping label+Packing slip_1.pdf 200 pages 11:02                  [×] │
│  10-07_11-02-21_Shipping label+Packing slip_2.pdf  57 pages 11:02                  [×] │
│                                                                                        │
│  Selected: 2 files · 257 pages                                         [ Read labels ] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```
- The current selection is listed: the batch's own PDFs first, from its `download/` copy
  ([day folder](../app/data-folder.md)), in the order stored in `state.json`, then what the owner
  chose here before.
- The batch's own grouping is shown in the title line and is not chosen here: an amend reads the batch's
  grouping again, whatever grouping was used last (the batch's `state.json` holds it), so *Redo batch 2* brings
  back the picks the batch had ([The grouping](#the-grouping), [revert and amend](../app/batch.md)).
- **Add files…** and a drop add PDFs ([Q-APP-2](../questions/app.md), default: any PDF on the PC, as
  in a new batch); an added file joins the list after the batch's own files. **×** takes a PDF off the
  list; its orders are then not in the batch
  ([Q-APP-3](../questions/app.md), default: they stay unsaved and the next batch reads them again).
- **Read labels** reads the selection as an amend and opens the Plan screen again (the plan changes
  made before **Back** are lost); **Back** from that Plan screen returns here with the files still
  listed.
- **Cancel** returns to the Day screen and writes nothing, and never asks: an amend writes nothing
  before **Save batch 2 again**.

## Reading

```
│  Reading labels …  page 143 of 257 (file 1 of 2)                                       │
│  ███████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░         [Cancel]    │
```

Cancel stops reading and returns to the file list; nothing is written. When reading ends, the
Plan screen opens — or, if the batch cannot be used, a stop message (see *Messages*) with a
**Back** button.

## The logo prompt

A label whose logo the app has never seen is asked about once per unseen logo, before the Plan
screen — of a page's images, the ones drawn in its **top 40 pt** are looked at ([Q-LABEL-5](../questions/label.md)), so the frame,
the barcode and the QR never ask. The picture is the logo as printed, so it can be recognised, and
the count says how many pages carry it:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  Which courier is this logo?                                                           │
│                                                                                        │
│    [ SICEPAT EKSPRES ]                                 4 pages · 4 orders               │
│                                                                                        │
│    [J&T Express]  [SiCepat]  [J&T Cargo]  [IDX]        [I don't know]  [Skip all]      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- The buttons are the names in `couriers.toml`. Each distinct unseen logo is asked about once per
  batch.
- **A courier name** names those orders at once, and the app remembers the logo's hash for that
  courier, so the logo is never asked about again ([D-LABEL-4](../decisions/label.md); where it is
  kept: [Q-LABEL-7](../questions/label.md)).
- **Not a courier** ([Q-LABEL-5](../questions/label.md)) answers for an image that is not a
  courier's logo — a buying channel's strip (`tokopedia | TikTok`, `tokopedia | Shop`,
  `TikTok Shop`, `tokopedia PLUS`, `Shopee`): its hash is remembered the same way, and that
  image is never asked about again, so a new channel costs one answer,
  not one per batch.
- **I don't know** writes nothing: those orders save with an empty courier and appear in the day's
  warning, and the logo is asked about again in the next batch.
- **Skip all** stops the prompts for the rest of **this batch**: every further unseen logo in it
  is treated as *I don't know*, and nothing is written. The next batch asks again from the start.
- Closing the prompt (Esc) skips only that one logo: the next unseen logo still asks. It never
  blocks the work — the batch saves either way.
