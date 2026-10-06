"""Tests for the three packing-list layouts (1.7)."""
from __future__ import annotations

import re
from datetime import datetime
from pathlib import Path

import pytest
from pypdf import PdfReader

from packing.batches import Batch, Group
from packing.orders import Line, Order
from packing.packing_list import LAYOUTS, PackingListError, write_packing_list
from packing.rules import UNCATEGORISED, Category

FONT = Path(r"C:\Windows\Fonts\arial.ttf")
needs_fonts = pytest.mark.skipif(not FONT.is_file(), reason="Arial font not installed")

DAY = "2026-10-06"
PRINTED = datetime(2026, 10, 6, 16, 40)

CAT_A = Category(code="A", name="Sepatu", when=None)
CAT_B = Category(code="B", name="Spion & Knalpot", when=None)

LONG_NAME = "Spion Scoopy New Gagang Hitam — Dove, honda, Datar Panjang Sekali Lagi"


def _line(sku: str, quantity: int, name: str, variation: str = "", display: str | None = None) -> Line:
    return Line(
        sku_id=sku,
        quantity=quantity,
        name=name,
        variation=variation,
        seller_sku="SELLER-" + sku[-4:],
        product_category=name,
        display_name=display or name,
    )


def _order(n: int, lines: tuple[Line, ...]) -> Order:
    return Order(
        order_id=f"58{n:016d}",
        tracking_id=f"JY{n:010d}",
        rts_time=None,
        courier="J&T",
        channel="Tokopedia",
        lines=lines,
    )


BIG_LINES = (_line("1730000000000000001", 1, "Sepatu Standar Samping Motor"),)
GROUP2_LINES = (
    _line("1730000000000000002", 2, "Sepatu Boots"),
    _line("1730000000000000010", 1, "Spion Beat", "honda"),
)
GROUP3_LINES = (
    _line("1730000000000000003", 1, "Spion Beat", "honda"),
    _line(
        "1730000000000000004",
        1,
        "Spion Scoopy New Gagang Hitam",
        "Dove, honda, Datar",
        display=LONG_NAME,
    ),
)
GROUP4_LINES = (_line("1730000000000000005", 1, "Hook Cantelan universal"),)


def make_batch() -> Batch:
    # 360 orders in one group -> 60 tracking-ID rows, which must continue onto a second page
    big = tuple(_order(n, BIG_LINES) for n in range(1, 361))
    groups = (
        Group(number=1, category=CAT_A, orders=big),
        Group(number=2, category=CAT_A, orders=tuple(_order(n, GROUP2_LINES) for n in range(401, 404))),
        Group(number=3, category=CAT_B, orders=(_order(501, GROUP3_LINES),)),
        Group(number=4, category=UNCATEGORISED, orders=(_order(601, GROUP4_LINES),)),
    )
    return Batch(number=2, groups=groups, removed=())


def missing_of(batch: Batch) -> frozenset[str]:
    return frozenset({batch.groups[0].orders[1].order_id})


def _pages(path: Path) -> tuple[int, str]:
    reader = PdfReader(str(path))
    text = "\n".join(page.extract_text() or "" for page in reader.pages)
    return len(reader.pages), text


def _compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


@needs_fonts
def test_full_layout(tmp_path: Path) -> None:
    batch = make_batch()
    out = tmp_path / "full.pdf"
    written = write_packing_list(
        out, "full", day=DAY, batch=batch, printed=PRINTED, missing_labels=missing_of(batch)
    )
    assert out.is_file()
    pages, text = _pages(out)
    assert written == pages
    assert pages >= 2
    compact = _compact(text)
    assert "Packinglist" in compact and "Batch2" in compact and DAY in compact
    assert re.search(r"page1/\d+", compact)
    assert "Picksummary(unitstotakefromstock)" in compact
    for number in ("01", "02", "03", "04"):
        assert number in text
    assert "JY0000000001" in compact  # first tracking ID
    assert "JY0000000360" in compact  # last tracking ID
    assert re.search(r"\(no\s*label\)", text)
    assert re.search(r"\(continued\)", text)


@needs_fonts
def test_summary_layout(tmp_path: Path) -> None:
    batch = make_batch()
    out = tmp_path / "summary.pdf"
    written = write_packing_list(out, "summary", day=DAY, batch=batch, printed=PRINTED)
    assert out.is_file()
    pages, text = _pages(out)
    assert written == pages
    compact = _compact(text)
    assert "Packinglist" in compact and "Batch2" in compact
    assert re.search(r"page1/\d+", compact)
    assert "JY0000000001" not in compact  # no tracking-ID lines
    assert "Picksummary(unitstotakefromstock)" in compact
    assert not re.search(r"\(continued\)", text)


@needs_fonts
def test_pick_layout_is_one_page(tmp_path: Path) -> None:
    batch = make_batch()
    out = tmp_path / "pick.pdf"
    written = write_packing_list(out, "pick", day=DAY, batch=batch, printed=PRINTED)
    assert out.is_file()
    pages, text = _pages(out)
    assert written == pages
    assert pages == 1
    compact = _compact(text)
    assert "Packinglist" in compact
    assert "Picksummary(unitstotakefromstock)" in compact
    assert "JY0000000001" not in compact
    # the long display name is present (wrapped, not cut)
    assert "PanjangSekaliLagi" in compact


@needs_fonts
def test_page_counts_are_sane(tmp_path: Path) -> None:
    batch = make_batch()
    counts = {}
    for layout in LAYOUTS:
        out = tmp_path / f"{layout}.pdf"
        counts[layout] = write_packing_list(out, layout, day=DAY, batch=batch, printed=PRINTED)
    assert counts["pick"] == 1
    assert counts["full"] > counts["summary"]


def test_unknown_layout_raises_value_error(tmp_path: Path) -> None:
    with pytest.raises(ValueError):
        write_packing_list(tmp_path / "x.pdf", "poster", day=DAY, batch=make_batch(), printed=PRINTED)
    assert not (tmp_path / "x.pdf").exists()


def test_missing_font_names_file_and_folder(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    import packing.packing_list as packing_list

    monkeypatch.setattr(packing_list, "_FONT_DIR", tmp_path / "nofonts")
    with pytest.raises(PackingListError) as excinfo:
        write_packing_list(
            tmp_path / "x.pdf", "pick", day=DAY, batch=make_batch(), printed=PRINTED
        )
    message = str(excinfo.value)
    assert "arial.ttf" in message
    assert str(tmp_path / "nofonts") in message
