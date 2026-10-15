# New batch

Choose the label files of this batch, then read them.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  New batch 3                                                            [Cancel]       │
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
  earlier batch of the day shows *used in batch n* in orange: it can still be read, its orders are
  then found as already saved and left out. **×** takes a file off the list; a file added twice
  is listed once.
- Files are read in the order of the list, sorted by file name (= download time and part
  number). **Read labels** needs at least one file.

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
