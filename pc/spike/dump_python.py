"""Dump the same per-order structure as the Rust spike, from the Python reference
(packing.labels.LabelSet.open(paths).orders()) to out/python-orders.json.

Prints counts/time/size only.
"""
from __future__ import annotations

import json
import os
import time
from pathlib import Path

from packing.labels import LabelSet

# SPIKE_LABELS=<dir> SPIKE_MATCH=<name part>: same override as the Rust spike.
SAMPLES = Path(os.environ.get("SPIKE_LABELS", Path(__file__).resolve().parents[2] / "samples"))
MATCH = os.environ.get("SPIKE_MATCH", "Shipping label+Packing slip")
OUT = Path(__file__).with_name("out")
OUT.mkdir(exist_ok=True)


def main() -> None:
    files = sorted(SAMPLES.glob(f"*{MATCH}*.pdf"))
    start = time.perf_counter()
    label_set = LabelSet.open(files)
    orders = label_set.orders()
    elapsed = time.perf_counter() - start

    dump = {}
    for order_id, order in orders.items():
        dump[order_id] = {
            "lines": [
                {
                    "name": line.name,
                    "variation": line.variation,
                    "seller_sku": line.seller_sku,
                    "quantity": line.quantity,
                }
                for line in order.lines
            ],
            "qty_total": order.qty_total,
            "customer_message": order.customer_message,
            "pages": [{"file": p.file.name, "index": p.index} for p in order.pages],
        }
    path = OUT / "python-orders.json"
    path.write_text(json.dumps(dump, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"python orders: {len(dump)}  seconds: {elapsed:.3f}  bytes: {path.stat().st_size}")


if __name__ == "__main__":
    main()
