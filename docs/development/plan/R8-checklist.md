# The owner's checklist for v0.2

The owner tests the program folder `target/portable/Packing/` (copy it out first, or use it in place)
with real downloads, in this order. A finding goes into [reviews/ui.md](../../reviews/ui.md) or
[reviews/app.md](../../reviews/app.md) as `R-<SCOPE>-n`: what was done, what happened, what was expected.

1. **Start.** On a fresh copy: *Where should the data be kept?* → **Use the folder next to the
   program**, then name the first namespace. The Day screen of today opens; the panel lists the
   namespace and today. A data folder of v0.1 shows the line *This folder also holds data of Packing
   v0.1* and is not changed.
2. **Panel.** **+ New namespace**, switch between the two, open an older date and today again; **«**
   and `Ctrl+B` hide and show it; **⋯ → Open folder** on a date that has batches.
3. **New batch.** **Add files…** and a drop; *used in batch n* on a file read before; **×**; the sort
   rule preselected; **Read labels** with its progress and **Cancel**.
4. **Logo prompt.** An unknown logo asks *Which courier is this logo?*; a courier answer and *Not a
   courier* are remembered in `packing-workflows.toml`; *I don't know*, Esc and *Skip all*.
5. **Repeated orders.** Two overlapping downloads: the screen opens before the plan; *Keep the first
   copies*, *Keep the last copies*, *Take file n out* (or the reason it cannot), a copy chosen by hand;
   **Continue to the plan**; **Back** shows the group chips on the file list.
6. **Plan.** The header's *copies left out ▸* and **Change…**; a group's order contents; **Use**,
   **↑ ↓**, a file name changed; **+ Add a group for this batch**; **Edit sort rule**; **Save batch**
   with its four stages; the checker's dialog if it finds something.
7. **Print.** Open each group PDF and the summary PDF; the pages are the downloaded labels, in the
   order of the summary; the summary's counts add up.
8. **Day.** The printed tick survives a restart; **Left out ▸**; **Amend** (Plan titled *Redo batch n*,
   **Back** to its files, **Cancel amend**) and **Revert** on the newest batch. Delete a group PDF in
   Explorer, reopen the date: *⚠ missing* and **Restore**.
9. **Search and reprint.** By tracking ID on *This day* and on *All days*, by product name and quantity;
   **▸** shows where an order came from; tick two orders, **Reprint 2 orders**, **Open**; today's Day
   screen lists the reprint.
10. **Sort rules.** **+ New rule** (empty and as a copy), **Rename…**, **Delete rule**; a group's
    filter in boxes and in the text view; the counts *In batch n* with a batch open; **Save**, then
    `packing-workflows.toml` keeps its comments; **Edit file as text**.
11. **Settings.** The data folder, **Change…**, *About* reads *Version v0.2*.
12. **The words.** Whether *namespace*, *label group*, *order content*, *group PDF* and *summary PDF*
    read well on the screens and the printed summary ([D-LABEL-15](../../software-design/decisions/label.md)).
