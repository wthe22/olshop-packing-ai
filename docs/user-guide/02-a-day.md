# A day's batches

A batch is one round of the day: you download the labels in the seller centre, prepare them in the
app, then print.

## A normal batch, step by step

### The Day screen

![The Day screen](images/02-day.png)

The **Day** screen shows what has been saved today and is the way to the next batch.

- The top line counts today: "Today: 2 batches · 729 orders · 5 saved PDFs".
- **+ New batch** starts a new batch.
- Each batch is a card: "Batch 1 · saved 07:40 · 3 files · 600 orders", with one line per saved
  PDF (number, pick code and name, orders, runs) and **Open** to open it in your PDF viewer.
- **Open folder** opens the batch folder in Explorer.
- On the newest batch only: **Amend** and **Revert**
  ([Revert and amend the last batch](03-changes.md#revert-and-amend-the-last-batch)).
- A day with nothing saved shows "Nothing saved on this day yet."

The date in the top bar is the day everything is saved to. ◀ ▶ step a day; **Today** goes back to
today (handy for a download just after midnight).

### New batch

![New batch with one file](images/03-new-batch.png)

1. Click **+ New batch**.
2. Add the label files: click **Add files…** and choose the *Shipping label + Packing slip* PDFs
   of this download, or drop them onto the window ("Drop label PDFs here, or Add files…").
   - Each file shows its page count and time. A file already used earlier today shows
     *used in batch n* in orange.
   - **×** removes a file from the list.
3. Click **Read labels**.

### Reading

![Reading the labels](images/04-reading.png)

The app reads the files page by page: "Reading labels …  page 143 of 257 (file 1 of 2)" with a
progress bar. **Cancel** stops and returns to the file list; nothing is written. When it is done
the **Plan** screen opens.

### Plan

![The Plan screen](images/05-plan.png)

The Plan screen is the heart of the app: which orders go into which saved PDF. It opens with your
categories applied in order.

- The header: "Batch 3 · 2 files · 257 pages · 256 orders · 0 already saved".
- **Rows** are the picks in order. Each takes its orders from the rows above.
- **Use** (the tick) turns a pick off for this batch only; its orders fall to the rows below. The
  rules file is not changed.
- **↑ ↓** move a pick. **▸** shows the runs of a pick — what the packing list will show:

![The runs of a pick](images/06-plan-runs.png)

- **File name**: the suggested name is `<n> <code> <name> ×<orders>`; you can change the whole
  name. Two saved PDFs of one batch cannot have the same name (the second shows "Same name as
  PDF 6").
- **+ Add a pick for this batch** adds a one-off pick (it is not saved to your categories).
- **Edit categories** opens your categories
  ([Editing your categories](03-changes.md#editing-your-categories)).
- **Packing list**: choose *one per PDF*, *one for the batch* (default) or *none*, and the layouts
  *full*, *summary*, *pick* (default *pick*).
- **Save batch** writes everything and returns to the Day screen. **Cancel** asks
  "Discard batch 3? Nothing has been saved."

### After saving

![The Day screen after saving](images/08-day-saved.png)

The Day screen shows the new batch on top with a green line "Batch 3 saved: 3 PDFs, 1 packing
list." Click **Open** on each saved PDF and on the packing list, and print them.

Everything is written into the program folder under `labels\<day>\batch <n>\`: the saved PDFs, the
packing list, a `summary.txt`, and a `download\` copy of the files you read (so Amend works even if
the originals are gone).

## Several batches a day

Do the same for each download. The saved-PDF numbers continue through the day (batch 2 starts
after batch 1's last number), so "sort by name" inside a batch folder is printing order. A file
you read in an earlier batch today is marked *used in batch n*; if you read it again, its orders
are found as already saved and left out (a warning lists them).
