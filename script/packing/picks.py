"""Merge CSV over slips, picks from what is left, runs, day state, numbering, duplicate guard.

03-pc-script: build_orders (csv wins, warn per difference), check_fields, make_picks, the
day folder state.json, the duplicate guard and redo, and the saved-PDF file names.
"""
from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from .labels import LabelOrder
from .orders import CsvResult, Line, Order, display_name
from .rules import (
    CSV_ONLY_FIELDS,
    ITEM_FIELDS,
    UNCATEGORISED,
    Category,
    evaluate,
    fields_used,
)

STATE_FILE = "state.json"
NAME_LIMIT = 100  # <name> characters; keeps file names clear of the Windows path limit
_ILLEGAL_NAME_CHARS = re.compile(r'[\\/:*?"<>|]')


class PickError(Exception):
    """A pick cannot be run; the message names the pick and the field."""


@dataclass(frozen=True)
class Run:
    number: int  # 1.. inside its saved PDF, printed as 2 digits
    orders: tuple[Order, ...]  # download page order


@dataclass(frozen=True)
class SavedPdf:
    number: int  # 1.. through the day
    category: Category  # the pick; UNCATEGORISED for leftovers matching no entry
    runs: tuple[Run, ...]  # printing order

    @property
    def orders(self) -> tuple[Order, ...]:
        return tuple(order for run in self.runs for order in run.orders)


# ------------------------------------------------------------------ merge (step 4)


def _from_label(label_order: LabelOrder) -> Order:
    """An order built from the label alone: slip lines, no CSV-only values."""
    lines = tuple(
        Line(
            sku_id="",
            quantity=slip_line.quantity,
            name=slip_line.name,
            variation=slip_line.variation,
            seller_sku=slip_line.seller_sku,
            product_category="",
            display_name=display_name(slip_line.name, slip_line.variation),
        )
        for slip_line in label_order.lines
    )
    return Order(
        order_id=label_order.order_id,
        tracking_id=label_order.tracking_id,
        rts_time=None,
        courier=label_order.courier,
        channel="",
        lines=lines,
        ship_by=label_order.ship_by,
        buyer_message=label_order.customer_message,
    )


def _slip_contents(label_order: LabelOrder) -> tuple[tuple[str, str, int], ...]:
    return tuple(sorted((line.name, line.variation, line.quantity) for line in label_order.lines))


def contents_text(contents: tuple[tuple[str, str, int], ...]) -> str:
    """Readable form of a contents key for a difference warning."""
    return " + ".join(
        f"{name} ({variation}) ×{quantity}" if variation else f"{name} ×{quantity}"
        for name, variation, quantity in contents
    )


def build_orders(
    label_orders: dict[str, LabelOrder], csv: CsvResult | None
) -> tuple[dict[str, Order], list[str]]:
    """Per order the CSV wins over the slip and the label; every difference is warned.

    The orders and their download order come from the labels. A CSV that holds none of the
    label orders is warned once and ignored (D4). CSV orders with no label page are warned.
    """
    warnings: list[str] = []
    csv_orders = csv.orders if csv is not None else {}
    uses_csv = bool(csv_orders) and any(order_id in csv_orders for order_id in label_orders)

    if csv is not None and not uses_csv:
        warnings.append(
            f"the orders CSV holds none of the {len(label_orders)} label orders; "
            "the slip and label values are used"
        )

    orders: dict[str, Order] = {}
    for order_id, label_order in label_orders.items():
        order = _from_label(label_order)
        csv_order = csv_orders.get(order_id) if uses_csv else None
        if csv_order is not None:
            if label_order.tracking_id and csv_order.tracking_id != label_order.tracking_id:
                warnings.append(
                    f'{order_id}: tracking_id differs: CSV "{csv_order.tracking_id}", '
                    f'label "{label_order.tracking_id}"'
                )
            if label_order.has_slip and csv_order.contents != _slip_contents(label_order):
                warnings.append(
                    f'{order_id}: lines differ: CSV "{contents_text(csv_order.contents)}", '
                    f'slip "{contents_text(_slip_contents(label_order))}"'
                )
            order = Order(
                order_id=order_id,
                tracking_id=csv_order.tracking_id,
                rts_time=csv_order.rts_time,
                courier=csv_order.courier,
                channel=csv_order.channel,
                lines=csv_order.lines,
                paid_time=csv_order.paid_time,
                created_time=csv_order.created_time,
                ship_by=label_order.ship_by,  # ship_by comes from the label only
                # 03-pc-script per-order data: the slip's Customer Message, else Buyer Message
                buyer_message=csv_order.buyer_message or label_order.customer_message,
            )
        orders[order_id] = order

    if uses_csv:
        for order_id in csv_orders:
            if order_id not in label_orders:
                warnings.append(f"order {order_id} is in the CSV but has no label page")

    for order_id, label_order in label_orders.items():
        if label_order.qty_total is None:
            continue
        total = sum(line.quantity for line in label_order.lines)
        if label_order.qty_total != total:
            warnings.append(
                f"{order_id}: Qty Total {label_order.qty_total} differs from the sum of Qty "
                f"{total}"
            )

    no_tracking = [order_id for order_id, order in orders.items() if not order.tracking_id]
    no_courier = [order_id for order_id, order in orders.items() if not order.courier]
    if no_tracking:
        warnings.append("no tracking ID for: " + ", ".join(no_tracking))
    if no_courier:
        warnings.append("no courier could be deduced for: " + ", ".join(no_courier))

    if not uses_csv and not any(label_order.has_slip for label_order in label_orders.values()):
        warnings.append(
            "no item data (no packing slip and no orders CSV): only label-level fields work"
        )
    return orders, warnings


def csv_is_used(label_orders: dict[str, LabelOrder], csv: CsvResult | None) -> bool:
    """True when the CSV holds a label order (else D4: the slip and label values are used)."""
    return csv is not None and any(order_id in csv.orders for order_id in label_orders)


# ---------------------------------------------------------------- field checks (step 1)


def check_fields(categories: list[Category], has_csv: bool, has_items: bool) -> None:
    """Raise PickError naming the pick and the field a pick uses with no source."""
    for category in categories:
        if category.when is None:
            continue
        used = fields_used(category.when)
        if not has_csv:
            for name in sorted(used):
                if name in CSV_ONLY_FIELDS:
                    raise PickError(
                        f'pick "{category.code}" uses field "{name}", which needs the orders CSV'
                    )
        if not has_items:
            for name in sorted(used):
                if name in ITEM_FIELDS:
                    raise PickError(
                        f'pick "{category.code}" uses field "{name}", which needs item data '
                        "(a packing slip or the orders CSV)"
                    )


# ------------------------------------------------------------- picks and runs (steps 6-7)


def matching_orders(orders, condition) -> list[Order]:
    """The orders matching the condition; a condition of None matches every order."""
    return [order for order in orders if condition is None or evaluate(condition, order)]


def _runs(orders: list[Order]) -> tuple[Run, ...]:
    groups: dict[tuple, list[Order]] = {}
    for order in orders:
        groups.setdefault(order.contents, []).append(order)
    ordered = sorted(groups.items(), key=lambda item: (-len(item[1]), item[0]))
    return tuple(
        Run(number=index + 1, orders=tuple(group))
        for index, (_, group) in enumerate(ordered)
    )


def make_saved_pdf(number: int, category: Category, orders) -> SavedPdf:
    """One saved PDF: the orders grouped into runs, numbered in printing order."""
    return SavedPdf(number=number, category=category, runs=_runs(list(orders)))


def make_picks(
    orders: dict[str, Order], categories: list[Category], first_number: int
) -> list[SavedPdf]:
    """File order, each entry taking matching orders from what is left; leftovers are the rest.

    An entry that takes no order is skipped (no empty PDF). Leftovers matching no entry become
    a final SavedPdf with UNCATEGORISED.
    """
    remaining = dict(orders)
    result: list[SavedPdf] = []
    number = first_number
    for category in categories:
        taken = matching_orders(remaining.values(), category.when)
        if not taken:
            continue
        for order in taken:
            del remaining[order.order_id]
        result.append(make_saved_pdf(number, category, taken))
        number += 1
    leftovers = list(remaining.values())
    if leftovers:
        result.append(make_saved_pdf(number, UNCATEGORISED, leftovers))
    return result


# ---------------------------------------------------------------------- day state


@dataclass(frozen=True)
class SavedEntry:
    order_id: str
    tracking_id: str
    invocation: int
    pdf: int
    run: int
    save_time: str  # ISO with the local UTC offset

    def as_json(self) -> dict[str, object]:
        return {
            "order_id": self.order_id,
            "tracking_id": self.tracking_id,
            "invocation": self.invocation,
            "pdf": self.pdf,
            "run": self.run,
            "save_time": self.save_time,
        }


@dataclass
class DayState:
    day: str
    saved: list[SavedEntry] = field(default_factory=list)

    @property
    def next_pdf_number(self) -> int:
        return max((entry.pdf for entry in self.saved), default=0) + 1

    @property
    def next_invocation(self) -> int:
        return max((entry.invocation for entry in self.saved), default=0) + 1


def load_state(day_dir: Path) -> DayState:
    """The day's state.json; a missing file is an empty state for that folder's day."""
    path = day_dir / STATE_FILE
    if not path.is_file():
        return DayState(day=day_dir.name)
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        saved = [
            SavedEntry(
                order_id=entry["order_id"],
                tracking_id=entry["tracking_id"],
                invocation=entry["invocation"],
                pdf=entry["pdf"],
                run=entry["run"],
                save_time=entry["save_time"],
            )
            for entry in data["saved"]
        ]
    except (OSError, ValueError, KeyError, TypeError) as exc:
        raise PickError(f"cannot read {path}: {exc}") from exc
    return DayState(day=data.get("day", day_dir.name), saved=saved)


def save_state(day_dir: Path, state: DayState) -> None:
    day_dir.mkdir(parents=True, exist_ok=True)
    data = {"day": state.day, "saved": [entry.as_json() for entry in state.saved]}
    (day_dir / STATE_FILE).write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


@dataclass(frozen=True)
class Duplicate:
    order_id: str
    tracking_id: str
    save_time: str  # the time the order was first saved today


def drop_saved(
    orders: dict[str, Order], state: DayState
) -> tuple[dict[str, Order], list[Duplicate]]:
    """The duplicate guard: drop orders already saved today, with their first save time."""
    first: dict[str, SavedEntry] = {}
    for entry in state.saved:
        first.setdefault(entry.order_id, entry)
    kept: dict[str, Order] = {}
    duplicates: list[Duplicate] = []
    for order_id, order in orders.items():
        entry = first.get(order_id)
        if entry is None:
            kept[order_id] = order
        else:
            duplicates.append(Duplicate(order_id, entry.tracking_id, entry.save_time))
    return kept, duplicates


def redo(state: DayState) -> tuple[DayState, list[int]]:
    """Forget the day's last invocation; return the saved-PDF numbers to delete."""
    if not state.saved:
        return state, []
    last = max(entry.invocation for entry in state.saved)
    numbers = sorted({entry.pdf for entry in state.saved if entry.invocation == last})
    kept = [entry for entry in state.saved if entry.invocation != last]
    return DayState(day=state.day, saved=kept), numbers


def saved_entries(pdfs: list[SavedPdf], invocation: int, save_time: str) -> list[SavedEntry]:
    """One entry per saved order: its saved PDF number and run number."""
    entries: list[SavedEntry] = []
    for pdf in pdfs:
        for run in pdf.runs:
            for order in run.orders:
                entries.append(
                    SavedEntry(
                        order_id=order.order_id,
                        tracking_id=order.tracking_id,
                        invocation=invocation,
                        pdf=pdf.number,
                        run=run.number,
                        save_time=save_time,
                    )
                )
    return entries


def saved_pdf_file_name(pdf: SavedPdf) -> str:
    """`<n> <code> <name> ×<orders>.pdf`; illegal Windows characters become '-'.

    The <name> part is cut at 100 characters (ending '…'), as in the old group file names.
    """
    name = _ILLEGAL_NAME_CHARS.sub("-", pdf.category.name)
    if len(name) > NAME_LIMIT:
        name = name[: NAME_LIMIT - 1] + "…"
    return _ILLEGAL_NAME_CHARS.sub(
        "-", f"{pdf.number} {pdf.category.code} {name} ×{len(pdf.orders)}.pdf"
    )
