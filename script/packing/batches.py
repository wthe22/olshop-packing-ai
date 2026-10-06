"""state.json, new/removed orders, categories and group numbering (01-requirements rules 3-7)."""
from __future__ import annotations

from dataclasses import dataclass

from .orders import Order
from .rules import Category


@dataclass(frozen=True)
class Group:
    number: int
    category: Category
    orders: tuple[Order, ...]  # rule 7 order = label page order

    @property
    def lines(self):
        """The group's items (all its orders share the signature): lines of the first order."""
        return self.orders[0].lines


@dataclass(frozen=True)
class Batch:
    number: int
    groups: tuple[Group, ...]  # in group-number order
    removed: tuple[str, ...]  # Order IDs of earlier batches missing from this CSV
