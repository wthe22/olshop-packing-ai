"""labels.py: Order ID per page, page copying, group file names (test PDFs made here)."""
from __future__ import annotations

from pathlib import Path

import pytest
from fpdf import FPDF
from fpdf.enums import XPos, YPos
from pypdf import PdfReader

from packing import labels
from packing.labels import LabelPage, LabelSet, LabelError, find_order_id, group_file_name
from packing.orders import Line


def make_pdf(path: Path, pages: list[list[str]]) -> Path:
    """Write one A6 (105 x 148 mm) label PDF; each inner list is one page's text lines."""
    pdf = FPDF(orientation="P", unit="mm", format=(105, 148))
    pdf.set_auto_page_break(False)
    for rows in pages:
        pdf.add_page()
        pdf.set_font("Helvetica", size=10)
        pdf.set_xy(5, 5)
        for row in rows:
            pdf.cell(0, 6, row, new_x=XPos.LMARGIN, new_y=YPos.NEXT)
    pdf.output(path)
    return path


def text_of(path: Path, index: int = 0) -> str:
    return PdfReader(path).pages[index].extract_text()


def make_line(display_name: str, quantity: int = 1) -> Line:
    return Line(
        sku_id="1730000000000000001",
        quantity=quantity,
        name=display_name,
        variation="",
        seller_sku="",
        product_category="",
        display_name=display_name,
    )


# --- find_order_id -------------------------------------------------------------------------

def test_jt_order_id_is_found_before_whitespace_is_removed():
    # Removing whitespace first would glue tracking ID and Order ID into one number.
    assert find_order_id("JY0000001234\n580000000000000001") == "580000000000000001"


def test_lookbehind_and_lookahead_reject_longer_numbers():
    assert find_order_id("Package ID: 1150000000000000001") is None  # 19 digits
    assert find_order_id("900580000000000000012") is None  # 18 digits preceded by a digit
    assert find_order_id("5800000000000000012") is None  # 18 digits followed by a digit
    assert find_order_id("Total: 12345") is None


def test_finds_order_id_on_jt_page(tmp_path):
    path = make_pdf(
        tmp_path / "jt.pdf",
        [["JY0000001234", "580000000000000001", "Telp: 0812****5678", "Package ID: 1150000000000000001"]],
    )
    text = text_of(path)
    assert find_order_id(text) == "580000000000000001"
    assert "1150000000000000001" in text  # present, yet not taken for an Order ID
    assert LabelSet.open([path]).pages == [LabelPage(file=path, index=0, order_id="580000000000000001")]


def test_spaced_characters_page_uses_the_second_search(tmp_path):
    order_id = "580000000000000002"
    path = make_pdf(tmp_path / "idx.pdf", [["O r d e r I d : " + " ".join(order_id)]])
    text = text_of(path)
    assert labels._ORDER_ID.search(text) is None  # the raw search must fail on this page
    assert LabelSet.open([path]).pages[0].order_id == order_id


def test_second_search_accepts_the_fullwidth_colon():
    # IDX / Tokopedia labels; U+FF1A, tested on text because Helvetica cannot draw it.
    assert find_order_id("O r d e r I d ： 5 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 2") == "580000000000000002"


# --- LabelSet.open -------------------------------------------------------------------------

def test_second_page_without_order_id_belongs_to_the_first(tmp_path):
    path = make_pdf(
        tmp_path / "multi.pdf",
        [["JY0000001235", "580000000000000003"], ["Jumlah : 3pcs, Barang : honda"]],
    )
    pages = LabelSet.open([path]).pages
    assert [p.order_id for p in pages] == ["580000000000000003", "580000000000000003"]
    assert [p.index for p in pages] == [0, 1]


def test_consecutive_pages_with_the_same_order_id_are_one_order(tmp_path):
    order_id = "580000000000000004"
    path = make_pdf(tmp_path / "dupe.pdf", [[order_id, "JY0000001236"], [order_id]])
    assert LabelSet.open([path]).by_order()[order_id] == [
        LabelPage(file=path, index=0, order_id=order_id),
        LabelPage(file=path, index=1, order_id=order_id),
    ]


def test_first_page_without_order_id_raises_with_file_and_page(tmp_path):
    path = make_pdf(tmp_path / "noid.pdf", [["JY0000001237", "Jumlah : 1pcs"]])
    with pytest.raises(LabelError) as excinfo:
        LabelSet.open([path])
    message = str(excinfo.value)
    assert "noid.pdf" in message
    assert "page 1" in message


def test_two_files_keep_the_given_order(tmp_path):
    first = make_pdf(tmp_path / "first.pdf", [["580000000000000005"]])
    second = make_pdf(tmp_path / "second.pdf", [["580000000000000006"]])
    label_set = LabelSet.open([first, second])
    assert [p.file for p in label_set.pages] == [first, second]
    assert list(label_set.by_order()) == ["580000000000000005", "580000000000000006"]


def test_missing_file_raises_with_the_name(tmp_path):
    with pytest.raises(LabelError) as excinfo:
        LabelSet.open([tmp_path / "absent.pdf"])
    assert "absent.pdf" in str(excinfo.value)


def test_unreadable_pdf_raises_with_the_name(tmp_path):
    bad = tmp_path / "broken.pdf"
    bad.write_bytes(b"not a pdf at all")
    with pytest.raises(LabelError) as excinfo:
        LabelSet.open([bad])
    assert "broken.pdf" in str(excinfo.value)


# --- LabelSet.by_order / write -------------------------------------------------------------

def test_write_copies_the_right_pages_and_creates_the_folder(tmp_path):
    order_id = "580000000000000007"
    path = make_pdf(
        tmp_path / "set.pdf",
        [[order_id, "JY0000001238"], ["Jumlah : 2pcs"], ["580000000000000008"]],
    )
    label_set = LabelSet.open([path])
    out = tmp_path / "labels" / "07 group.pdf"  # parent does not exist yet
    label_set.write(out, label_set.by_order()[order_id])

    reader = PdfReader(out)
    assert len(reader.pages) == 2
    assert order_id in reader.pages[0].extract_text()
    assert "Jumlah" in reader.pages[1].extract_text()


def test_write_keeps_a_two_file_order_page_order(tmp_path):
    # An order can start in one downloaded file and continue in the next.
    first = make_pdf(tmp_path / "a.pdf", [["580000000000000009"]])
    second = make_pdf(tmp_path / "b.pdf", [["580000000000000009"]])
    label_set = LabelSet.open([first, second])
    out = tmp_path / "out.pdf"
    label_set.write(out, label_set.by_order()["580000000000000009"])
    assert len(PdfReader(out).pages) == 2


# --- group_file_name -----------------------------------------------------------------------

def test_spec_example_sorts_items_by_display_name():
    lines = [
        make_line("Spion Beat — Chrome Standard, honda"),
        make_line("Cover Knalpot Beat — FI 2012-2015"),
    ]
    assert group_file_name(30, 1, lines) == (
        "30 ×1 Cover Knalpot Beat — FI 2012-2015 x1 + Spion Beat — Chrome Standard, honda x1.pdf"
    )


def test_group_number_is_zero_padded_to_at_least_two_digits():
    line = make_line("Spion Beat — Standard, honda", 2)
    assert group_file_name(3, 609, [line]) == "03 ×609 Spion Beat — Standard, honda x2.pdf"
    assert group_file_name(100, 4, [line]) == "100 ×4 Spion Beat — Standard, honda x2.pdf"


def test_illegal_filename_characters_become_dash():
    line = make_line('A\\B/C:D*E?F"G<H>I|J')
    assert group_file_name(2, 1, [line]) == "02 ×1 A-B-C-D-E-F-G-H-I-J x1.pdf"


def test_items_are_cut_at_100_characters_with_ellipsis():
    lines = [make_line(f"Produk Panjang Nomor {i:02d} dengan Nama Cukup Panjang") for i in range(8)]
    name = group_file_name(5, 3, lines)
    assert name.startswith("05 ×3 ")
    items = name[len("05 ×3 ") : -len(".pdf")]
    assert len(items) == 100
    assert items.endswith("…")
