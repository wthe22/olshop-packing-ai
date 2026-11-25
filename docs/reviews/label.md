# LABEL: the owner's feedback

The owner's findings on reading labels and slips, couriers, label groups and the summary PDF. Open items
only; see [README](README.md).

## R-LABEL-17 — Province, city and district of each order

**Present.** The receiver's area is printed on every label but not read.

**Wanted.** Each order records its province, city and district **as printed** (no normalising), in the
journal's order record and as three columns of `orders.csv`. Where it is printed (the study of the
samples, 1,312 orders):

| Layout | Where | Province | City | District |
|---|---|---|---|---|
| TikTok standard (J&T Express 85%, SiCepat 13%) | the line under `Penerima :`, `province,city,district` | 100% | 100% (J&T prints its delivery town) | 100% |
| Tokopedia boxed (IDX, J&T Cargo, Anteraja) | under `Penerima`, the line after the name: `province, city, district, street` | 9 of 9 | 9 of 9 | 9 of 9 |
| TikTok instant (Gojek, Grab) | the end of the address under `Alamat :`, `…, district, city, province`; lines wrap every 55 characters, inside a word too | 5 of 5 | 5 of 5 | 5 of 5 |
| Shopee-style (SPX, Grab Express) | city and district on their own line above `Berat:`; province at the end of the address, left of the `Pengirim:` column, wrapped | 8 of 8 | 8 of 8 | 7 of 8 |

Batches saved before get theirs from their stored inputs, by a migration the owner starts (R-APP-16): one
appended record per batch, never an edited one. It runs before an archive removes the PDFs.

## R-LABEL-18 — An instant order saved before shows its pickup code as its tracking ID

**Present.** An instant order saved before D-LABEL-24 has no tracking ID; its record holds the pickup code.

**Wanted.** The fold reads such an order's pickup code as its tracking ID, as for a new one: Search, the
filters, the Day screen and the views show it. No file is changed.
