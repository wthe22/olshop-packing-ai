# 01 — Packing Workflow

## Channels and couriers

- Orders come from TikTok Shop. Tokopedia orders appear in the same TikTok seller-centre export
  (`Purchase Channel = Tokopedia`, 8 of 1,436 lines on 2026-10-06).
- All orders are `Fulfillment by seller`, shipped from the shop's own location.
- Couriers on 2026-10-06 (orders): J&T Express 1,252 · SiCepat 174 · IDX 9 · J&T Cargo 1.

## Daily steps

1. **Arrange shipment** in the seller centre. Each order gets a tracking ID. The `RTS Time`
   values come in clusters of up to 50 orders, consistent with arranging 50 at a time.
2. **Download** the shipping-label PDF and **export** the orders CSV, on a PC or on Android
   *(owner)*. Label downloads come in files of at most 200 pages (samples: 200 + 200 + 162).
3. **Sort and group** on a PC *(owner)*:
   - Group orders with exactly the same packing list (same items, same quantities).
   - Split the groups by **category** — predefined rules on the items, reused every day.
     Example *(owner)*: one category for the side-stand cover ("sepatu"); one for items other
     than the side-stand cover whose name contains "spion" or "knalpot"; one for the
     remaining motorcycle-related items.
   - Print one A4 packing list for the batch.
   - Print the labels one group at a time, largest group first. The printer stops at the end of
     each group; the labels are torn off and the next group is printed. Label pages are printed
     exactly as downloaded.
4. **Pack**: the labels and packing list go to the warehouse team, who pack the parcels.
5. **Check** on Android with the phone camera *(owner)*. Some items cannot be checked once
   packed; the quantity is easy to check. When a stack is known to hold one item type with
   quantity 1, anything else scanned in that stack is an error. Each parcel ends up:
   - **checked**;
   - **wrong packing** → repacked, then checked again;
   - **label lost or damaged** → label reprinted at the PC, then checked again;
   - **pending** → set aside, e.g. no stock.
6. **Repeat during the day**: new orders arrive; steps 1–5 run again as batch 2, 3, … The new
   orders CSV contains all orders still waiting, so it is compared with the earlier batches to
   find the new ones.
7. **Reset**: every day starts from zero *(owner)*. One device does the checking; work moves to
   another device by exporting and importing the day's session *(owner)*.

## Which orders need packing

- An order in the "to ship" export (`Perlu dikirim` / `Menunggu pengambilan`) must be sent
  *(owner)*.
- This includes orders made ready on earlier days and not yet picked up: the first export of
  2026-10-06 held 204 of them (49 from 2026-10-03, 155 from 2026-10-05).
- An order that is cancelled, or picked up by the courier, disappears from the export. Neither
  needs the packing team's attention *(owner)*. Two orders of the 14:26 export were gone at 16:29.

## Volumes on 2026-10-06

| | Batch 1 (14:26 export) | Batch 2 (16:29 export) |
|---|---|---|
| Orders in export | 956 | 1,435 |
| New orders | 956 | 481 |
| Orders gone since previous export | — | 2 |
| Orders whose items, quantities or tracking ID changed | — | 0 |

- 49 distinct packing lists in the 16:29 export; 23 of them occur only once.
- The side-stand cover is in 1,275 of 1,435 orders (89 %). Its single-product groups:
  ×1 957 orders, ×2 247, ×3 50, ×4 19.
- 45 SKUs from 20 products. One order has quantity 98 (rubber grinding bits).
- One order holds two different products (an exhaust cover and a mirror).
- 4 orders have a buyer message; some state the variant wanted (size, brand).
- One SKU was renamed between the 14:26 and 16:29 exports. Names, variations and Seller SKUs
  can change; only the platform `SKU ID` is stable *(owner)*.
