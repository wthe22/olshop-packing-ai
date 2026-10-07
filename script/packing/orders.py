"""Orders CSV: row filter, grouping by Order ID, display names, signatures (01-requirements rules 1-2)."""
from __future__ import annotations

import csv
from dataclasses import dataclass, replace
from datetime import datetime
from pathlib import Path

PACKING_STATUS = ("Perlu dikirim", "Menunggu pengambilan")  # (Order Status, Order Substatus)
TIME_FORMAT = "%d/%m/%Y %H:%M:%S"
DEFAULT_VARIATION = "Default"
# Every value arrives with a trailing tab (business-process 02); strip tabs and spaces.
TRIM = " \t"

REQUIRED_COLUMNS = (
    "Order ID",
    "Order Status",
    "Order Substatus",
    "Tracking ID",
    "RTS Time",
    "Shipping Provider Name",
    "Purchase Channel",
    "SKU ID",
    "Quantity",
    "Product Name",
    "Variation",
    "Seller SKU",
    "Product Category",
)


class CsvError(Exception):
    """The CSV cannot be used; the message is shown to the user as is."""


@dataclass(frozen=True)
class Line:
    sku_id: str  # "" when the line comes from a packing slip
    quantity: int
    name: str  # full Product Name
    variation: str  # "" when the CSV/slip says Default
    seller_sku: str
    product_category: str  # "" when the line comes from a packing slip
    display_name: str


@dataclass(frozen=True)
class Order:
    order_id: str
    tracking_id: str
    rts_time: datetime | None
    courier: str  # Shipping Provider Name, or deduced from the label
    channel: str  # Purchase Channel
    lines: tuple[Line, ...]  # CSV row order / slip row order
    paid_time: datetime | None = None
    created_time: datetime | None = None
    ship_by: datetime | None = None  # label "In transit by"
    buyer_message: str = ""

    @property
    def contents(self) -> tuple[tuple[str, str, int], ...]:
        """Identical-contents key (01-requirements): per line (name, variation, quantity), sorted."""
        return tuple(sorted((line.name, line.variation, line.quantity) for line in self.lines))

    @property
    def signature(self) -> str:
        return signature(self.lines)

    @property
    def total_quantity(self) -> int:
        return sum(line.quantity for line in self.lines)

    @property
    def distinct_items(self) -> int:
        return len(self.lines)


@dataclass(frozen=True)
class CsvResult:
    orders: dict[str, Order]  # by Order ID, in first-seen CSV order
    kept_rows: int
    ignored_rows: int


def display_name(product_name: str, variation: str) -> str:
    short = product_name.split("|", 1)[0].strip()
    variant = variation.strip()
    if variant and variant != DEFAULT_VARIATION:
        return f"{short} — {variant}"
    return short


def signature(lines: tuple[Line, ...] | list[Line]) -> str:
    ordered = sorted(lines, key=lambda line: line.sku_id)
    return "+".join(f"{line.sku_id}×{line.quantity}" for line in ordered)


def read_orders(path: Path) -> CsvResult:
    try:
        handle = open(path, encoding="utf-8-sig", newline="")
    except OSError as exc:
        raise CsvError(f"cannot read {path}: {exc}") from exc
    with handle:
        reader = csv.DictReader(handle)
        fieldnames = reader.fieldnames or []
        missing = [name for name in REQUIRED_COLUMNS if name not in fieldnames]
        if missing:
            raise CsvError("missing required column(s): " + ", ".join(missing))
        rows = list(reader)

    orders: dict[str, Order] = {}
    kept_rows = 0
    ignored_rows = 0
    unfinished: set[str] = set()
    for row in rows:
        values = {name: (row.get(name) or "").strip(TRIM) for name in REQUIRED_COLUMNS}
        if (values["Order Status"], values["Order Substatus"]) != PACKING_STATUS:
            ignored_rows += 1
            continue
        kept_rows += 1
        order_id = values["Order ID"]
        sku_id = values["SKU ID"]
        tracking_id = values["Tracking ID"]
        if not tracking_id or not sku_id:
            unfinished.add(order_id)
        variation = values["Variation"]
        if variation == DEFAULT_VARIATION:
            variation = ""
        line = Line(
            sku_id=sku_id,
            quantity=int(values["Quantity"]),
            name=values["Product Name"],
            variation=variation,
            seller_sku=values["Seller SKU"],
            product_category=values["Product Category"],
            display_name=display_name(values["Product Name"], variation),
        )
        rts_raw = values["RTS Time"]
        rts_time = datetime.strptime(rts_raw, TIME_FORMAT) if rts_raw else None
        order = orders.get(order_id)
        if order is None:
            orders[order_id] = Order(
                order_id=order_id,
                tracking_id=tracking_id,
                rts_time=rts_time,
                courier=values["Shipping Provider Name"],
                channel=values["Purchase Channel"],
                lines=(line,),
            )
        else:
            orders[order_id] = replace(order, lines=order.lines + (line,))

    if unfinished:
        raise CsvError(
            "kept rows with an empty Tracking ID or SKU ID (the export was made before "
            "shipment was arranged): " + ", ".join(sorted(unfinished))
        )
    return CsvResult(orders=orders, kept_rows=kept_rows, ignored_rows=ignored_rows)
