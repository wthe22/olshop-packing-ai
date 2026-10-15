"""Compare out/rust-orders.json and out/python-orders.json.

Prints counts only, never any real values.
"""
from __future__ import annotations

import json
from pathlib import Path

OUT = Path(__file__).with_name("out")
FIELDS = ("lines", "qty_total", "customer_message", "pages")


def main() -> None:
    rust = json.loads((OUT / "rust-orders.json").read_text(encoding="utf-8"))
    python = json.loads((OUT / "python-orders.json").read_text(encoding="utf-8"))
    kr, kp = set(rust), set(python)

    equal = different = 0
    field_diff = {f: 0 for f in FIELDS}
    line_count_diff = 0
    line_value_diff = 0
    for key in kr & kp:
        if rust[key] == python[key]:
            equal += 1
            continue
        different += 1
        for f in FIELDS:
            if rust[key].get(f) != python[key].get(f):
                field_diff[f] += 1
        rl, pl = rust[key].get("lines", []), python[key].get("lines", [])
        if len(rl) != len(pl):
            line_count_diff += 1
        elif rl != pl:
            line_value_diff += 1

    print(f"rust orders: {len(rust)}  python orders: {len(python)}")
    print(f"equal: {equal}  different: {different}")
    print(f"missing in rust: {len(kp - kr)}  missing in python: {len(kr - kp)}")
    print(f"field diffs: {field_diff}")
    print(f"orders with different line count: {line_count_diff}  same count but different lines: {line_value_diff}")


if __name__ == "__main__":
    main()
