"""Packing-slip table read by text position (03-pc-script "Reading the packing slip")."""
from __future__ import annotations

import re
from dataclasses import dataclass

_HEADER_WORDS = ("Product Name", "SKU", "Seller SKU", "Qty")
_COLUMN_TOLERANCE = 3.0  # a column starts this much left of its header word
_LINE_TOLERANCE = 0.6  # a run a hair above a row's y still belongs to that row
_EXCLUDED = ("Order ID", "Qty Total", "Customer Message")
_DEFAULT_VARIATION = "Default"
_INT = re.compile(r"-?\d+")
_COLONS = (":", "：")  # the slip uses a fullwidth colon the core font cannot draw


@dataclass(frozen=True)
class SlipLine:
    name: str
    variation: str  # "" when the slip says Default
    seller_sku: str
    quantity: int


@dataclass(frozen=True)
class Slip:
    lines: tuple[SlipLine, ...]
    qty_total: int | None
    customer_message: str


def read_slip(page) -> Slip | None:
    """The slip table of one page, or None when the page carries no slip header."""
    return read_page(page)[1]


def read_page(page) -> tuple[str, Slip | None]:
    """The page text and its slip from ONE text extraction (extraction is the slow part)."""
    runs: list[tuple[float, float, str]] = []
    text = page.extract_text(visitor_text=_collector(runs))
    return text, _slip(runs, float(page.mediabox.width))


def _slip(runs: list[tuple[float, float, str]], page_width: float) -> Slip | None:
    header = _find_header(runs, page_width)
    if header is None:
        return None
    header_y, columns = header
    qty_total_y, qty_total = _qty_total(runs)
    rows = _rows(runs, columns, header_y, qty_total_y)
    lower_bound = qty_total_y if qty_total_y is not None else float("-inf")
    lines = tuple(
        SlipLine(
            name=cells["Product Name"],
            variation="" if cells["SKU"] == _DEFAULT_VARIATION else cells["SKU"],
            seller_sku=cells["Seller SKU"],
            quantity=quantity,
        )
        for cells, quantity in _cells(runs, columns, rows, lower_bound)
    )
    return Slip(lines=lines, qty_total=qty_total, customer_message=_customer_message(runs))


def _collector(collected: list[tuple[float, float, str]]):
    """pypdf visitor appending every text run as (x, y, stripped text); y grows upwards."""

    def visitor(text, cm, tm, font_dict, font_size):
        stripped = text.strip()
        if not stripped:
            return
        x = tm[4] * cm[0] + tm[5] * cm[2] + cm[4]
        y = tm[4] * cm[1] + tm[5] * cm[3] + cm[5]
        collected.append((x, y, stripped))

    return visitor


def _find_header(
    runs: list[tuple[float, float, str]], page_width: float
) -> tuple[float, list[tuple[str, float, float]]] | None:
    """The header's y and its columns (word, left edge, right edge), or None without one."""
    positions: dict[str, list[tuple[float, float]]] = {}
    for x, y, text in runs:
        if text in _HEADER_WORDS:
            positions.setdefault(text, []).append((x, y))
    if len(positions) < len(_HEADER_WORDS):
        return None
    for _, name_y in positions["Product Name"]:
        found: dict[str, float] = {}
        for word in _HEADER_WORDS:
            xs = [x for x, y in positions[word] if abs(y - name_y) < _LINE_TOLERANCE]
            if not xs:
                break
            found[word] = min(xs)
        else:
            ordered = sorted(found.items(), key=lambda item: item[1])
            columns = []
            for index, (word, left) in enumerate(ordered):
                right = ordered[index + 1][1] if index + 1 < len(ordered) else page_width
                columns.append((word, left - _COLUMN_TOLERANCE, right))
            return name_y, columns
    return None


def _qty_total(runs: list[tuple[float, float, str]]) -> tuple[float | None, int | None]:
    for x, y, text in runs:
        if text.startswith("Qty Total"):
            for x2, y2, text2 in runs:
                if x2 > x and abs(y2 - y) < _LINE_TOLERANCE and _INT.fullmatch(text2):
                    return y, int(text2)
            return y, None
    return None, None


def _rows(
    runs: list[tuple[float, float, str]],
    columns: list[tuple[str, float, float]],
    header_y: float,
    qty_total_y: float | None,
) -> list[tuple[float, int]]:
    """(y, quantity) per row, top first; a row starts at each Qty value."""
    left, right = next((left, right) for word, left, right in columns if word == "Qty")
    rows: list[tuple[float, int]] = []
    for x, y, text in runs:
        if not (left <= x < right):
            continue
        if y >= header_y - _LINE_TOLERANCE:
            continue
        if qty_total_y is not None and y <= qty_total_y + _LINE_TOLERANCE:
            continue
        if _INT.fullmatch(text):
            rows.append((y, int(text)))
    rows.sort(key=lambda row: -row[0])
    return rows


def _cells(
    runs: list[tuple[float, float, str]],
    columns: list[tuple[str, float, float]],
    rows: list[tuple[float, int]],
    lower_bound: float,
) -> list[tuple[dict[str, str], int]]:
    result = []
    for index, (row_y, quantity) in enumerate(rows):
        low = rows[index + 1][0] if index + 1 < len(rows) else lower_bound
        cells = {}
        for word, left, right in columns:
            pieces = [
                (x, y, text)
                for x, y, text in runs
                if left <= x < right
                and low < y <= row_y + _LINE_TOLERANCE
                and not text.startswith(_EXCLUDED)
            ]
            cells[word] = _join(pieces)
        result.append((cells, quantity))
    return result


def _join(pieces: list[tuple[float, float, str]]) -> str:
    """Top-to-bottom, left-to-right; a wrap right after '-' joins without a space."""
    pieces = sorted(pieces, key=lambda piece: (-piece[1], piece[0]))
    parts: list[str] = []
    for _, _, text in pieces:
        if parts and not parts[-1].endswith("-"):
            parts.append(" ")
        parts.append(text)
    return "".join(parts)


def _customer_message(runs: list[tuple[float, float, str]]) -> str:
    for x, y, text in runs:
        if text == "Customer Message":
            pieces = sorted(
                piece
                for piece in runs
                if abs(piece[1] - y) < _LINE_TOLERANCE and piece[0] > x and piece[2] not in _COLONS
            )
            return " ".join(piece[2] for piece in pieces)
    return ""
