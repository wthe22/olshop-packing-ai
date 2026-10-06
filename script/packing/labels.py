"""Label PDFs: page -> Order ID, page copying into per-group PDFs, file names."""
from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence

from pypdf import PdfReader, PdfWriter

from .orders import Line

_ORDER_ID = re.compile(r"(?<!\d)(5\d{17})(?!\d)")
_LABELLED_ORDER_ID = re.compile(r"OrderI[dD][:：](\d{18})")  # second colon is fullwidth U+FF1A
_WHITESPACE = re.compile(r"\s+")

_ILLEGAL_NAME_CHARS = re.compile(r'[\\/:*?"<>|]')
ITEMS_LIMIT = 100  # <items> characters; keeps file names clear of the Windows path limit


class LabelError(Exception):
    """The label PDFs cannot be used; the message is shown to the user as is."""


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


@dataclass(frozen=True)
class LabelPage:
    file: Path
    index: int  # 0-based page index within the file
    order_id: str


class LabelSet:
    """All label pages; the readers are kept open so pages can be copied later."""

    def __init__(self, readers: dict[Path, PdfReader], pages: list[LabelPage]) -> None:
        self._readers = readers
        self.pages = pages

    @classmethod
    def open(cls, paths: list[Path]) -> LabelSet:
        readers: dict[Path, PdfReader] = {}
        pages: list[LabelPage] = []
        for path in paths:
            reader = _read(path)
            readers[path] = reader
            previous: str | None = None
            for index, page in enumerate(reader.pages):
                order_id = find_order_id(page.extract_text())
                if order_id is None:
                    # No Order ID = the previous page's order continues (a long label); a first
                    # page has nothing to continue.
                    if previous is None:
                        raise LabelError(f"{path}: page {index + 1} has no Order ID")
                    order_id = previous
                previous = order_id
                pages.append(LabelPage(file=path, index=index, order_id=order_id))
        return cls(readers, pages)

    def by_order(self) -> dict[str, list[LabelPage]]:
        by_order: dict[str, list[LabelPage]] = {}
        for page in self.pages:
            by_order.setdefault(page.order_id, []).append(page)
        return by_order

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
