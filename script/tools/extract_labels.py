#!/usr/bin/env python3
"""Extract Order ID and Tracking ID from shipping-label PDFs into one CSV.

One row per order (`order_id,tracking_id`), in the order the orders first appear
in the PDFs. A page with no Order ID is the continuation of the previous page's
order (a long label spanning several pages).

Usage (from the repository root, Git Bash):
    .venv/Scripts/python.exe script/tools/extract_labels.py "samples/*Shipping label*.pdf" -o work/labels.csv

Needs only `pypdf` (already in the project venv). Reads the PDFs as they are;
nothing is uploaded anywhere.
"""
from __future__ import annotations

import argparse
import csv
import glob
import re
import sys
from pathlib import Path

from pypdf import PdfReader

# Same order as script/packing/labels.py: the raw search first, because on J&T
# labels the Order ID sits on its own line right under the Tracking ID.
_ORDER_ID = re.compile(r"(?<!\d)(5\d{17})(?!\d)")
_LABELLED_ORDER_ID = re.compile(r"OrderI[dD][:：](\d{18})")  # second colon is fullwidth
_WHITESPACE = re.compile(r"\s+")

# Tracking IDs printed as text on the label: J&T Express `JY` + 10 digits,
# SiCepat 12 digits starting `00`, IDX `TK` + 11 characters, J&T Cargo 12 digits.
# The prefixed forms are searched first so that a 12-digit phone number on the same
# page cannot be mistaken for the tracking ID. Some labels (IDX, fast track) extract
# one character per line, so the tracking ID is only visible once the whitespace is
# dropped - and then it is glued to the text around it ("REG" + "TKP…"), which is why
# the collapsed form is searched without lookaround guards.
_TRACKING_PREFIXED = re.compile(r"(?<![0-9A-Z])(JY\d{10}|TK[0-9A-Z]{11}|00\d{10})(?![0-9A-Z])")
_TRACKING_GLUED = re.compile(r"(JY\d{10}|TK[0-9A-Z]{11}|00\d{10})")
_TRACKING_PLAIN = re.compile(r"(?<!\d)(\d{12})(?!\d)")


def find_order_id(text: str) -> str | None:
    match = _ORDER_ID.search(text)
    if match is not None:
        return match.group(1)
    match = _LABELLED_ORDER_ID.search(_WHITESPACE.sub("", text))
    if match is not None:
        return match.group(1)
    return None


def find_tracking_id(text: str) -> str | None:
    collapsed = _WHITESPACE.sub("", text)
    for pattern, candidate in (
        (_TRACKING_PREFIXED, text),
        (_TRACKING_GLUED, collapsed),
        (_TRACKING_PLAIN, text),
        (_TRACKING_PLAIN, collapsed),
    ):
        match = pattern.search(candidate)
        if match is not None:
            return match.group(1)
    return None


def pdf_paths(inputs: list[str]) -> list[Path]:
    """Expand files, globs and directories the shell did not expand into PDFs."""
    paths: list[Path] = []
    for item in inputs:
        path = Path(item)
        if path.is_dir():
            found = sorted(path.glob("*.pdf"))
        elif path.is_file():
            found = [path]
        else:
            found = sorted(Path(p) for p in glob.glob(item))
        if not found:
            raise SystemExit(f"no PDF file matches {item!r}")
        paths.extend(found)
    seen = set()
    unique = []
    for path in paths:
        if path not in seen:
            seen.add(path)
            unique.append(path)
    return unique


def extract(paths: list[Path]) -> list[tuple[str, str]]:
    """Return (order_id, tracking_id) once per order, in first-seen order."""
    rows: list[tuple[str, str]] = []
    index: dict[str, int] = {}
    pages = 0
    no_order_id = 0
    for path in paths:
        try:
            reader = PdfReader(path)
        except Exception as exc:  # pypdf signals a missing/unreadable file this way
            raise SystemExit(f"{path}: cannot read PDF ({exc})") from exc
        previous: str | None = None
        for number, page in enumerate(reader.pages, start=1):
            pages += 1
            text = page.extract_text() or ""
            order_id = find_order_id(text)
            if order_id is None:
                if previous is None:
                    raise SystemExit(f"{path}: page {number} has no Order ID")
                order_id = previous  # continuation page of a multi-page label
                no_order_id += 1
            previous = order_id
            tracking_id = find_tracking_id(text) or ""
            if order_id in index:
                position = index[order_id]
                known = rows[position][1]
                if tracking_id and known and tracking_id != known:
                    print(
                        f"warning: {path} page {number}: order {order_id} has tracking "
                        f"{tracking_id}, earlier page had {known}",
                        file=sys.stderr,
                    )
                if tracking_id and not known:
                    rows[position] = (order_id, tracking_id)
                continue
            index[order_id] = len(rows)
            rows.append((order_id, tracking_id))
    print(
        f"{len(paths)} PDF(s), {pages} page(s), {len(rows)} order(s), "
        f"{no_order_id} continuation page(s), "
        f"{sum(1 for _, t in rows if not t)} order(s) without a Tracking ID",
        file=sys.stderr,
    )
    return rows


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("pdfs", nargs="+", help="PDF files, a folder, or a glob")
    parser.add_argument("-o", "--output", default="labels.csv", help="output CSV (default: labels.csv)")
    args = parser.parse_args(argv)

    rows = extract(pdf_paths(args.pdfs))

    output = Path(args.output)
    if output.parent != Path(""):
        output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(["order_id", "tracking_id"])
        writer.writerows(rows)
    print(f"wrote {len(rows)} row(s) to {output}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
