"""slip.py: the packing-slip table read by text position (small generated pages)."""
from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path

from fpdf import FPDF
from pypdf import PdfReader

from packing.slip import Slip, SlipLine, read_slip
from packing import slip as slip_module

COLUMNS = {"Product Name": 5.8, "SKU": 129.6, "Seller SKU": 172.2, "Qty": 264.6}
HEADER_Y = 290.0
STEP = 8.2
QTY_TOTAL_X = 252.2
QTY_TOTAL_VALUE_X = 288.3
ORDER_ID_X = 208.1
CUSTOMER_MESSAGE_X = 10.5
CUSTOMER_MESSAGE_VALUE_X = 86.6


def _lines(value) -> list[str]:
    if isinstance(value, str):
        return [value] if value else []
    return list(value)


def make_slip(
    path: Path,
    rows: list[dict],
    *,
    qty_total: int | None = None,
    order_id: str = "580000000000000001",
    customer_message: str | None = None,
    columns: dict[str, float] = COLUMNS,
) -> Path:
    """One A6 page with the slip header, the rows, the Order ID and optional message."""
    pdf = FPDF(unit="pt", format=(298, 420))
    pdf.set_auto_page_break(False)
    pdf.set_creation_date(datetime(2026, 1, 1, tzinfo=timezone.utc))
    pdf.add_page()
    pdf.set_font("Helvetica", size=9)
    for word, x in columns.items():
        pdf.text(x, HEADER_Y, word)
    y = HEADER_Y + STEP
    for row in rows:
        name_lines = _lines(row.get("name", ""))
        variation_lines = _lines(row.get("variation", ""))
        seller_lines = _lines(row.get("seller_sku", ""))
        for index, line in enumerate(name_lines):
            pdf.text(columns["Product Name"], y + index * STEP, line)
        for index, line in enumerate(variation_lines):
            pdf.text(columns["SKU"], y + index * STEP, line)
        for index, line in enumerate(seller_lines):
            pdf.text(columns["Seller SKU"], y + index * STEP, line)
        pdf.text(columns["Qty"] + 4.0, y, str(row["qty"]))
        y += STEP * max(len(name_lines), len(variation_lines), len(seller_lines), 1)
    if qty_total is not None:
        y += STEP
        pdf.text(QTY_TOTAL_X, y, "Qty Total:")
        pdf.text(QTY_TOTAL_VALUE_X, y, str(qty_total))
    y += STEP
    pdf.text(ORDER_ID_X, y, f"Order ID: {order_id}")
    if customer_message is not None:
        y += STEP
        pdf.text(CUSTOMER_MESSAGE_X, y, "Customer Message")
        pdf.text(70.0, y, ":")
        pdf.text(CUSTOMER_MESSAGE_VALUE_X, y, customer_message)
    pdf.output(path)
    return path


def slip_of(path: Path) -> Slip | None:
    return read_slip(PdfReader(path).pages[0])


# --- header --------------------------------------------------------------------------------

def test_page_without_a_slip_header_returns_none(tmp_path):
    path = tmp_path / "plain.pdf"
    pdf = FPDF(unit="pt", format=(298, 420))
    pdf.add_page()
    pdf.set_font("Helvetica", size=9)
    pdf.text(5.8, 40, "JY0000001234")
    pdf.text(5.8, 60, "580000000000000001")
    pdf.output(path)
    assert slip_of(path) is None


def test_columns_come_from_the_header_positions_not_fixed_x(tmp_path):
    shifted = {"Product Name": 20.0, "SKU": 140.0, "Seller SKU": 185.0, "Qty": 260.0}
    path = make_slip(
        tmp_path / "shifted.pdf",
        [{"name": "Spion Beat", "variation": "honda", "seller_sku": "SELL-SPI-01", "qty": 2}],
        qty_total=2,
        columns=shifted,
    )
    assert slip_of(path) == Slip(
        lines=(SlipLine(name="Spion Beat", variation="honda", seller_sku="SELL-SPI-01", quantity=2),),
        qty_total=2,
        customer_message="",
    )


# --- rows and cells ------------------------------------------------------------------------

def test_one_row_and_qty_total(tmp_path):
    path = make_slip(
        tmp_path / "one.pdf",
        [{"name": "Sepatu Standar", "variation": "honda", "seller_sku": "SELL-SEP-01", "qty": 1}],
        qty_total=1,
    )
    assert slip_of(path) == Slip(
        lines=(SlipLine("Sepatu Standar", "honda", "SELL-SEP-01", 1),),
        qty_total=1,
        customer_message="",
    )


def test_default_variation_is_stored_empty(tmp_path):
    path = make_slip(
        tmp_path / "default.pdf",
        [{"name": "Knalpot Beat", "variation": "Default", "seller_sku": "", "qty": 3}],
        qty_total=3,
    )
    slip = slip_of(path)
    assert slip.lines == (SlipLine("Knalpot Beat", "", "", 3),)


def test_two_rows_keep_the_top_to_bottom_order(tmp_path):
    path = make_slip(
        tmp_path / "two.pdf",
        [
            {"name": "Spion Beat", "variation": "Standard", "seller_sku": "SELL-SPI-01", "qty": 1},
            {"name": "Knalpot Beat", "variation": "Default", "seller_sku": "", "qty": 2},
        ],
        qty_total=3,
    )
    slip = slip_of(path)
    assert slip.lines == (
        SlipLine("Spion Beat", "Standard", "SELL-SPI-01", 1),
        SlipLine("Knalpot Beat", "", "", 2),
    )
    assert slip.qty_total == 3


def test_a_qty_total_that_does_not_match_is_still_read_raw(tmp_path):
    path = make_slip(
        tmp_path / "mismatch.pdf",
        [{"name": "Spion Beat", "variation": "honda", "seller_sku": "", "qty": 1}],
        qty_total=5,
    )
    assert slip_of(path).qty_total == 5


def test_seller_sku_may_be_absent(tmp_path):
    path = make_slip(
        tmp_path / "nosku.pdf",
        [{"name": "Cover Body", "variation": "Default", "seller_sku": "", "qty": 1}],
        qty_total=1,
    )
    assert slip_of(path).lines[0].seller_sku == ""


# --- wraps ---------------------------------------------------------------------------------

def test_wrapped_cells_join_with_a_space(tmp_path):
    path = make_slip(
        tmp_path / "wrap.pdf",
        [
            {
                "name": ["Sepatu Standar Samping", "Motor | Pelindung Rantai"],
                "variation": ["Standard"],
                "seller_sku": ["SELL-SEP-01"],
                "qty": 1,
            }
        ],
        qty_total=1,
    )
    assert slip_of(path).lines[0].name == "Sepatu Standar Samping Motor | Pelindung Rantai"


def test_a_wrap_right_after_a_dash_joins_without_a_space(tmp_path):
    path = make_slip(
        tmp_path / "dash.pdf",
        [
            {
                "name": ["Cover Knalpot Beat", "FI 2012-", "2015 Full Set"],
                "variation": ["Yamaha-", "Mio"],
                "seller_sku": ["SPION-SCOL-DH-", "CY"],
                "qty": 1,
            }
        ],
        qty_total=1,
    )
    line = slip_of(path).lines[0]
    assert line.name == "Cover Knalpot Beat FI 2012-2015 Full Set"
    assert line.variation == "Yamaha-Mio"
    assert line.seller_sku == "SPION-SCOL-DH-CY"


# --- continuation page ---------------------------------------------------------------------

def test_continuation_page_has_the_header_but_no_rows(tmp_path):
    path = make_slip(
        tmp_path / "cont.pdf",
        [],
        order_id="580000000000000004",
        customer_message="Tolong bubble wrap, jangan dilipat",
    )
    slip = slip_of(path)
    assert slip.lines == ()
    assert slip.qty_total is None
    assert slip.customer_message == "Tolong bubble wrap, jangan dilipat"


def test_customer_message_accepts_the_fullwidth_colon():
    # Core Helvetica cannot draw U+FF1A, so the parser must accept it as text.
    runs = [(10.5, 100.0, "Customer Message"), (78.0, 100.0, "："), (86.6, 100.0, "Halo")]
    assert slip_module._customer_message(runs) == "Halo"


def test_header_only_page_reads_as_an_empty_slip(tmp_path):
    path = make_slip(tmp_path / "empty.pdf", [])
    assert slip_of(path) == Slip(lines=(), qty_total=None, customer_message="")
