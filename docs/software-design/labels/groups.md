# Label groups and group PDFs

## The plan

- A batch is sorted by the **sort rule** chosen for it ([rules file](../rules-file/README.md)). Each of its
  **label group rules**, in file order, takes the orders its `filter` matches **from what is left** after
  the groups before it; a rule with no `filter` takes every order left, so it must be last.
- **The rest**: when the last rule has a filter, or is skipped for this batch, the orders left form one
  more group, code `?`, name *Uncategorised*.
- A group that takes no order gets no file.
- A sort rule whose only group has no filter puts every order into one group PDF; the Sort rules and Plan
  screens then say *No groups yet: all orders go into one PDF* ([D-UI-4](../decisions/ui.md)).

## A group PDF

- **Runs**: inside a group PDF, orders with the same **packing key** — their lines as (product name,
  variation, quantity), sorted — are consecutive ([D-APP-20](../decisions/app-features.md)). The Seller SKU and the
  printed `Qty Total` are kept with the order's content ([records](../data/records.md)) but do not split a
  run. Runs are sorted by number of orders (most first), then by packing key as text in line order, name
  first (upper case before lower case; a quantity compares as a number). Inside a run the orders keep their
  **reading position**: the input's place in the batch, then the page.
- **Pages**: every page of each order's written copy, unchanged and in its order.
- **Run numbers** count `01`, `02`, … in printing order within the group PDF and are written
  `<batch>-<code>-<run>`: `2-A-05` is batch 2, group A, run 05.

## File names

- **Group PDF**: `{batch}-{code} {name} x{count}.pdf`, e.g. `2-A Sepatu x88.pdf`; `count` is the group's
  orders. **Summary PDF**: `{batch} summary.pdf` ([summary PDF](summary-pdf.md)). **Reprint**:
  `reprint {n} x{count}.pdf` ([reprint](../app/reprint.md#reprint)).
- A suggested name follows the default file-name rule ([data](../data/README.md#types-versions-and-names)):
  `Spion & Knalpot` gives `2-B Spion Knalpot x35.pdf`. The batch number in front makes *sort by name*
  printing order. The rest group `?` has no code in its name, since Windows does not allow `?`:
  `2 Uncategorised x12.pdf`.
- The owner may change a group PDF's name on the Plan screen; it must be a valid Windows file name and unique
  in the batch (upper and lower case count as the same).

## The already-saved guard

An order saved in a live batch of the same namespace and date is left out of a new batch, all its copies
recorded as left out with the batch that saved it, and listed in a warning with its tracking ID, batch,
group and time ([03 Plan](../workflow/03-plan.md#7-orders-already-saved)).
