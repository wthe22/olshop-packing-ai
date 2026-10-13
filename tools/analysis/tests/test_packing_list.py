"""Tests for the three packing-list layouts over saved PDFs and runs (1.12)."""
from __future__ import annotations

import re
from datetime import datetime
from pathlib import Path

import pytest
from pypdf import PdfReader

from packing.orders import Line, Order, display_name
from packing.packing_list import LAYOUTS, SCOPES, PackingListError, write_packing_list
from packing.picks import Run, SavedPdf
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
        seller_sku="SELLER-" + (sku[-4:] if sku else "0000"),
        product_category=name,
        display_name=display or display_name(name, variation),
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
RUN2_LINES = (
    _line("1730000000000000002", 2, "Sepatu Boots | Kulit"),
    _line("1730000000000000010", 1, "Spion Beat | Kaca Spion", "honda"),
)
RUN3_LINES = (
    _line("1730000000000000003", 1, "Spion Beat | Kaca Spion", "honda"),
    # a slip line: no SKU ID, so the pick item key is (name, variation)
    _line("", 1, "Spion Scoopy New Gagang Hitam", "Dove, honda, Datar", display=LONG_NAME),
)
RUN4_LINES = (_line("", 1, "Hook Cantelan universal"),)


def make_pdfs() -> tuple[SavedPdf, ...]:
    # 1,200 orders in one run -> 200 tracking-ID rows, which continue onto later pages in full
    big = tuple(_order(n, BIG_LINES) for n in range(1, 1201))
    pdf1 = SavedPdf(
        number=1,
        category=CAT_A,
        runs=(
            Run(number=1, orders=big),
            Run(number=2, orders=tuple(_order(n, RUN2_LINES) for n in range(401, 404))),
        ),
    )
    pdf2 = SavedPdf(number=2, category=CAT_B, runs=(Run(number=1, orders=(_order(501, RUN3_LINES),)),))
    pdf3 = SavedPdf(
        number=3,
        category=UNCATEGORISED,
        runs=(Run(number=1, orders=(_order(601, RUN4_LINES),)),),
    )
    return (pdf1, pdf2, pdf3)


def _pages(path: Path) -> tuple[int, str]:
    reader = PdfReader(str(path))
    text = "\n".join(page.extract_text() or "" for page in reader.pages)
    return len(reader.pages), text


def _compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def test_scopes_constant() -> None:
    assert SCOPES == ("per-pdf", "whole")


@needs_fonts
@pytest.mark.parametrize("layout", LAYOUTS)
@pytest.mark.parametrize("scope", SCOPES)
def test_layouts_and_scopes(tmp_path: Path, layout: str, scope: str) -> None:
    all_pdfs = make_pdfs()
    pdfs = [all_pdfs[0]] if scope == "per-pdf" else all_pdfs
    out = tmp_path / f"{scope}-{layout}.pdf"
    written = write_packing_list(out, layout, day=DAY, pdfs=pdfs, printed=PRINTED)
    assert out.is_file()
    pages, text = _pages(out)
    assert written == pages
    compact = _compact(text)
    assert "Packinglist" in compact and DAY in compact
    assert re.search(r"page1/\d+", compact)
    assert "Picksummary(unitstotakefromstock)" in compact
    if layout == "pick":
        assert pages == 1
    if layout == "full":
        assert "JY0000000001" in compact
    else:
        assert "JY0000000001" not in compact


@needs_fonts
def test_full_has_more_pages_than_summary(tmp_path: Path) -> None:
    pdfs = make_pdfs()
    full = write_packing_list(tmp_path / "full.pdf", "full", day=DAY, pdfs=pdfs, printed=PRINTED)
    summary = write_packing_list(
        tmp_path / "summary.pdf", "summary", day=DAY, pdfs=pdfs, printed=PRINTED
    )
    assert full > summary


@needs_fonts
def test_per_pdf_header_names_the_saved_pdf(tmp_path: Path) -> None:
    pdfs = make_pdfs()
    out = tmp_path / "one.pdf"
    write_packing_list(out, "summary", day=DAY, pdfs=[pdfs[0]], printed=PRINTED)
    _, text = _pages(out)
    compact = _compact(text)
    assert "Packinglist·2026-10-06·1ASepatu" in compact
    assert "1,205orders" not in compact  # counts cover only the one saved PDF


@needs_fonts
def test_whole_header_names_the_range_and_every_section(tmp_path: Path) -> None:
    out = tmp_path / "all.pdf"
    write_packing_list(out, "summary", day=DAY, pdfs=make_pdfs(), printed=PRINTED)
    _, text = _pages(out)
    compact = _compact(text)
    assert "Packinglist·2026-10-06·savedPDFs1-3" in compact
    assert "1,205orders" in compact and "1,212units" in compact and "4runs" in compact
    for heading in ("1ASepatu", "2BSpion&Knalpot", "3?Uncategorised"):
        assert heading in compact
    for run in ("1-01", "1-02", "2-01", "3-01"):
        assert run in compact


@needs_fonts
def test_full_tracking_and_continuation(tmp_path: Path) -> None:
    out = tmp_path / "full.pdf"
    written = write_packing_list(out, "full", day=DAY, pdfs=make_pdfs(), printed=PRINTED)
    pages, text = _pages(out)
    assert written == pages and pages >= 2
    compact = _compact(text)
    assert "JY0000000001" in compact  # first tracking ID of the big run
    assert "JY0000001200" in compact  # last tracking ID of the big run
    assert "1-01(continued)" in compact
    assert "nolabel" not in compact  # the "(no label)" marker is gone


@needs_fonts
def test_pick_summary_wraps_a_long_display_name(tmp_path: Path) -> None:
    out = tmp_path / "pick.pdf"
    written = write_packing_list(out, "pick", day=DAY, pdfs=make_pdfs(), printed=PRINTED)
    pages, text = _pages(out)
    assert written == pages == 1
    compact = _compact(text)
    assert "Picksummary(unitstotakefromstock)" in compact
    assert "PanjangSekaliLagi" in compact  # wrapped, not cut
    assert "SpionBeat" in compact and "honda" in compact
    assert "JY0000000001" not in compact


def test_no_item_data_raises(tmp_path: Path) -> None:
    empty = _order(900, ())
    bad = (SavedPdf(number=1, category=CAT_A, runs=(Run(number=1, orders=(empty,)),)),)
    out = tmp_path / "x.pdf"
    with pytest.raises(PackingListError) as excinfo:
        write_packing_list(out, "full", day=DAY, pdfs=bad, printed=PRINTED)
    assert "no item data" in str(excinfo.value)
    assert not out.exists()


def test_unknown_layout_raises_value_error(tmp_path: Path) -> None:
    out = tmp_path / "x.pdf"
    with pytest.raises(ValueError):
        write_packing_list(out, "poster", day=DAY, pdfs=make_pdfs(), printed=PRINTED)
    assert not out.exists()


def test_missing_font_names_file_and_folder(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    import packing.packing_list as packing_list

    monkeypatch.setattr(packing_list, "_FONT_DIR", tmp_path / "nofonts")
    with pytest.raises(PackingListError) as excinfo:
        write_packing_list(
            tmp_path / "x.pdf", "pick", day=DAY, pdfs=make_pdfs(), printed=PRINTED
        )
    message = str(excinfo.value)
    assert "arial.ttf" in message
    assert str(tmp_path / "nofonts") in message
