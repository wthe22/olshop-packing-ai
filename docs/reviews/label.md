# LABEL: the owner's feedback

The owner's findings on reading labels and slips, couriers, picks and the packing list. Open items
only; see [README](README.md).

## R-LABEL-1 — The same order twice in a batch is not detected

**Present.** Two labels with the same Order ID in one batch are read as one order whose item lines and
quantities are added twice and whose pages are written twice. The only sign is a *Slip total differs*
warning on a slip that prints its total.

**Wanted.** Duplicate orders within the same batch are detected and counted.

**Design written, waits for the owner.** [Repeated orders: how they are found](../software-design/app/duplicates.md);
choices in [Q-LABEL-9, Q-LABEL-10](../software-design/questions/duplicates.md).
