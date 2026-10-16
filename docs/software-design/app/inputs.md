# Inputs

| Input | Content |
|---|---|
| Label PDFs | The *Shipping label + Packing slip* files of one batch ([business-process 02](../../business-process/02-data-sources.md)). One or more files; the seller centre splits a download into files of at most 200 pages |
| `categories.toml` | In the data folder. Each entry is one pick ([rules file](../rules-file/README.md)). Edited inside the app ([Categories screen](../ui/categories.md)) or by hand |
| `couriers.toml` | The courier-deduction data ([D-LABEL-1](../decisions/label.md)). **In the data folder**, like `categories.toml`: written from the bundled file (same content as `tools/analysis/packing/couriers.toml`) on first start, then the owner's; a broken file stops the batch with its line. Fields and an example below |
| The day's `state.json` | What earlier batches of the same day saved (below) |

## `couriers.toml`

The courier list, one `[[courier]]` entry each:

| Field | Meaning |
|---|---|
| `name` | The name written into the order and used by a `courier` condition |
| `logo` | The courier's logo images, each as **XXH3-64** of the image's **encoded bytes** (16 hex characters, seed 0). A list, because a courier can have more than one (an asset the platform re-made) |
| `shape` | The courier's logos by **appearance**, each as **XXH3-64** of a 64 × 64 1-bit canvas (XXH3-64 of the bytes of a 64×64 image composited onto white with its mask, fitted to the canvas and thresholded at 200/255, seed 0). The clue for a **masked** logo, whose encoded bytes name nothing and are re-rastered between exports (J&T Cargo); a list like `logo`. Tried only when no `logo` entry matched, and a hit asks the owner nothing extra |
| `tracking` | The tracking-ID form: a token (or a match in the whitespace-collapsed text) must match it fully, and it need not be repeated. The form gives the *candidate* couriers; the courier is named when the logo's (or text clue's) courier is among them |
| `text` | A brand line or web address printed on the label (`www.jet.co.id`, `FastTrack`), used only where no logo is known |

```toml
[[courier]]
name = "J&T Express"
logo = ["39b9b7b9f61d0e7f"]      # 975x377 px, 14,351 bytes, the same asset in every sample file
tracking = "JY\\d{10}"
text = ["jet.co.id"]

[[courier]]
name = "SiCepat"
logo = ["cf33ed60daa291b1"]      # 184x60 px, 10,636 bytes
tracking = "00\\d{10}"

[[courier]]
name = "J&T Cargo"
logo = []                        # its base image is one flat colour: no name in those bytes
shape = ["753ccbd14749befa"]     # the masked wordmark, one value for both rasters (680x156, 683x157)
tracking = "57\\d{10}"
text = ["FastTrack"]             # its header strip and its order-ID caption `TT Order ID`

[[courier]]
name = "IDX"
logo = ["9131c1f94ad363d4"]      # 1200x859 px, 9,810 bytes
tracking = "TKP\\d{10}"

[[courier]]
name = "SPX"
logo = ["4583097ef370a670"]      # 68x27 px, 1,495 bytes, masked: the small red SPX wordmark
tracking = "SPXID\\d{11}"        # the Shopee-style label; the ID carries a check letter
```

The `SPX` entry is the one the 2026-10-15 samples measured ([Courier deduction](../labels/courier.md));
the name stored is the owner's answer in the prompt — the wordmark reads `SPX`.

The app also writes this file. A logo the owner names in the prompt ([the logo prompt](../ui/new-batch.md)) is appended to the
`logo` list of the named courier, and an image answered **Not a courier** (a buying channel's strip,
[Q-LABEL-5](../questions/label.md)) is appended to a top-level `not_courier` list of hashes. The app
therefore never asks about the same logo again ([D-LABEL-4](../decisions/label.md)). Comments and the order
of the file are kept, as for `categories.toml`.

**Open question:** whether the remembered logos live in this file or in a file of their own —
[Q-LABEL-7](../questions/label.md).

## Every page must carry a packing slip

The packing slip is the only source of item data (product name, variation, Seller SKU,
quantity). A page that is not a continuation page and has no slip table stops the batch:

> **These labels have no packing slip.** 200 of 200 pages are plain shipping labels. In the
> seller centre, download the labels again with **Shipping label + Packing slip**.

Nothing is written. If only some pages lack the slip, the message names the file and the page
numbers.

## Fields the conditions can use

Only fields with a source on the label or slip:

| Field | Level | Source |
|---|---|---|
| `name`, `display_name`, `variation`, `seller_sku`, `line_quantity` | item | slip |
| `total_quantity`, `distinct_items` | order | computed from the slip |
| `tracking_id`, `courier`, `ship_by` | order | label |

A condition that uses a CSV field (`sku_id`, `product_category`, `channel`, `paid_time`,
`rts_time`, `created_time`) is an error in the rules check, with line and column:

> `line 18, col 8: field "product_category" needs the orders CSV, which the PC app does not read`

The app scan-filter fields (`category`, `batch`, `group`) stay errors as in
[conditions](../rules-file/conditions.md).
