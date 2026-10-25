# Open questions: labels

## Q-LABEL-11 — What to call the orders with the same items inside a group PDF

**Words.** A label group is saved as one PDF. Inside it, orders whose items are the same (product name,
variation, quantity) are printed next to each other, so their labels come off the printer together
([groups](../labels/groups.md#a-group-pdf)). The design calls such a block a **run** and numbers it `2-A-05`
(batch, group, block). The number is printed nowhere: it shows only on the Plan screen and in Search. *Run*
is not clear to a first-time user.

**Example.** Group A *Sepatu* of batch 2 holds 122 orders: 96 with *Sepatu ×1*, 26 with *Sepatu ×2*. The Plan
screen lists the two blocks under the group.

| Option | What the owner sees |
|---|---|
| **1. Order content** (**Default**) | `▸ 2-A Sepatu — 2 order contents: Sepatu ×1 · 96 orders, Sepatu ×2 · 26 orders`. The owner's own word, and the data already says so (`order_contents`, `order-contents.toml`). To make one order content exactly one block, the Seller SKU and the printed `Qty Total` move from the content to the order: two orders that differ only in Seller SKU share one order content |
| 2. Packing list | `… 2 packing lists: …`. The owner's word in the draft's business workflow ("sort labels by packing list", "packing lists that appear often are pre-packed"); free again now that the A4 sheet is the summary PDF. The v0.1 app used it for the A4 sheet |
| 3. Stack | `… 2 stacks: …`. What the printed labels become on the table; the plainest for a new user, less exact as a data word |
| 4. Item set | `… 2 item sets: …`. Says what is shared; a new word |

With every option the `2-A-05` number goes: the blocks are listed in printing order under their group, and
nothing printed needs the number.

**What the answer changes:** the word on the Plan, Day and Search screens and in the design; with option 1,
where the Seller SKU and the printed total are stored ([records](../data/records.md), [views](../data/views.md)).
