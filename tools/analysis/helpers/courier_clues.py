#!/usr/bin/env python3
"""Show the courier evidence of every page of shipping-label PDFs.

One line per page with the clues the courier deduction uses
([desktop design 03 › Courier deduction](../../../docs/desktop/design/03-labels-and-picks.md)):

- `raw`      XXH3-64 (seed 0, 16 hex) of the image's **encoded bytes** — the app's first clue
             (`FPDFImageObj_GetImageDataRaw` on the label's logo image). The white list of a
             *masked* logo (`/SMask`) is a flat colour, so this value names nothing for it.
- `shape`    XXH3-64 of the **appearance**: the image composited with its `/SMask` (if any) and
             fitted into a fixed 64 x 64 white canvas, thresholded at 200/255. One value per
             courier *asset* that survives a re-raster (measured: J&T Cargo 680x156 and 683x157
             both give 2a36acec58f5), which the raw bytes do not.
- `tag`      the page's first printed word: the service tag of the header strip (EZ, ECO, NDD,
             REG, HALU, FastTrack, …). Read it here, never search the page for it.
- `form`     the tracking-ID form of the whitespace-collapsed text (JY+10, 00+10, 57+10, TKP+10).
- `text`     the printed clauses a masked logo's courier can be recognised by (FastTrack,
             TT Order ID, Print Time, Nomor Order, …).

Images that every page carries (the 905x118 frame, the 2000x2000 barcode/QR, the 1000x1000
boxed-layout frame) are skipped; the platform strips (`tokopedia | TikTok` and friends) are shown,
because they sit in the header next to a courier logo.

Usage (from the repository root, Git Bash), with the project virtual environment:

    source .venv/Scripts/activate
    python -m pip install pypdf pillow xxhash      # once
    python tools/analysis/helpers/courier_clues.py samples/*.pdf
    python tools/analysis/helpers/courier_clues.py samples/*.pdf --summary

`samples/` is real data: keep the output in `work/` or `tmp/` (git-ignored) and never paste real
Order ID, tracking ID, names or addresses into the repository.
"""
from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

import xxhash
from PIL import Image
from pypdf import PdfReader

# images every label page carries, never the courier's logo
PAGE_IMAGES = {(905, 118), (2000, 2000), (1000, 1000)}
CANVAS = 64
THRESHOLD = 200

CLAUSES = ("FastTrack", "TTOrderID", "PrintTime", "NomorOrder", "KodePengambilan",
           "TT Order ID", "Print Time", "Nomor Order", "Kode Pengambilan")
ORDER_ID = re.compile(r"(?<!\d)5\d{17}(?!\d)")
LABELLED_ORDER_ID = re.compile(r"OrderI[dD][:：](\d{18})", re.I)
TRACKING = re.compile(r"(?<![0-9A-Z])(JY|JT|TKP|WX)([0-9]{10,14})(?![0-9A-Z])")
TRACKING_GLUED = re.compile(r"(JY|JT|TKP|WX)([0-9]{10})")
TRACKING_NUMERIC = re.compile(r"(?<!\d)([0-9]{12})(?!\d)")


def collapse(text: str) -> str:
    return re.sub(r"\s+", "", text)


def page_text(page) -> tuple[str, str, str]:
    """(raw text, whitespace-collapsed text, first printed word)."""
    raw = page.extract_text() or ""
    flat = re.sub(r"\s+", " ", raw).strip()
    first = re.sub(r"[^A-Za-z]", "", flat.split(" ", 1)[0]) if flat else ""
    return raw, collapse(raw), first


def order_id(raw: str, collapsed: str) -> str:
    match = ORDER_ID.search(raw) or ORDER_ID.search(collapsed)
    if match:
        return match.group(0)
    match = LABELLED_ORDER_ID.search(collapsed)
    return match.group(1) if match else ""


def tracking_forms(raw: str, collapsed: str, has_logo: bool) -> str:
    """The tracking-ID candidates of the page as `form xN`, strongest first.

    A page without a logo image is a packing-slip continuation: it carries no tracking ID, and its
    18-digit Order ID (printed several times) would otherwise show up as a bare 12-digit candidate.

    The count is the evidence: a label prints its tracking ID several times, while a phone number
    or an address number appears once — but J&T Cargo's label prints its tracking ID only once, in
    the whitespace-collapsed text, so a count of 1 is not by itself a reason to drop a candidate.
    The courier data decides which form names a courier (`couriers.toml`), exactly as the reader
    does.
    """
    if not has_logo:
        return ""
    counts: Counter[str] = Counter()
    for text in (raw, collapsed):
        for pattern in (TRACKING, TRACKING_GLUED):
            for match in pattern.finditer(text):
                counts[f"{match.group(1)}+{len(match.group(2))}dig"] += 1
        for match in TRACKING_NUMERIC.finditer(text):
            counts[f"{match.group(1)[:2]}+10dig"] += 1
    if not counts:
        return ""
    ordered = sorted(counts.items(), key=lambda item: (-item[1], item[0]))
    return " ".join(f"{form}x{count}" if count > 1 else form for form, count in ordered)


def clauses(flat: str, collapsed: str) -> list[str]:
    return sorted({c for c in CLAUSES if c in flat or c in collapsed})


def to_image(obj):
    """The image object as a picture, with its /SMask applied; None if the data cannot be read."""
    width, height = int(obj.get("/Width", 0)), int(obj.get("/Height", 0))
    if width <= 0 or height <= 0:
        return None
    space = str(obj.get("/ColorSpace"))
    data = obj.get_data()
    try:
        if space.endswith("DeviceGray"):
            image = Image.frombytes("L", (width, height), data[: width * height].ljust(width * height, b"\x00"))
            image = image.convert("RGB")
        else:
            size = width * height * 3
            image = Image.frombytes("RGB", (width, height), data[:size].ljust(size, b"\x00"))
    except Exception:
        return None
    if obj.get("/SMask") is not None:
        try:
            smask = obj["/SMask"].get_object()
            size = (int(smask.get("/Width", 0)), int(smask.get("/Height", 0)))
            raw = smask.get_data()
            mask = Image.frombytes("L", size, raw[: size[0] * size[1]].ljust(size[0] * size[1], b"\x00"))
            if mask.size != image.size:
                mask = mask.resize(image.size)
            image = Image.composite(image, Image.new("RGB", image.size, (255, 255, 255)), mask)
        except Exception:
            pass
    return image


def shape_hash(obj) -> str:
    """XXH3-64 of the appearance: fitted into CANVAS x CANVAS on white, thresholded."""
    image = to_image(obj)
    if image is None:
        return ""
    scale = min(CANVAS / image.width, CANVAS / image.height)
    resized = image.resize((max(1, round(image.width * scale)), max(1, round(image.height * scale))),
                           Image.LANCZOS)
    canvas = Image.new("RGB", (CANVAS, CANVAS), (255, 255, 255))
    canvas.paste(resized, ((CANVAS - resized.width) // 2, (CANVAS - resized.height) // 2))
    bits = canvas.convert("L").point(lambda p: 255 if p < THRESHOLD else 0).convert("1")
    return xxhash.xxh3_64(bits.tobytes()).hexdigest()


def images(page) -> list[tuple[str, int, int, str, str, bool]]:
    """(name, width, height, raw hash, shape hash, has mask) for the page's images."""
    xobjects = page.get("/Resources", {}).get("/XObject")
    if xobjects is None:
        return []
    if hasattr(xobjects, "get_object"):
        xobjects = xobjects.get_object()
    found = []
    for name, reference in list(xobjects.items()):
        obj = reference.get_object()
        width, height = int(obj.get("/Width", 0)), int(obj.get("/Height", 0))
        if (width, height) in PAGE_IMAGES:
            continue
        found.append((str(name), width, height,
                      xxhash.xxh3_64(obj._data).hexdigest(),  # noqa: SLF001 - the encoded bytes
                      shape_hash(obj), "/SMask" in obj))
    return found


def report(paths: list[Path], summary: bool) -> None:
    combinations: Counter[tuple] = Counter()
    for path in paths:
        try:
            reader = PdfReader(path)
        except Exception as exc:
            raise SystemExit(f"{path}: cannot read PDF ({exc})") from exc
        for number, page in enumerate(reader.pages, start=1):
            raw, collapsed, first = page_text(page)
            found = images(page)
            form = tracking_forms(raw, collapsed, bool(found))
            clues = clauses(re.sub(r"\s+", " ", raw), collapsed)
            note = "" if found else "  (no logo image: packing-slip continuation)"
            print(f"{path.name} p{number} order={order_id(raw, collapsed) or '-'} "
                  f"tag={first or '-'} forms={form or '-'} "
                  f"logo_sizes={[f'{w}x{h}' for _, w, h, _, _, _ in found] or '-'} "
                  f"clauses={clues or '-'}{note}")
            for name, width, height, digest, shape, masked in found:
                print(f"    {name} {width}x{height}{' masked' if masked else ''} "
                      f"raw={digest} shape={shape}")
            logo = tuple(sorted(f"{w}x{h}:{shape[:8]}" for _, w, h, _, shape, _ in found if shape))
            combinations[(first if found else "", form, logo, tuple(clues))] += 1
    if summary:
        print("\n=== page groups (tag, tracking forms, logo shape hashes, clauses) ===")
        for key, count in combinations.most_common():
            tag, form, logo, clues = key
            print(f"  n={count:4} tag={tag or '-'} forms={form or '-'} "
                  f"logo={list(logo) or '-'} clauses={list(clues) or '-'}")


def pdf_paths(inputs: list[str]) -> list[Path]:
    found: list[Path] = []
    for item in inputs:
        path = Path(item)
        if path.is_dir():
            found.extend(sorted(path.glob("*.pdf")))
        elif path.is_file():
            found.append(path)
        else:
            raise SystemExit(f"no PDF file matches {item!r}")
    seen, unique = set(), []
    for path in found:
        if path not in seen:
            seen.add(path)
            unique.append(path)
    return unique


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("pdfs", nargs="+", help="label PDF files or a folder")
    parser.add_argument("--summary", action="store_true",
                        help="also group the pages by (tag, form, logo shape hash, clauses)")
    args = parser.parse_args(argv)
    paths = pdf_paths(args.pdfs)
    report(paths, args.summary)
    print(f"\n{len(paths)} PDF(s) read", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
