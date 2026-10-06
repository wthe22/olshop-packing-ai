"""categories.toml and the condition language (04-rules-file)."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .orders import Order


class RulesError(Exception):
    """Invalid rules file or condition; the message names the place (file/category, line, col)."""


class Condition:
    """Parsed condition (AST root). Concrete node classes are defined by the implementation."""


@dataclass(frozen=True)
class Category:
    code: str
    name: str
    when: Condition | None  # None only on the last category, and on UNCATEGORISED


UNCATEGORISED = Category(code="?", name="Uncategorised", when=None)


def parse_condition(text: str, *, app_fields: bool = False) -> Condition:
    """Raise RulesError 'line L, col C: expected …'. app_fields allows category/batch/group."""
    raise NotImplementedError


def format_condition(cond: Condition) -> str:
    """Canonical text: lowercase keywords, one space around operators, minimal parentheses."""
    raise NotImplementedError


def evaluate(cond: Condition, order: Order, app_values: dict[str, Any] | None = None) -> bool:
    raise NotImplementedError


def load_rules(path: Path) -> list[Category]:
    raise NotImplementedError


def categorize(order: Order, categories: list[Category]) -> Category:
    """First matching category, else UNCATEGORISED."""
    raise NotImplementedError
