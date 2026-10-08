"""Command line: `prepare` (03-pc-script) — read, merge, picks, write, summary."""
from __future__ import annotations

import argparse
import glob
import logging
import sys
from datetime import date, datetime
from pathlib import Path
from typing import Callable, Sequence

from .labels import LabelError, LabelSet
from .orders import CsvError, read_orders
from .packing_list import LAYOUTS, PackingListError, write_packing_list
from .picks import (
    Duplicate,
    PickError,
    SavedPdf,
    build_orders,
    check_fields,
    csv_is_used,
    drop_saved,
    load_state,
    make_picks,
    make_saved_pdf,
    matching_orders,
    redo,
    save_state,
    saved_entries,
    saved_pdf_file_name,
)
from .rules import (
    UNCATEGORISED,
    Category,
    RulesError,
    format_condition,
    load_rules,
    parse_condition,
)

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RULES = REPO_ROOT / "categories.toml"
SCOPES = ("per-pdf", "whole", "none")
SUMMARY_FILE = "summary.txt"


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m packing", description="Prepare the day's picks from label PDFs."
    )
    sub = parser.add_subparsers(dest="command", required=True)
    prepare = sub.add_parser("prepare", help="read the labels and CSV, write the pick PDFs")
    prepare.add_argument("--labels", nargs="+", required=True, metavar="LABEL.pdf")
    prepare.add_argument("--csv", metavar="ORDERS.csv")
    prepare.add_argument("--rules", default=str(DEFAULT_RULES), metavar="CATEGORIES.toml")
    prepare.add_argument("--interactive", action="store_true")
    prepare.add_argument("--layout", default="full", metavar="LAYOUTS")
    prepare.add_argument("--packing-list", dest="packing_list", choices=SCOPES, default="per-pdf")
    prepare.add_argument("--day", metavar="YYYY-MM-DD")
    prepare.add_argument("--work", default="work", metavar="DIR")
    prepare.add_argument("--redo", action="store_true")
    return parser


def main(argv: Sequence[str] | None = None, *, input_fn: Callable[[str], str] = input) -> int:
    # pypdf logs harmless font notes ("MERG NOT subset") while copying label pages.
    logging.getLogger("pypdf").setLevel(logging.ERROR)
    parser = _build_parser()
    args = parser.parse_args(argv)
    try:
        return _prepare(args, input_fn)
    except (RulesError, CsvError, LabelError, PickError, PackingListError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


def _expand(patterns: Sequence[str]) -> list[Path]:
    """Wildcards expanded in the program: Windows shells do not expand them."""
    paths: list[Path] = []
    for pattern in patterns:
        matches = sorted(glob.glob(pattern))
        if matches:
            paths.extend(Path(match) for match in matches)
        else:
            paths.append(Path(pattern))
    return paths


def _layouts(text: str) -> list[str]:
    layouts = [part.strip() for part in text.split(",") if part.strip()]
    if not layouts:
        raise PickError("no layout given")
    for layout in layouts:
        if layout not in LAYOUTS:
            raise PickError(f'unknown layout "{layout}"; expected one of {", ".join(LAYOUTS)}')
    return layouts


def _delete_invocation_files(day_dir: Path, numbers: list[int]) -> None:
    """Delete the last invocation's saved PDFs and packing lists (redo)."""
    if not day_dir.is_dir():
        return
    targets: list[Path] = []
    for number in numbers:
        targets += list(day_dir.glob(f"{number} *.pdf"))
        targets += list(day_dir.glob(f"packing-list-{number}.pdf"))
        targets += list(day_dir.glob(f"packing-list-{number}-*.pdf"))
    # The whole-scope list overwrites the same name every invocation, so removing it is safe.
    targets.append(day_dir / "packing-list.pdf")
    targets += [day_dir / f"packing-list-{layout}.pdf" for layout in LAYOUTS]
    for path in set(targets):
        path.unlink(missing_ok=True)


def _prepare(args: argparse.Namespace, input_fn: Callable[[str], str]) -> int:
    layouts = _layouts(args.layout)
    day = args.day or date.today().isoformat()
    day_dir = Path(args.work) / day

    categories = load_rules(Path(args.rules))
    if not args.interactive and not args.csv:
        # Fail before the slow label reading; item fields are checked once the labels are read.
        check_fields(categories, has_csv=False, has_items=True)
    csv_result = read_orders(Path(args.csv)) if args.csv else None
    label_paths = _expand(args.labels)
    label_set = LabelSet.open(label_paths)
    label_orders = label_set.orders()

    uses_csv = csv_is_used(label_orders, csv_result)
    slip_pages = sum(
        len(label_order.pages) for label_order in label_orders.values() if label_order.has_slip
    )
    has_items = uses_csv or slip_pages > 0

    orders, warnings = build_orders(label_orders, csv_result)

    if not args.interactive:
        check_fields(categories, uses_csv, has_items)

    state = load_state(day_dir)
    if args.redo:
        state, delete_numbers = redo(state)
        _delete_invocation_files(day_dir, delete_numbers)

    kept, duplicates = drop_saved(orders, state)
    if duplicates:
        warnings.append(
            f"{len(duplicates)} order(s) already saved today: "
            + ", ".join(
                f"{dup.order_id} ({dup.tracking_id}) first saved {dup.save_time}"
                for dup in duplicates
            )
        )

    if not kept:
        if orders:
            print(f"nothing to save: all {len(orders)} orders were already saved today")
        else:
            print("nothing to save: no orders")
        return 0

    if args.interactive:
        pdfs = _interactive(categories, kept, state.next_pdf_number, uses_csv, has_items, input_fn)
    else:
        pdfs = make_picks(kept, categories, state.next_pdf_number)

    now = datetime.now().astimezone()
    save_time = now.isoformat(timespec="seconds")
    invocation = state.next_invocation

    for pdf in pdfs:
        pages = [page for order in pdf.orders for page in label_orders[order.order_id].pages]
        label_set.write(day_dir / saved_pdf_file_name(pdf), pages)

    packed_names, packing_skipped = _write_packing_lists(
        args.packing_list, layouts, pdfs, day, now, day_dir, has_items
    )
    if packing_skipped:
        if not has_items:
            warnings.append("no item data: the packing list was skipped")
        else:
            warnings.append("some orders have no item data: the packing list was skipped")

    state.saved.extend(saved_entries(pdfs, invocation, save_time))
    save_state(day_dir, state)

    text = _summary(
        day=day,
        label_files=len(label_paths),
        csv_name=Path(args.csv).name if args.csv else None,
        pages_read=len(label_set.pages),
        orders_total=len(orders),
        not_in_csv=(
            sum(1 for order_id in label_orders if order_id not in csv_result.orders)
            if csv_result is not None
            else None
        ),
        has_slips=slip_pages > 0,
        slip_pages=slip_pages,
        uses_csv=uses_csv,
        pdfs=pdfs,
        duplicates=duplicates,
        warnings=warnings,
        day_dir=day_dir,
        packed_names=packed_names,
    )
    print(text)
    (day_dir / SUMMARY_FILE).write_text(text + "\n", encoding="utf-8")
    return 0


def _write_packing_lists(
    scope: str,
    layouts: list[str],
    pdfs: list[SavedPdf],
    day: str,
    printed: datetime,
    day_dir: Path,
    has_items: bool,
) -> tuple[list[str], bool]:
    """Write the packing lists; return the file names and whether they were skipped."""
    if scope == "none":
        return [], False
    if not has_items or any(not order.lines for pdf in pdfs for order in pdf.orders):
        return [], True
    names: list[str] = []
    if scope == "per-pdf":
        for pdf in pdfs:
            for layout in layouts:
                name = (
                    f"packing-list-{pdf.number}.pdf"
                    if len(layouts) == 1
                    else f"packing-list-{pdf.number}-{layout}.pdf"
                )
                write_packing_list(day_dir / name, layout, day=day, pdfs=[pdf], printed=printed)
                names.append(name)
    else:  # whole
        for layout in layouts:
            name = "packing-list.pdf" if len(layouts) == 1 else f"packing-list-{layout}.pdf"
            write_packing_list(day_dir / name, layout, day=day, pdfs=pdfs, printed=printed)
            names.append(name)
    return names, False


def _interactive(
    categories: list[Category],
    orders: dict,
    first_number: int,
    has_csv: bool,
    has_items: bool,
    input_fn: Callable[[str], str],
) -> list[SavedPdf]:
    """The numbered menu of 03-pc-script: a number saves a pick, `c` a condition, `r` the rest."""
    remaining = dict(orders)
    chosen: list[tuple[Category, list]] = []

    def take(category: Category, condition) -> None:
        check_fields([category], has_csv, has_items)
        taken = matching_orders(remaining.values(), condition)
        for order in taken:
            del remaining[order.order_id]
        if taken:
            chosen.append((category, taken))

    while remaining:
        menu = [
            (index, category, matching_orders(remaining.values(), category.when))
            for index, category in enumerate(categories, start=1)
        ]
        menu = [entry for entry in menu if entry[2]]
        print(f"{len(remaining)} orders left")
        for index, category, taken in menu:
            print(f"  {index}  {category.code}  {category.name}    {len(taken)}")
        print(f"1-{len(menu)} = save that pick · c = type a condition · r = save the rest")
        try:
            choice = input_fn("> ").strip()
        except StopIteration:
            choice = "r"
        if choice == "r":
            take(UNCATEGORISED, None)
            break
        if choice == "c":
            text = input_fn("condition: ")
            condition = parse_condition(text)
            take(Category(code="C", name=format_condition(condition), when=condition), condition)
            continue
        if choice.isdigit() and 1 <= int(choice) <= len(menu):
            _, category, _ = menu[int(choice) - 1]
            take(category, category.when)
            continue
        # anything else: show the menu again

    pdfs: list[SavedPdf] = []
    number = first_number
    for category, taken in chosen:
        pdfs.append(make_saved_pdf(number, category, taken))
        number += 1
    return pdfs


def _summary(
    *,
    day: str,
    label_files: int,
    csv_name: str | None,
    pages_read: int,
    orders_total: int,
    not_in_csv: int | None,
    has_slips: bool,
    slip_pages: int,
    uses_csv: bool,
    pdfs: list[SavedPdf],
    duplicates: list[Duplicate],
    warnings: list[str],
    day_dir: Path,
    packed_names: list[str],
) -> str:
    files = f"{label_files} label file" + ("" if label_files == 1 else "s")
    lines = [f"Day {day} · {files} · {csv_name or 'no CSV'}"]

    pages = f"  Pages: {pages_read} read · {orders_total} orders"
    if not_in_csv is not None:
        pages += f" · {not_in_csv} not in the CSV"
    lines.append(pages)

    sources = []
    if has_slips:
        sources.append(f"slips ({slip_pages} pages)")
    if uses_csv:
        sources.append("CSV")
    lines.append("  Item data: " + (", ".join(sources) if sources else "none"))

    for pdf in pdfs:
        kind = "Rest" if pdf.category.when is None else "Pick"
        lines.append(
            f"  {kind} {pdf.category.code}  {pdf.category.name}: {len(pdf.orders)} orders "
            f"→ {saved_pdf_file_name(pdf)}"
        )

    lines.append(f"  Duplicates: {len(duplicates)}")
    lines.append(f"  Warnings: {len(warnings)}" if warnings else "  Warnings: none")
    for warning in warnings:
        lines.append(f"    - {warning}")

    saved = f"{len(pdfs)} saved PDF" + ("" if len(pdfs) == 1 else "s")
    written = ", ".join(packed_names) if packed_names else "no packing list"
    lines.append(f"Written: {day_dir.as_posix()}/  ({saved}, {written})")
    return "\n".join(lines)


if __name__ == "__main__":
    raise SystemExit(main())
