"""Reference: build a packing list with the Python code on testdata, for a size comparison.

Uses only testdata/ (labels-slip.pdf, labels-plain.pdf, orders-1.csv, categories.toml) and
the real packing pipeline, and writes out/python-packing-list.pdf (git-ignored).
"""

from __future__ import annotations

from datetime import datetime
from pathlib import Path

from packing.labels import LabelSet
from packing.orders import read_orders
from packing.packing_list import write_packing_list
from packing.picks import build_orders, make_picks
from packing.rules import load_rules

REPO = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent / "out"


def main() -> None:
    td = REPO / "testdata"
    label_set = LabelSet.open([td / "labels-slip.pdf"])
    label_orders = label_set.orders()
    csv = read_orders(td / "orders-1.csv")
    orders, warnings = build_orders(label_orders, csv)
    categories = load_rules(td / "categories.toml")
    pdfs = make_picks(orders, categories, 1)

    out = OUT / "python-packing-list.pdf"
    pages = write_packing_list(
        out, "full", day="2026-10-06", pdfs=pdfs, printed=datetime(2026, 10, 6, 16, 40)
    )
    print(
        f"python-packing-list.pdf: bytes={out.stat().st_size} pages={pages} "
        f"orders={len(orders)} pdfs={len(pdfs)} warnings={len(warnings)}"
    )


if __name__ == "__main__":
    main()
