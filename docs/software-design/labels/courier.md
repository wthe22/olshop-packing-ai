# Courier deduction

The label is read for two clues and the courier is named **only when the two agree** (the
owner's rule): one clue alone can be wrong, so a wrong name would need two labels to be wrong in
the same way.

1. **The logo** — the main clue. Every label prints the courier's logo as an embedded image; the
   image's **encoded bytes** are hashed (pdfium's `FPDFImageObj_GetImageDataRaw`, *not* the
   decoded pixels) with **XXH3-64** and looked up in the courier data's logo table. One hash per
   courier, measured stable across every sample export — J&T Express `39b9b7b9f61d0e7f`, SiCepat
   `cf33ed60daa291b1`, iD Express (IDX) `9131c1f94ad363d4`, SPX Express `4583097ef370a670` (the
   Shopee-style label; identical on all 3 of its pages, one file so far). The cost is about 0.5 µs
   a page (the bytes are 1.5–15 KB; measured 0.44 µs for a 14 KB logo). Python's `xxhash` (in
   `tools/analysis`) and Rust's `xxhash-rust` compute the same value, so a hash taken there can be
   checked against the app's. A hash not in the table is **asked of the owner once** ("which
   courier is this logo?") and the answer is written into the data file, so later labels are named
   without asking. **Which** of a page's images is the logo needs a rule of its own
   ([Q-LABEL-5](../questions/label.md)): a page can carry, besides the courier's logo, its buying
   channel's brand strip, a page-wide frame, the slip's footer strip and — on the Shopee-style
   label — its barcode and QR as images too, and none of those is the courier's. The logo is not
   always the smallest image (on the Shopee-style label the channel strip is larger, on the boxed
   layouts the `tokopedia | Shop` strip is smaller) and not always the unmasked one (J&T Cargo's
   and SPX's logos both carry a mask). **Default** ([Q-LABEL-5](../questions/label.md)): the
   candidates are the images drawn in the page's **top 40 pt** — on all six sample styles those
   are exactly the courier's logo and the channel strip, and the frame (134–418 pt), the slip's
   footer strip (39–62, or 104–124 on an instant label) and the barcode and QR (173–224, 330–373)
   all fall outside — and a candidate whose hash the table does not know is **asked of the owner
   once**, the prompt gaining a **Not a courier** answer for a channel strip
   ([the logo prompt](../ui/new-batch.md)).
2. **The tracking-ID form** ([business-process 02](../../business-process/02-data-sources.md)):
   `JY` + 10 digits → J&T Express; `00` + 10 digits → SiCepat; `57` + 10 digits → J&T Cargo;
   `TKP` + 10 digits → iD Express (IDX); `SPXID` + 11 digits → SPX Express (the Shopee-style
   label, which prints a check letter around the number: `Resi:` without it, `SPXID…A` eight times
   on the page with it). The form must match a token (or a match in the whitespace-collapsed text)
   **fully**; it does not have to be repeated — on the boxed layouts the tracking ID is printed
   once ([business-process 02](../../business-process/02-data-sources.md)).
3. **A masked logo is hashed by appearance.** A courier can print its logo as an image whose
   shape is a `/SMask` and whose base is one flat colour (J&T Cargo: 331 bytes for 680×156): the
   base's bytes name no courier, and the platform re-rastered the same wordmark at two sizes
   (680×156, 683×157), so the encoded bytes differ between exports. For an image with no hash in
   the table, the app hashes its **appearance** instead: composite the image with its `/SMask` (if
   any), fit it into a fixed **64 × 64** canvas on white, threshold at **200/255**, and hash that
   1-bit canvas with XXH3-64. Measured over the samples: one value per courier asset, and J&T
   Cargo's two rasters both give `2a36acec58f5…` — so the appearance table needs one entry per
   courier, and a re-raster does not need the owner asked again. The appearance of the three
   plain-image logos is stable too (`31643d0eb0f1…` J&T Express, `3cf94ea34e54…` SiCepat,
   `dc99f0056376…` iD Express), but the encoded bytes are cheaper and are tried first.
4. **Named only when the two clues agree**: the tracking form gives the *candidate* couriers, and
   the courier is named when the logo's courier (by its bytes, else by its appearance) is among
   them. A logo whose courier and form disagree, a logo with no form, a form with no logo, or
   neither → no name. Unknown → `courier` empty, and a warning lists those orders.
5. **A printed text clue** (`www.jet.co.id`, `FastTrack`) stands in for the logo only where the
   logo cannot be read at all. J&T Cargo's label prints `FastTrack` in its header strip and
   `TT Order ID` as the caption of its order ID, and SiCepat and iD Express print no courier name.
6. **The service tag is not used** (the owner: it is weak for some couriers). The measured tag
   lists stay in [business-process 02](../../business-process/02-data-sources.md) for when a case
   needs them.
7. **An instant-service label names no courier.** The platform's *Instant / Same Day* label
   ([business-process 02](../../business-process/02-data-sources.md)) is the only style with **no
   courier logo at all** (its images are platform strips) **and no tracking ID**, so both clues
   that name every other courier are absent and nothing on the page can name one. It is recognised
   by the tag `GKS` with the caption `Kode Pengambilan`, no tracking ID and no courier logo, and an
   `In transit by` that is a clock time rather than `23:59`. Two marks that look like marks and are
   not: the **pickup code** beside that caption is two letters + 4 digits (`WX` on one page, `CJ`
   on the other) and the **header strip** is the buying channel's brand (`TikTok Shop` on a TikTok
   Shop order, `tokopedia PLUS` on a Tokopedia one). The service is served by **Gojek, Grab or
   Blitz** and the page never says which, so the name comes from the order export's
   `Shipping Provider Name`; the **Default** ([Q-LABEL-2](../questions/label.md)) shows
   `Gojek (instant)` from the tag `GKS` when the export has no row, and lists the order in the
   unknown-courier warning when neither is at hand.
8. **An instant label is still keyed by its Order ID**, printed on its `Nomor Order` line and in
   its barcode, so it is read like any other page; it prints no tracking ID, so that order's
   `tracking_id` stays empty and its slip is read normally.
9. **The Shopee-style label carries both clues, both new.** It is the fourth label style
   ([business-process 02](../../business-process/02-data-sources.md)): its logo is the small red
   `SPX` wordmark (68×27, masked, its encoded bytes `4583097ef370a670`, identical on all 3 of its
   pages), and its tracking ID has the form `SPXID` + 11 digits, which no other courier uses — so
   the two clues agree on every page of it, as the owner's rule requires. On the first batch the
   app asks the owner once about the logo (the prompt) and remembers the name for the logo
   ([D-LABEL-4](../decisions/label.md)); the name stored is the owner's — the wordmark reads `SPX`. The page's four other
   images (the `Shopee` channel strip, two barcodes and a QR, all different per order) belong to
   the rule for picking the logo ([Q-LABEL-5](../questions/label.md)), not to this one. **The rest
   of that page cannot be packed as it stands** — no 18-digit Order ID, no `Qty Total`, no
   `Customer Message`, and its slip's columns are named in Indonesian
   (`Nama Produk` · `SKU` · `Variasi` · `Qty`) — which is [Q-LABEL-4](../questions/label.md).

The courier list is `couriers.toml` (in the data folder in the app); its fields and an example are
in [inputs.md](../app/inputs.md). The app writes the logos the owner names in the prompt into it,
so a taught logo is recognised in every later batch ([D-LABEL-4](../decisions/label.md),
[Q-LABEL-7](../questions/label.md)).
