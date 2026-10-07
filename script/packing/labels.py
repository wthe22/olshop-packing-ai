"""Label PDFs: page -> Order ID, label-level fields, slip lines, per-order grouping."""
from __future__ import annotations

import re
import tomllib
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Iterable, Sequence

from pypdf import PdfReader, PdfWriter

from .orders import Line
from .slip import Slip, SlipLine, read_page

_ORDER_ID = re.compile(r"(?<!\d)(5\d{17})(?!\d)")
_LABELLED_ORDER_ID = re.compile(r"OrderI[dD][:：](\d{18})")  # second colon is fullwidth U+FF1A
_WHITESPACE = re.compile(r"\s+")
_TOKEN_TRIM = " \t.,;:"
_SHIP_BY_RAW = re.compile(r"In\s+transit\s+by\s*:\s*(\d{2})/(\d{2})/(\d{4})\s+(\d{2}):(\d{2})")
_SHIP_BY_COLLAPSED = re.compile(r"Intransitby:(\d{2})/(\d{2})/(\d{4})(\d{2}):(\d{2})")

_ILLEGAL_NAME_CHARS = re.compile(r'[\\/:*?"<>|]')
ITEMS_LIMIT = 100  # <items> characters; keeps file names clear of the Windows path limit


class LabelError(Exception):
    """The label PDFs cannot be used; the message is shown to the user as is."""


@dataclass(frozen=True)
class Courier:
    name: str
    text: tuple[str, ...]
    tracking: re.Pattern | None
    tracking_unique: bool


@dataclass(frozen=True)
class LabelOrder:
    order_id: str
    pages: tuple[LabelPage, ...]
    tracking_id: str
    courier: str
    ship_by: datetime | None
    has_slip: bool
    lines: tuple[SlipLine, ...]
    qty_total: int | None
    customer_message: str


def _load_couriers() -> tuple[Courier, ...]:
    with Path(__file__).with_name("couriers.toml").open("rb") as handle:
        data = tomllib.load(handle)
    couriers = []
    for entry in data.get("courier", []):
        tracking = entry.get("tracking")
        couriers.append(
            Courier(
                name=entry["name"],
                text=tuple(entry.get("text", [])),
                tracking=re.compile(tracking) if tracking else None,
                tracking_unique=bool(entry.get("tracking_unique", False)),
            )
        )
    return tuple(couriers)


COURIERS = _load_couriers()


def _combined_tracking() -> re.Pattern | None:
    """Every tracking form in one regex; a form starting with a digit gets a left boundary."""
    parts = []
    for courier in COURIERS:
        if courier.tracking is None:
            continue
        pattern = courier.tracking.pattern
        left = r"(?<!\d)" if pattern[0].isdigit() else ""
        parts.append(left + "(?:" + pattern + r")(?!\d)")
    if not parts:
        return None
    return re.compile("|".join(parts))


_COMBINED_TRACKING = _combined_tracking()


def find_order_id(text: str) -> str | None:
    match = _ORDER_ID.search(text)
    if match is not None:
        return match.group(1)
    # Whitespace is dropped only in this second pass: on J&T labels the Order ID sits on its own
    # line right under the tracking ID, and removing whitespace first would glue the two into one
    # number. So the raw search must come first.
    collapsed = _WHITESPACE.sub("", text)
    match = _LABELLED_ORDER_ID.search(collapsed)
    if match is not None:
        return match.group(1)
    return None


def find_tracking_id(text: str, order_id: str = "") -> str:
    """The most frequent tracking token of the text, "" when none matches a courier form."""
    counts: dict[str, int] = {}
    for token in text.split():
        token = token.strip(_TOKEN_TRIM)
        if token == order_id or not token:
            continue
        for courier in COURIERS:
            if courier.tracking is not None and courier.tracking.fullmatch(token):
                counts[token] = counts.get(token, 0) + 1
                break
    if not counts and _COMBINED_TRACKING is not None:
        # Spaced-out IDX/Tokopedia pages: the tracking ID is only whole in the collapsed text.
        collapsed = _WHITESPACE.sub("", text)
        for match in _COMBINED_TRACKING.finditer(collapsed):
            token = match.group(0)
            if token != order_id:
                counts[token] = counts.get(token, 0) + 1
    if not counts:
        return ""
    return max(counts, key=lambda token: counts[token])


def deduce_courier(text: str, tracking_id: str) -> str:
    """Courier from the label text clues, else an unambiguous tracking form, else ""."""
    lowered = text.lower()
    collapsed = _WHITESPACE.sub("", text).lower()
    for courier in COURIERS:
        if any(clue.lower() in lowered or clue.lower() in collapsed for clue in courier.text):
            return courier.name
    for courier in COURIERS:
        if (
            courier.tracking_unique
            and courier.tracking is not None
            and tracking_id
            and courier.tracking.fullmatch(tracking_id)
        ):
            return courier.name
    return ""


def find_ship_by(text: str) -> datetime | None:
    """The label's 'In transit by: dd/mm/yyyy hh:mm' deadline, None when absent."""
    match = _SHIP_BY_RAW.search(text)
    if match is None:
        match = _SHIP_BY_COLLAPSED.search(_WHITESPACE.sub("", text))
    if match is None:
        return None
    day, month, year, hour, minute = (int(part) for part in match.groups())
    return datetime(year, month, day, hour, minute)


@dataclass(frozen=True)
class LabelPage:
    file: Path
    index: int  # 0-based page index within the file
    order_id: str


class LabelSet:
    """All label pages; the readers are kept open so pages can be copied later."""

    def __init__(
        self,
        readers: dict[Path, PdfReader],
        pages: list[LabelPage],
        parsed: dict[tuple[Path, int], tuple[str, Slip | None]] | None = None,
    ) -> None:
        self._readers = readers
        self.pages = pages
        self._parsed = parsed or {}

    @classmethod
    def open(cls, paths: list[Path]) -> LabelSet:
        readers: dict[Path, PdfReader] = {}
        pages: list[LabelPage] = []
        parsed: dict[tuple[Path, int], tuple[str, Slip | None]] = {}
        for path in paths:
            reader = _read(path)
            readers[path] = reader
            previous: str | None = None
            for index, page in enumerate(reader.pages):
                text, slip = read_page(page)
                parsed[(path, index)] = (text, slip)
                order_id = find_order_id(text)
                if order_id is None:
                    # No Order ID = the previous page's order continues (a long label); a first
                    # page has nothing to continue.
                    if previous is None:
                        raise LabelError(f"{path}: page {index + 1} has no Order ID")
                    order_id = previous
                previous = order_id
                pages.append(LabelPage(file=path, index=index, order_id=order_id))
        return cls(readers, pages, parsed)

    def by_order(self) -> dict[str, list[LabelPage]]:
        by_order: dict[str, list[LabelPage]] = {}
        for page in self.pages:
            by_order.setdefault(page.order_id, []).append(page)
        return by_order

    def orders(self) -> dict[str, LabelOrder]:
        """Per Order ID, all its pages merged, in download order."""
        result: dict[str, LabelOrder] = {}
        for order_id, pages in self.by_order().items():
            first_text = self._parse(pages[0])[0]
            tracking_id = find_tracking_id(first_text, order_id)
            ship_by = None
            has_slip = False
            lines: list[SlipLine] = []
            qty_total: int | None = None
            customer_message = ""
            for page_ref in pages:
                text, slip = self._parse(page_ref)
                if ship_by is None:
                    ship_by = find_ship_by(text)
                if slip is None:
                    continue
                has_slip = True
                lines.extend(slip.lines)
                if slip.qty_total is not None:
                    qty_total = slip.qty_total
                if slip.customer_message:
                    customer_message = slip.customer_message
            result[order_id] = LabelOrder(
                order_id=order_id,
                pages=tuple(pages),
                tracking_id=tracking_id,
                courier=deduce_courier(first_text, tracking_id),
                ship_by=ship_by,
                has_slip=has_slip,
                lines=tuple(lines),
                qty_total=qty_total,
                customer_message=customer_message,
            )
        return result

    def _parse(self, page_ref: LabelPage) -> tuple[str, Slip | None]:
        key = (page_ref.file, page_ref.index)
        if key not in self._parsed:
            self._parsed[key] = read_page(self._page(page_ref))
        return self._parsed[key]

    def _page(self, page_ref: LabelPage):
        reader = self._readers.get(page_ref.file)
        if reader is None:
            raise LabelError(f"{page_ref.file}: page was not read by this LabelSet")
        return reader.pages[page_ref.index]

    def write(self, path: Path, pages: Iterable[LabelPage]) -> None:
        writer = PdfWriter()
        for page in pages:
            reader = self._readers.get(page.file)
            if reader is None:
                raise LabelError(f"{page.file}: page was not read by this LabelSet")
            writer.add_page(reader.pages[page.index])
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("wb") as handle:
            writer.write(handle)


def _read(path: Path) -> PdfReader:
    try:
        return PdfReader(path)
    except Exception as exc:  # pypdf signals missing and unreadable files both this way
        raise LabelError(f"{path}: cannot read label PDF ({exc})") from exc


def group_file_name(number: int, order_count: int, lines: Sequence[Line]) -> str:
    items = " + ".join(
        f"{line.display_name} x{line.quantity}"
        for line in sorted(lines, key=lambda line: line.display_name)
    )
    items = _ILLEGAL_NAME_CHARS.sub("-", items)
    if len(items) > ITEMS_LIMIT:
        items = items[: ITEMS_LIMIT - 1] + "…"  # 100 characters including the ellipsis
    return f"{number:02d} ×{order_count} {items}.pdf"
