"""Orders CSV: row filter, grouping by Order ID, display names, signatures (01-requirements rules 1-2)."""
from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

PACKING_STATUS = ("Perlu dikirim", "Menunggu pengambilan")  # (Order Status, Order Substatus)


class CsvError(Exception):
    """The CSV cannot be used; the message is shown to the user as is."""


@dataclass(frozen=True)
class Line:
    sku_id: str
    quantity: int
    name: str  # full Product Name
    variation: str  # "" when the CSV says Default
    seller_sku: str
    product_category: str
    display_name: str


@dataclass(frozen=True)
class Order:
    order_id: str
    tracking_id: str
    rts_time: datetime | None
    courier: str  # Shipping Provider Name
    channel: str  # Purchase Channel
    lines: tuple[Line, ...]  # CSV row order

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
    raise NotImplementedError


def signature(lines: tuple[Line, ...] | list[Line]) -> str:
    raise NotImplementedError


def read_orders(path: Path) -> CsvResult:
    raise NotImplementedError
