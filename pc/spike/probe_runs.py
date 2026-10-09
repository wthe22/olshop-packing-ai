"""Probe: what granularity does pypdf's visitor_text give on the real samples?

Dumps runs to out/ (git-ignored). Prints counts only, never real values.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

from pypdf import PdfReader

OUT = Path(__file__).with_name("out")
OUT.mkdir(exist_ok=True)
SAMPLES = Path(__file__).resolve().parents[2] / "samples"

_HEADER_WORDS = ("Product Name", "SKU", "Seller SKU", "Qty")


def collector(collected):
    def visitor(text, cm, tm, font_dict, font_size):
        stripped = text.strip()
        if not stripped:
            return
        x = tm[4] * cm[0] + tm[5] * cm[2] + cm[4]
        y = tm[4] * cm[1] + tm[5] * cm[3] + cm[5]
        collected.append((x, y, stripped))

    return visitor


def main() -> None:
    files = sorted(SAMPLES.glob("*Shipping label+Packing slip_*.pdf"))
    reader = PdfReader(files[0])
    lines_out = []
    stats = {
        "pages": 0,
        "runs": 0,
        "single_token": 0,
        "has_space": 0,
        "len_eq_headerword": 0,
        "int_runs": 0,
    }
    for pno in (0, 1, 2):
        runs = []
        page = reader.pages[pno]
        page.extract_text(visitor_text=collector(runs))
        stats["pages"] += 1
        stats["runs"] += len(runs)
        for x, y, t in runs:
            if " " not in t:
                stats["single_token"] += 1
            else:
                stats["has_space"] += 1
            if t in _HEADER_WORDS:
                stats["len_eq_headerword"] += 1
            if re.fullmatch(r"-?\d+", t):
                stats["int_runs"] += 1
            lines_out.append(f"{pno}\t{x:.2f}\t{y:.2f}\t{t!r}")
    (OUT / "probe-pyruns.txt").write_text("\n".join(lines_out), encoding="utf-8")
    print("PROBE", stats)


if __name__ == "__main__":
    main()
