# Courier deduction

The label is read for two clues and the courier is named **only when the two agree** (the owner's rule,
[D-LABEL-2](../decisions/label.md)): one clue alone can be wrong, so a wrong name would need two marks of
the label to be wrong in the same way. The clues are the owner's data: `courier_classifier` and
`not_courier` of `packing-workflows.toml` ([rules file](../rules-file/README.md#couriers)).

1. **The logo** — the main clue. Every label prints the courier's logo as an embedded image. The
   **candidates** are the images whose whole box lies in the page's **top 40 pt**
   ([D-LABEL-9](../decisions/label.md)): on all six sample styles those are exactly the courier's logo and
   the buying channel's strip, while the page-wide frame (134–418 pt), the slip's footer strip (39–62, or
   104–124 on an instant label) and the barcodes and the QR (173–224, 330–373) fall outside. A candidate's
   **encoded bytes** (PDFium's `FPDFImageObj_GetImageDataRaw`, not the decoded pixels) are hashed with
   XXH3-64 (seed 0) and looked up in `logos`; a hash in `not_courier.logos` is not a logo. Measured stable
   across every sample export: J&T Express `39b9b7b9f61d0e7f`, SiCepat `cf33ed60daa291b1`, iD Express (IDX)
   `9131c1f94ad363d4`, SPX `4583097ef370a670`, Anteraja `556e376839c5805d` (354×142, one sample page). The
   buying channels' strips are in the bundled `not_courier.logos`: `tokopedia | Shop` `6c1340227db55be4`,
   `tokopedia | TikTok` `c2230c3c5d8c560d`, the Shopee wordmark `07fea99cbec66ae7`, `TikTok Shop`
   `f2cf7019646c955b`, `tokopedia PLUS` `330b79a3b29d4646`. About 0.5 µs a page.
2. **A masked logo by its appearance** ([D-LABEL-6](../decisions/label.md)). A courier can print its logo as
   an image whose shape is a `/SMask` over one flat colour (J&T Cargo: 331 bytes for 680×156), re-rastered
   between exports (680×156, 683×157), so its bytes name nothing. For a candidate whose bytes are in
   neither list, the app hashes its **appearance**: the image as PDFium renders it (straight RGBA, the
   `/SMask` as alpha) composited onto white; scaled to fit 64 × 64 with a Lanczos (a = 3) filter in `f64`
   and pasted centred on a white canvas; a pixel is ink when its luma
   (`(r·19595 + g·38470 + b·7471 + 32768) >> 16`) is below 200; the 64 × 64 bits packed 8 to a byte, most
   significant first (512 bytes), hashed with XXH3-64 and looked up in `shapes`. J&T Cargo's two rasters
   both give `753ccbd14749befa`. The decode is paid only where the bytes matched nothing (about 1 page in 600).
3. **The tracking-ID form** ([business-process 02](../../business-process/02-data-sources.md)): `JY` or
   `JX` + 10 digits → J&T Express; `00` + 10 digits → SiCepat; `57` + 10 digits → J&T Cargo; `TKP` + 10
   digits → IDX; `SPXID` + 11 digits → SPX (the Shopee-style label prints a check letter around the number:
   `Resi:` without it, `SPXID…A` with it); `TSA-` + 11 digits → Anteraja (one sample page). The form must match a token, or a match in the
   whitespace-collapsed text, **fully**; it need not be repeated (the boxed layouts print it once). A form
   the platform adds is a line of `packing-workflows.toml`, not a program change.
4. **Named only when the two agree.** The form gives the *candidate* couriers; the courier is named when the
   logo's courier (by its bytes, else by its appearance) is among them. A logo whose courier and form
   disagree, a logo with no form, a form with no logo, or neither → no name: `courier` stays empty and a
   warning lists the orders. A page with **no header image at all** (only made-up test labels) is named by
   a lettered form that fits one courier (`JY`, `TKP`, `SPXID`), never by a bare-number form (`00`, `57`)
   ([D-LABEL-11](../decisions/label.md)).
5. **A printed text clue** (`text`: `jet.co.id`, `FastTrack`) stands in for the logo only where no candidate
   image can be read at all. J&T Cargo prints `FastTrack` in its header strip; SiCepat and IDX print no name.
6. **The service tag is not used** on a normal label (the owner: it is weak for some couriers). The
   measured tags stay in [business-process 02](../../business-process/02-data-sources.md).
7. **An instant-service label is named by its tag** ([D-LABEL-21](../decisions/label.md)). The platform's
   *Instant / Same Day* label ([business-process 02](../../business-process/02-data-sources.md)) is the only
   style with no courier logo and no tracking ID. It is recognised by the caption `Kode Pengambilan` with no
   tracking ID and no courier logo; its tag is looked up in `instant_tags`: `GKS` names Gojek, `Sameday`
   Grab. Its header images are the buying channel's strips and are **never asked about**: a strip named as
   a courier would leave every instant label of that channel unnamed. A tag in no list is asked about once
   in the logo prompt, like an unseen logo, and the answer is added to that courier's `instant_tags`. The
   style is known from three pages: each of these marks is re-checked on the next instant labels before it
   is relied on.
8. **The Shopee-style label carries both clues**: the small red `SPX` wordmark (68×27, masked, bytes
   `4583097ef370a670`) and `SPXID` + 11 digits, which no other courier uses
   ([D-LABEL-5](../decisions/label.md)).

## The logo prompt, and what is remembered

A candidate whose hash is in no list is asked of the owner once, before the plan: a courier, **Not a
courier** (a channel strip), or **I don't know** ([New batch › The logo prompt](../ui/new-batch.md#the-logo-prompt)).
A courier's answer appends the hash to that courier's `logos`; *Not a courier* appends it to
`not_courier.logos`; comments and order of the file are kept ([D-LABEL-4](../decisions/label.md),
[D-LABEL-10](../decisions/label.md)). The same image is never asked about again.

## The clue that named it

Each saved order records its `courier_clue`: the kind (`logo`, `shape`, `text`, `form` for a page with no
header image, `tag`, or `none`), the hash, text or tag, and the tracking-ID form that agreed
([records](../data/records.md), [D-LABEL-14](../decisions/label.md)). Search shows it in the order's trace.
