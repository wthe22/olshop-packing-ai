"""Picks, what is left, runs of identical contents, day numbering and state (03-pc-script)."""
from __future__ import annotations

from dataclasses import dataclass

from .orders import Order
from .rules import Category


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
