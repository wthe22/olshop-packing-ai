# 2.16 Courier: the SiCepat labels

23 of the owner's 98 orders get no courier (J&T Express 74, SiCepat 23, IDX 1; the day warns 15 + 8):
SiCepat labels print their name **only as a logo image** and their tracking ID is 12 digits starting
`00`, a form `couriers.toml` marks as shared (`tracking_unique = false`; J&T Cargo is also 12
digits), so `deduce_courier` (`crates/engine/src/label.rs:217`) refuses and warns. The 601-page
sample download is the same shape: 82 of its pages.

**The owner's rule: two clues, and the courier is named only when they agree** — the **logo** (the
page's logo image, its encoded bytes hashed with **XXH3-64**) and the **tracking-ID form** (`00` →
SiCepat, `57` → J&T Cargo, both unambiguous). [courier.md](../../software-design/labels/courier.md)
holds the rule; the logo hashes, the ask-the-owner flow and J&T Cargo's unreadable logo are
[Q-LABEL-1](../../software-design/questions/label.md).

**Code:** `couriers.toml` gains a logo-hash list (XXH3-64, `xxhash-rust`) per courier and a J&T Cargo
entry — whose clue is the logo's **appearance** (`shape`; its encoded bytes name nothing) — and
`deduce_courier` hashes the page's logo image (`FPDFImageObj_GetImageDataRaw`) and requires the two
clues to agree; a hash not in the table is asked of the owner once (`couriers.toml` — one file,
[D-LABEL-1](../../software-design/decisions/label.md); the taught hash is appended to `couriers.toml`,
[D-LABEL-4](../../software-design/decisions/label.md) and
[Q-LABEL-7](../../software-design/questions/label.md); *I don't know* and *Skip all* write nothing).

**Tests:** every measured sample page keeps its courier, a logo with a foreign form warns, an unseen
logo asks, the two slip-continuation pages stay unknown. Then re-check the owner's day (the warning
is gone for the SiCepat orders) and rebuild the portable folder.

**Measured counts and shapes** (the Python reader page by page and the exports read by provider):

- The 601 sample pages → J&T Express 517, no courier 84 (82 with a `00` + 10-digit tracking ID, 1 slip
  continuation, 1 `TikTok Shop` label with no tracking ID); `sicepat` on 0 pages, in the plain and in
  the whitespace-stripped text; the exports of an earlier sample download name SiCepat 173 orders
  (`00…`) and J&T Cargo 1 (`57…`). The two 12-digit forms do not collide. Both J&T Cargo labels in the
  samples print the `57…` form (one with its slip continuing on the next page), and for one of them
  the export's own row names J&T Cargo; both print `FastTrack` as text.
- The samples as a whole (9 files, 1,269 pages; two of them of 50 and 40 pages, the 50-page one
  carrying no J&T Cargo label): J&T Express 1,114 · SiCepat 148 · J&T Cargo 2 · IDX 2 · `TikTok Shop`
  1 · slip continuations 2. Every page is one order except that one Cargo order. The two clues: the
  service tag read from the header strip gives J&T Express `EZ` 453 · `ECO` 253 · `NDD` 408, SiCepat
  `NDD` 41 · `REG` 73 · `HALU` 34, J&T Cargo `FastTrack` 2, IDX `REG` 2, TikTok Shop `GKS` 1 — form
  and tag agree on all 1,266 label pages that have a tracking form, and the one `TikTok Shop` label
  has a tag but no form (so it stays unknown); the 2 slip continuations have neither. The courier
  logo is an embedded image whose *encoded bytes* (1.5–15 KB, so an XXH3-64 hash costs about 0.5 µs a
  page; measured 0.44 µs for a 14 KB logo) hash to one stable value per courier across all 9 files:
  J&T Express `39b9b7b9f61d0e7f`, SiCepat `cf33ed60daa291b1`, IDX `9131c1f94ad363d4`, TikTok Shop
  `f2cf7019646c955b`. J&T Cargo's logo is a *masked* image re-rastered between exports (680×156 /
  683×157): its encoded bytes name nothing, but its **appearance** — composited with its `/SMask`,
  fitted into a 64 × 64 canvas on white, thresholded at 200/255 — gives **one value for both
  rasters** (`2a36acec58f5…`), so it can be a logo clue like the others. The barcode and QR are
  vector paths and every page also carries a page-wide frame image. The spill: the label takes the
  top 70% of the page on every style (the slip starts at 126.5 pt on all four), so only the slip's
  own content spills — the sample's one spilled order had a single item line and a buyer message.
- A later slip download (3 files, 604 pages, 600 orders, `tools/analysis/helpers/courier_clues.py`):
  J&T Express 500 · SiCepat 96 · iD Express (IDX) 3 · J&T Cargo 1 · 4 slip continuations. It is the
  first slip export with a J&T Cargo order (its slip continuing on the next page) and with iD Express
  orders; both couriers appear otherwise only on plain label exports. The Cargo page's clues: the
  `57` + 10-digit form (printed **once**, in the whitespace-collapsed text), `FastTrack` in the
  header strip, `TT Order ID` as its order-ID caption, `COD` and `Print Time`; its logo is the 680×156
  raster whose base and mask bytes are identical to the other Cargo sample label's. No new courier,
  tracking form or service tag appears in the whole samples folder (12 files, 1,873 pages): J&T
  Express 1,614 · SiCepat 244 · iD Express 5 · J&T Cargo 3 · the one instant-service label · 6
  continuations.
- **The new courier the owner reported is Gojek.** It is the page counted above as the `TikTok Shop`
  label: the platform's *Instant / Same Day* style, the only label with **no courier logo, no
  tracking ID and no `Weight`** — it prints `TikTok Shop` + the tag `GKS` + `Kode Pengambilan` with a
  `WX` + 4-digit pickup code, then `Nomor Order` + the 18-digit Order ID + a barcode, and its
  `In transit by` is a pickup **time** (`08/10/2026 10:00`) where the 199 other pages of that download
  say `23:59`. Tokopedia's seller guide: the service is served by **Gojek, Grab or Blitz**, the pickup
  code is printed as a watermark too, and a Same Day label cannot be printed before the delivery is
  arranged. The page never names which of the three, so an instant label cannot give a courier the
  way `couriers.toml` does — its name has to come from the export's `Shipping Provider Name`:
  [Q-LABEL-2](../../software-design/questions/label.md), with
  [Q-LABEL-3](../../software-design/questions/label.md) for where the pickup code shows during
  packing.
- Checked page by page for that style in the later download (3 files, 604 pages, 600 orders): **none**
  — `Kode Pengambilan` appears on exactly one of the 1,873 sample pages, the 326×59 `TikTok Shop`
  image on none of the 604, every page's images and captions are one of the four courier styles, every
  page prints `In transit by: 14/10/2026 23:59`, and the 600 distinct 18-digit Order IDs cover the 604
  pages exactly (596 orders one page, 4 orders two) — so the day's Gojek order either has no printable
  label (the platform's rule above) or is the one page the owner can point at. **A single page is not
  a pattern**: every mark of the instant label comes from that one page, and the sweep looked for the
  one caption `Kode Pengambilan`, so a differently drawn instant label would not have been caught.

**Result:** (not started; [Q-LABEL-2](../../software-design/questions/label.md) and
[Q-LABEL-3](../../software-design/questions/label.md) have Default answers, so nothing waits for the
owner)
