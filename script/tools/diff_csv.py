#!/usr/bin/env python3
"""Diff two CSVs by Order ID and write the differences only when asked to.

    diff_csv.py A.csv B.csv [--missing FILE] [--new FILE] [--common FILE]

A is the earlier file, B the current one. Three sets of Order IDs:

    missing   in A, gone from B
    new       in B, not in A
    common    in both

By default nothing is written: the three counts and the Order IDs are printed on the
screen. A set is written to a CSV file (whole rows, same header as the source) only
when its option is given; `--common` is written from B.

The key is always the Order ID: the column named `Order ID` or `order_id`
(case-insensitive), which the file must have. A repeated Order ID is kept once - an
order with several products is several rows in an orders export.

Usage (from the repository root, Git Bash):
    .venv/Scripts/python.exe script/tools/diff_csv.py work/labels-old.csv work/labels-new.csv
    .venv/Scripts/python.exe script/tools/diff_csv.py work/labels-old.csv work/labels-new.csv \
        --missing work/missing.csv --new work/new.csv --common work/common.csv
"""
from __future__ import annotations

import argparse
import csv
import sys
from pathlib import Path

MISSING = "missing"
NEW = "new"
COMMON = "common"


def read_csv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(newline="", encoding="utf-8-sig") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames is None:
            raise SystemExit(f"{path}: no header row")
        header = list(reader.fieldnames)
        rows = [{name: (row.get(name) or "").strip() for name in header} for row in reader]
    return header, rows


def order_id_column(header: list[str], path: Path) -> str:
    for name in header:
        if name.lower().replace(" ", "_") == "order_id":
            return name
    raise SystemExit(f"{path}: no Order ID column; columns are: {', '.join(header)}")


def keyed(rows: list[dict[str, str]], key: str, path: Path) -> dict[str, dict[str, str]]:
    result: dict[str, dict[str, str]] = {}
    duplicates = 0
    for row in rows:
        value = row.get(key, "")
        if not value:
            raise SystemExit(f"{path}: a row has an empty {key!r}")
        if value in result:
            duplicates += 1
            continue
        result[value] = row
    if duplicates:
        print(f"note: {path}: {duplicates} row(s) share an Order ID, first one kept", file=sys.stderr)
    return result


def report(name: str, rows: list[dict[str, str]], key: str, out: Path | None, header: list[str]) -> None:
    if out is None:
        print(f"{name}: {len(rows)}")
        for row in rows:
            print(f"  {row[key]}")
        return
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=header, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    print(f"{name}: {len(rows)} -> {out}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("first", help="earlier CSV (A)")
    parser.add_argument("second", help="current CSV (B)")
    parser.add_argument("--missing", metavar="FILE", type=Path, help="write the Order IDs in A but not in B")
    parser.add_argument("--new", metavar="FILE", type=Path, help="write the Order IDs in B but not in A")
    parser.add_argument("--common", metavar="FILE", type=Path, help="write the Order IDs in both (from B)")
    args = parser.parse_args(argv)

    first_path = Path(args.first)
    second_path = Path(args.second)
    first_header, first_rows = read_csv(first_path)
    second_header, second_rows = read_csv(second_path)
    first_key = order_id_column(first_header, first_path)
    second_key = order_id_column(second_header, second_path)
    first = keyed(first_rows, first_key, first_path)
    second = keyed(second_rows, second_key, second_path)

    missing = [row for value, row in first.items() if value not in second]
    new = [row for value, row in second.items() if value not in first]
    common = [row for value, row in second.items() if value in first]

    print(f"{first_path}: {len(first)} order(s), {second_path}: {len(second)} order(s)")
    report(MISSING, missing, first_key, args.missing, first_header)
    report(NEW, new, second_key, args.new, second_header)
    report(COMMON, common, second_key, args.common, second_header)

    if "tracking_id" in first_header and "tracking_id" in second_header:
        changed = sum(
            1
            for value, row in second.items()
            if value in first
            and row["tracking_id"]
            and first[value]["tracking_id"]
            and row["tracking_id"] != first[value]["tracking_id"]
        )
        if changed:
            print(f"note: {changed} order(s) are in both files with a different Tracking ID")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
