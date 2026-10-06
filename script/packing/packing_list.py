"""A4 packing list in the layouts full, summary, pick (03-pc-script)."""
from __future__ import annotations

from collections import Counter
from datetime import datetime
from pathlib import Path

from fpdf import FPDF

from .batches import Batch, Group
from .rules import Category

LAYOUTS = ("full", "summary", "pick")

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


class PackingListError(Exception):
    """The PDF cannot be written; the message is shown to the user as is."""


def write_packing_list(
    path: Path,
    layout: str,
    *,
    day: str,
    batch: Batch,
    printed: datetime,
    missing_labels: frozenset[str] = frozenset(),
) -> int:
    """Write the packing list for `layout`; return the number of pages."""
    if layout not in LAYOUTS:
        raise ValueError(f"unknown layout {layout!r}; expected one of {', '.join(LAYOUTS)}")

    orders = sum(len(g.orders) for g in batch.groups)
    units = sum(o.total_quantity for g in batch.groups for o in g.orders)
    title = f"Packing list · {day} · Batch {batch.number}"
    subtitle = (
        f"{orders:,} orders · {units:,} units · {len(batch.groups):,} groups"
        f" · printed {printed:%H:%M}"
    )

    pdf = _List(title=title, subtitle=subtitle, layout=layout)
    pdf.add_page()
    if layout != "pick":
        pdf.draw_groups(batch, missing_labels)
    pdf.draw_pick_summary(batch)
    pdf.output(str(path))
    return pdf.pages_count


def _add_fonts(pdf: FPDF) -> None:
    for family, styles in _FONTS.items():
        for style, filename in styles.items():
            font = _FONT_DIR / filename
            if not font.is_file():
                raise PackingListError(
                    f"font file not found: {font} (Windows fonts folder: {_FONT_DIR})"
                )
            pdf.add_font(family, style, str(font))


def _by_category(batch: Batch) -> list[tuple[Category, list[Group]]]:
    """Categories in the order they first appear in batch.groups, with their groups."""
    seen: dict[str, list[Group]] = {}
    found: list[Category] = []
    for group in batch.groups:
        code = group.category.code
        if code not in seen:
            seen[code] = []
            found.append(group.category)
        seen[code].append(group)
    return [(category, seen[category.code]) for category in found]


def _pick_entries(batch: Batch) -> list[tuple[int, str]]:
    """One (units, display name) entry per SKU ID, sorted by display name then SKU ID."""
    units: Counter[str] = Counter()
    names: dict[str, str] = {}
    for group in batch.groups:
        for order in group.orders:
            for line in order.lines:
                units[line.sku_id] += line.quantity
                names.setdefault(line.sku_id, line.display_name)
    return [(units[sku], names[sku]) for sku in sorted(units, key=lambda s: (names[s], s))]


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

    def draw_groups(self, batch: Batch, missing: frozenset[str]) -> None:
        for category, groups in _by_category(batch):
            orders = sum(len(g.orders) for g in groups)
            units = sum(o.total_quantity for g in groups for o in g.orders)
            self._need(_BAR_H + _LH * 2)
            self.set_fill_color(225, 225, 225)
            self.set_font("A", "B", 11)
            self.cell(150, _BAR_H, f" {category.code}  {category.name}", fill=True)
            self.set_font("A", "", 9.5)
            self.cell(40, _BAR_H, f"{orders:,} orders · {units:,} units", fill=True,
                      align="R", new_x="LMARGIN", new_y="NEXT")
            for group in groups:
                self._group(group, missing)
            self.ln(1.5)

    def _group(self, group: Group, missing: frozenset[str]) -> None:
        items = sorted(f"{line.display_name}  ×{line.quantity}" for line in group.lines)
        tracking = self._layout == "full"
        self._need(_LH * len(items) + (_TH + 1 if tracking else 0) + 1.2)
        top = self.get_y()
        self.set_font("A", "B", 11.5)
        self.cell(12, _LH, f"{group.number:02d}")
        self.set_font("A", "", 10.5)
        for i, text in enumerate(items):
            self.set_xy(_X_ITEMS, top + _LH * i)
            self.cell(158, _LH, text)
        self.set_xy(180, top)
        self.set_font("A", "B", 11.5)
        self.cell(20, _LH, f"{len(group.orders):,}", align="R")
        self.set_xy(10, top + _LH * len(items))
        if tracking:
            self._tracking(group, items, missing)
        line_y = self.get_y() + 0.6
        self.set_draw_color(170, 170, 170)
        self.line(10, line_y, 200, line_y)
        self.set_xy(10, line_y + 0.6)

    def _tracking(self, group: Group, items: list[str], missing: frozenset[str]) -> None:
        ids = [
            f"{o.tracking_id} (no label)" if o.order_id in missing else o.tracking_id
            for o in group.orders
        ]
        # a "(no label)" ID is wider than its 6-column cell and may reach into the next one;
        # accepted, missing labels are rare
        continued = f"{group.number:02d} (continued)  " + "  /  ".join(items)
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

    def draw_pick_summary(self, batch: Batch) -> None:
        entries = _pick_entries(batch)
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
