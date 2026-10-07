#!/usr/bin/env python3
"""Diff two CSVs by one key column and write the differences as two CSVs.

    diff_csv.py A.csv B.csv [--key COLUMN] [--prefix PREFIX] [--out-dir DIR]
        ->  <prefix>missing.csv   rows whose key is in A but not in B
            <prefix>new.csv       rows whose key is in B but not in A

A is the earlier file, B the current one: "missing" = gone since A, "new" = appeared
since A. Rows are written whole (all columns of the file they come from), with the
same header as the source. The key column defaults to the first column; give
`--key "Order ID"` for an orders export. The prefix defaults to B's file name without
`.csv`, so `diff_csv.py old.csv orders-06-2.csv` writes `orders-06-2-missing.csv` and
`orders-06-2-new.csv` next to B.

Usage (from the repository root, Git Bash):
    .venv/Scripts/python.exe script/tools/diff_csv.py work/labels-old.csv work/labels-new.csv
"""
from __future__ import annotations

import argparse
import csv
import sys
from pathlib import Path

MISSING_SUFFIX = "missing.csv"
NEW_SUFFIX = "new.csv"


def read_csv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(newline="", encoding="utf-8-sig") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames is None:
            raise SystemExit(f"{path}: no header row")
        header = list(reader.fieldnames)
        rows = [{name: (row.get(name) or "").strip() for name in header} for row in reader]
    return header, rows


def resolve_key(header: list[str], wanted: str | None, path: Path) -> str:
    if wanted is not None:
        if wanted in header:
            return wanted
        for name in header:
            if name.lower() == wanted.lower():
                return name
        raise SystemExit(f"{path}: no column {wanted!r}; columns are: {', '.join(header)}")
    for name in header:
        if name.lower() in ("order_id", "order id"):
            return name
    return header[0]


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
        print(f"note: {path}: {duplicates} row(s) share a key, first one kept", file=sys.stderr)
    return result


def write(path: Path, header: list[str], rows: list[dict[str, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=header, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("first", help="earlier CSV (A)")
    parser.add_argument("second", help="current CSV (B)")
    parser.add_argument("--key", help='column to compare (default: first column, or "Order ID" if present)')
    parser.add_argument("--prefix", help="output file prefix (default: B's file name without .csv)")
    parser.add_argument("--out-dir", help="folder for the two output files (default: B's folder)")
    args = parser.parse_args(argv)

    first_path = Path(args.first)
    second_path = Path(args.second)
    first_header, first_rows = read_csv(first_path)
    second_header, second_rows = read_csv(second_path)

    first_key = resolve_key(first_header, args.key, first_path)
    second_key = resolve_key(second_header, args.key, second_path)
    first = keyed(first_rows, first_key, first_path)
    second = keyed(second_rows, second_key, second_path)

    missing = [row for value, row in first.items() if value not in second]
    new = [row for value, row in second.items() if value not in first]

    changed = 0
    if first_key.lower() == "order_id" and "tracking_id" in second_header and "tracking_id" in first_header:
        for value, row in first.items():
            other = second.get(value)
            if other is not None and row["tracking_id"] and other["tracking_id"] and row["tracking_id"] != other["tracking_id"]:
                changed += 1

    prefix = args.prefix or second_path.stem
    out_dir = Path(args.out_dir) if args.out_dir else second_path.parent
    missing_path = out_dir / f"{prefix}-{MISSING_SUFFIX}"
    new_path = out_dir / f"{prefix}-{NEW_SUFFIX}"
    write(missing_path, first_header, missing)
    write(new_path, second_header, new)

    print(f"{first_path}: {len(first)} key(s), {second_path}: {len(second)} key(s)", file=sys.stderr)
    print(f"missing (in {first_path.name}, not in {second_path.name}): {len(missing)} -> {missing_path}", file=sys.stderr)
    print(f"new (in {second_path.name}, not in {first_path.name}): {len(new)} -> {new_path}", file=sys.stderr)
    if changed:
        print(f"note: {changed} order(s) are in both files but with a different Tracking ID", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
