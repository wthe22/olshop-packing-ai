"""A4 packing list in the layouts full, summary, pick (03-pc-script)."""
from __future__ import annotations

from collections import Counter
from datetime import datetime
from pathlib import Path
from typing import Sequence

from fpdf import FPDF

from .picks import Run, SavedPdf

LAYOUTS = ("full", "summary", "pick")
SCOPES = ("per-pdf", "whole")

_FONT_DIR = Path(r"C:\Windows\Fonts")
_FONTS = {"A": {"": "arial.ttf", "B": "arialbd.ttf"}, "M": {"": "consola.ttf"}}

# millimetres; the sizes and spacings of the layout prototype
_LH = 5.2      # item line height
_TH = 4.6      # tracking-ID row height
_NCOL = 6      # tracking-ID columns
_CW = 29.6     # tracking-ID column width
_X_ITEMS = 22  # item text left edge
_COL_X = (10.0, 106.0)  # pick-summary column left edges
_COL_W = 80.0           # pick-summary name wrap width inside the 96 mm column pitch
_ROW_H = 5.0            # pick-summary row height
_BAR_H = 6.5            # heading bar height

_NO_ITEM_DATA = "no item data: the packing list needs labels with packing slip or the orders CSV"


class PackingListError(Exception):
    """The PDF cannot be written; the message is shown to the user as is."""


def write_packing_list(
    path: Path,
    layout: str,
    *,
    day: str,
    pdfs: Sequence[SavedPdf],
    printed: datetime,
) -> int:
    """Write the packing list for `layout`; return the number of pages.

    One call writes one file: the caller passes one saved PDF for scope `per-pdf` and
    all saved PDFs for scope `whole`.
    """
    if layout not in LAYOUTS:
        raise ValueError(f"unknown layout {layout!r}; expected one of {', '.join(LAYOUTS)}")
    if not pdfs:
        raise ValueError("no saved PDFs to write")
    # The lists describe items and runs; an order whose lines are unknown cannot be listed.
    for saved in pdfs:
        for order in saved.orders:
            if not order.lines:
                raise PackingListError(_NO_ITEM_DATA)

    orders = sum(len(saved.orders) for saved in pdfs)
    units = sum(o.total_quantity for saved in pdfs for o in saved.orders)
    runs = sum(len(saved.runs) for saved in pdfs)
    subtitle = (
        f"{orders:,} orders · {units:,} units · {runs:,} runs"
        f" · printed {printed:%H:%M}"
    )

    pdf = _List(title=_title(day, pdfs), subtitle=subtitle, layout=layout)
    pdf.add_page()
    if layout != "pick":
        pdf.draw_pdfs(pdfs)
    pdf.draw_pick_summary(pdfs)
    pdf.output(str(path))
    return pdf.pages_count


def _title(day: str, pdfs: Sequence[SavedPdf]) -> str:
    """Line 1: one saved PDF is named; several cover a number range."""
    if len(pdfs) == 1:
        saved = pdfs[0]
        return (
            f"Packing list · {day} · {saved.number}  "
            f"{saved.category.code}  {saved.category.name}"
        )
    return f"Packing list · {day} · saved PDFs {pdfs[0].number}-{pdfs[-1].number}"


def _add_fonts(pdf: FPDF) -> None:
    for family, styles in _FONTS.items():
        for style, filename in styles.items():
            font = _FONT_DIR / filename
            if not font.is_file():
                raise PackingListError(
                    f"font file not found: {font} (Windows fonts folder: {_FONT_DIR})"
                )
            pdf.add_font(family, style, str(font))


def _run_items(run: Run) -> list[str]:
    """The run's identical lines as display text; any order of the run carries them."""
    if not run.orders:
        return []
    return sorted(f"{line.display_name}  ×{line.quantity}" for line in run.orders[0].lines)


def _pick_entries(pdfs: Sequence[SavedPdf]) -> list[tuple[int, str]]:
    """One (units, display name) entry per (name, variation), sorted by display name.

    A slip line carries no SKU ID, so the item key is (name, variation).
    """
    units: Counter[tuple[str, str]] = Counter()
    names: dict[tuple[str, str], str] = {}
    for saved in pdfs:
        for order in saved.orders:
            for line in order.lines:
                key = (line.name, line.variation)
                units[key] += line.quantity
                names.setdefault(key, line.display_name)
    return [
        (units[key], names[key]) for key in sorted(units, key=lambda k: (names[k], k[0], k[1]))
    ]


def _pick_fit(entries: list[tuple[int, str]], avail: float, pdf: _List) -> int:
    """Largest count of the leading entries whose balanced two-column split fits `avail`."""
    heights = []
    for _units, name in entries:
        pdf.set_font("A", "", 10)
        lines = pdf.multi_cell(_COL_W, _ROW_H, name, dry_run=True, output="LINES")
        heights.append(_ROW_H * len(lines) + 0.4)
    prefix = [0.0]
    for height in heights:
        prefix.append(prefix[-1] + height)
    for count in range(len(entries), 0, -1):
        left = (count + 1) // 2
        if max(prefix[left], prefix[count] - prefix[left]) <= avail:
            return count
    return 0


class _List(FPDF):
    def __init__(self, *, title: str, subtitle: str, layout: str) -> None:
        super().__init__("P", "mm", "A4")
        self._title = title
        self._subtitle = subtitle
        self._layout = layout
        self.set_margins(10, 10, 10)
        self.set_auto_page_break(False)
        self.alias_nb_pages()
        _add_fonts(self)
        self._bottom = self.h - 10

    def header(self) -> None:
        self.set_font("A", "B", 13)
        self.cell(150, 7, self._title)
        self.set_font("A", "", 9.5)
        self.cell(40, 7, f"page {self.page_no()}/{{nb}}", align="R",
                  new_x="LMARGIN", new_y="NEXT")
        self.cell(0, 5, self._subtitle, new_x="LMARGIN", new_y="NEXT")
        self.ln(1)

    def _need(self, height: float, continued: str | None = None) -> bool:
        """Start a new page if `height` does not fit; repeat `continued` on the new page."""
        if self.get_y() + height > self._bottom:
            self.add_page()
            if continued is not None:
                self.set_font("A", "", 9.5)
                self.set_text_color(90)
                self.cell(0, 5, continued, new_x="LMARGIN", new_y="NEXT")
                self.set_text_color(0)
            return True
        return False

    def draw_pdfs(self, pdfs: Sequence[SavedPdf]) -> None:
        for saved in pdfs:
            orders = len(saved.orders)
            units = sum(o.total_quantity for o in saved.orders)
            self._need(_BAR_H + _LH * 2)
            self.set_fill_color(225, 225, 225)
            self.set_font("A", "B", 11)
            self.cell(
                150, _BAR_H,
                f" {saved.number}  {saved.category.code}  {saved.category.name}",
                fill=True,
            )
            self.set_font("A", "", 9.5)
            self.cell(40, _BAR_H, f"{orders:,} orders · {units:,} units", fill=True,
                      align="R", new_x="LMARGIN", new_y="NEXT")
            for run in saved.runs:
                self._run(saved, run)
            self.ln(1.5)

    def _run(self, saved: SavedPdf, run: Run) -> None:
        label = f"{saved.number}-{run.number:02d}"
        items = _run_items(run)
        tracking = self._layout == "full"
        self._need(_LH * len(items) + (_TH + 1 if tracking else 0) + 1.2)
        top = self.get_y()
        self.set_font("A", "B", 11.5)
        self.cell(12, _LH, label)
        self.set_font("A", "", 10.5)
        for i, text in enumerate(items):
            self.set_xy(_X_ITEMS, top + _LH * i)
            self.cell(158, _LH, text)
        self.set_xy(180, top)
        self.set_font("A", "B", 11.5)
        self.cell(20, _LH, f"{len(run.orders):,}", align="R")
        self.set_xy(10, top + _LH * len(items))
        if tracking:
            self._tracking(label, run, items)
        line_y = self.get_y() + 0.6
        self.set_draw_color(170, 170, 170)
        self.line(10, line_y, 200, line_y)
        self.set_xy(10, line_y + 0.6)

    def _tracking(self, label: str, run: Run, items: list[str]) -> None:
        ids = [order.tracking_id for order in run.orders]
        continued = f"{label} (continued)  " + "  /  ".join(items)
        self.set_y(self.get_y() + 0.6)
        self.set_font("M", "", 10)
        for start in range(0, len(ids), _NCOL):
            if self._need(_TH, continued):
                self.set_font("M", "", 10)
            row_y = self.get_y()
            for col, tracking_id in enumerate(ids[start:start + _NCOL]):
                self.set_xy(_X_ITEMS + col * _CW, row_y)
                self.cell(_CW, _TH, tracking_id)
            self.set_xy(10, row_y + _TH)

    def draw_pick_summary(self, pdfs: Sequence[SavedPdf]) -> None:
        entries = _pick_entries(pdfs)
        if not entries:
            return
        self._need(_BAR_H + min(len(entries), 4) * (_ROW_H + 1) + 2)
        self.ln(1)
        self.set_font("A", "B", 11)
        self.set_fill_color(225, 225, 225)
        self.cell(190, _BAR_H, " Pick summary (units to take from stock)", fill=True,
                  new_x="LMARGIN", new_y="NEXT")
        top = self.get_y()
        index = 0
        while index < len(entries):
            take = _pick_fit(entries[index:], self._bottom - top, self)
            if take == 0:
                self.add_page()
                top = self.get_y()
                take = _pick_fit(entries[index:], self._bottom - top, self) or 1
            chunk = entries[index:index + take]
            half = (len(chunk) + 1) // 2
            for column, part in enumerate((chunk[:half], chunk[half:])):
                self.set_xy(_COL_X[column], top)
                for units, name in part:
                    x, y = self.get_x(), self.get_y()
                    self.set_font("A", "", 10)
                    lines = self.multi_cell(_COL_W, _ROW_H, name, dry_run=True, output="LINES")
                    self.set_font("A", "B", 10.5)
                    self.cell(12, _ROW_H, f"{units:,}", align="R")
                    self.set_font("A", "", 10)
                    for i, text in enumerate(lines):
                        self.set_xy(x + 14, y + _ROW_H * i)
                        self.cell(_COL_W, _ROW_H, text)
                    self.set_xy(x, y + _ROW_H * len(lines) + 0.4)
            index += take
            if index < len(entries):
                self.add_page()
                top = self.get_y()
