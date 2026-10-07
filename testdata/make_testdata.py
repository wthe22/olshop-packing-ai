"""Write testdata/orders-1.csv and orders-2.csv (made-up data, deterministic).

Run:  python testdata/make_testdata.py [output_dir]

The two files are two exports of one day (06/10/2026), so export 2 holds the still-open
orders of export 1 plus the orders that are new in export 2. Real values are never used.
Only the orders CSVs are written; categories.toml and expected.json are hand-written.
"""
from __future__ import annotations

import csv
import sys

from datetime import datetime, timezone
from pathlib import Path

from fpdf import FPDF

HEADER = [
    "Order ID", "Order Status", "Order Substatus", "Cancelation/Return Type",
    "Normal or Pre-order", "SKU ID", "Seller SKU", "Product Name", "Variation", "Quantity",
    "Sku Quantity of return", "SKU Unit Original Price", "SKU Subtotal Before Discount",
    "SKU Platform Discount", "SKU Seller Discount", "SKU Subtotal After Discount",
    "Shipping Fee After Discount", "Original Shipping Fee", "Shipping Fee Seller Discount",
    "Shipping Fee Platform Discount", "Distance Shipping Fee", "Distance Fee",
    "Order Refund Amount", "Payment platform discount", "Buyer Service Fee", "Handling Fee",
    "Shipping Insurance", "Item Insurance", "Order Amount", "Created Time", "Paid Time",
    "RTS Time", "Shipped Time", "Delivered Time", "Cancelled Time", "Cancel By",
    "Cancel Reason", "Fulfillment Type", "Warehouse Name", "Tracking ID", "Delivery Option",
    "Shipping Provider Name", "Buyer Message", "Buyer Username", "Recipient", "Phone #",
    "Zipcode", "Country", "Province", "Regency and City", "Districts", "Villages",
    "Detail Address", "Additional address information", "Payment Method", "Weight(kg)",
    "Product Category", "Package ID", "Purchase Channel", "Seller Note", "Checked Status",
    "Checked Marked by", "Tokopedia Invoice Number", "Order Channel", "Creator Handle",
]

# Columns whose values end with a tab; an empty one is a lone tab (business-process 02).
TRAILING_TAB = {
    "Order ID", "SKU ID", "Package ID", "Zipcode",
    "Created Time", "Paid Time", "RTS Time", "Shipped Time", "Delivered Time", "Cancelled Time",
}

PACKING = ("Perlu dikirim", "Menunggu pengambilan")
OTHER = ("Selesai", "Selesai")

CREATED = "06/10/2026 07:00:00"
PAID = "06/10/2026 07:00:05"
UNIT_PRICE = 50000

# made-up catalogue: sku id -> (Product Name, Seller SKU, Product Category)
SEPATU = "1730000000000000001"
SPION = "1730000000000000002"
KNALPOT = "1730000000000000003"
COVER = "1730000000000000004"
JOK = "1730000000000000005"
FOOTSTEP = "1730000000000000006"

PRODUCTS = {
    SEPATU: ("Sepatu Standar Samping Motor | Pelindung Rantai", "SELL-SEP-01", "Aksesoris Sepeda Motor"),
    SPION: ("Spion Beat | Kaca Spion Motor Honda Beat", "SELL-SPI-01", "Cermin & Aksesori"),
    KNALPOT: ("Knalpot Racing Beat Full System", "", "Aksesoris Sepeda Motor"),
    COVER: ("Cover Body Mio | Plastik Body Motor", "", "Body & Cover"),
    JOK: ("Jok Kulit Vario | Custom", "SELL-JOK-01", "Jok & Aksesori"),
    FOOTSTEP: ("Footstep Motor | Aluminium", "", "Aksesoris Sepeda Motor"),
}

# One spec = (suffix, tracking, courier, channel, rts, [(sku, qty, variation), ...],
#             optional buyer message). Order ID = "580000000000000" + suffix.
EXPORT1 = [
    ("001", "JY0000000101", "J&T Express", "TikTok", "06/10/2026 08:00:00", [(SEPATU, 1, "honda")]),
    ("002", "000000000202", "SiCepat REG", "TikTok", "06/10/2026 08:10:00", [(SEPATU, 1, "HONDA")]),
    ("003", "JY0000000103", "J&T Express", "TikTok", "06/10/2026 08:20:00", [(SEPATU, 2, "honda")]),
    ("004", "JY0000000104", "J&T Express", "TikTok", "06/10/2026 08:30:00",
     [(SPION, 1, "Standard")], "Tolong bubble wrap, jangan dilipat"),
    # order 004 and 005 share the group S2x1 and have the same RTS Time (Order ID tie-break)
    ("005", "000000000205", "SiCepat REG", "Tokopedia", "06/10/2026 08:30:00", [(SPION, 1, "Standard")]),
    ("006", "TKP0000000106", "IDX", "TikTok", "06/10/2026 08:40:00",
     [(SPION, 1, "Standard"), (KNALPOT, 1, "Default")], "Kirim cepat ya, terima kasih"),
    # order 007 is 006 with the two rows in the other line order (same signature)
    ("007", "JY0000000107", "J&T Express", "TikTok", "06/10/2026 08:45:00",
     [(KNALPOT, 1, "Default"), (SPION, 1, "HONDA")]),
    ("008", "000000000208", "SiCepat REG", "TikTok", "06/10/2026 08:50:00", [(KNALPOT, 1, "Default")]),
    ("009", "000000000209", "SiCepat REG", "TikTok", "06/10/2026 09:00:00", [(COVER, 3, "Default")]),
    ("010", "000000000210", "SiCepat REG", "TikTok", "06/10/2026 09:05:00", [(JOK, 3, "Default")]),
    ("011", "000000000211", "SiCepat REG", "TikTok", "06/10/2026 09:10:00", [(COVER, 3, "Default")]),
    ("012", "JY0000000112", "J&T Express", "Tokopedia", "06/10/2026 09:20:00", [(COVER, 1, "Default")]),
    ("013", "JY0000000113", "J&T Express", "Tokopedia", "06/10/2026 09:30:00", [(JOK, 1, "Default")]),
    ("014", "000000000214", "SiCepat REG", "TikTok", "06/10/2026 09:40:00", [(FOOTSTEP, 1, "Default")]),
]

EXPORT2_NEW = [
    ("015", "JY0000000115", "J&T Express", "TikTok", "06/10/2026 10:00:00", [(SEPATU, 1, "honda")]),
    ("016", "000000000216", "SiCepat REG", "Tokopedia", "06/10/2026 10:05:00", [(SEPATU, 1, "honda")]),
    ("017", "JY0000000117", "J&T Express", "TikTok", "06/10/2026 10:10:00", [(SEPATU, 2, "honda")]),
    # order 018 and 019 share the group S2x1 and have the same RTS Time (Order ID tie-break)
    ("018", "JY0000000118", "J&T Express", "TikTok", "06/10/2026 10:20:00", [(SPION, 1, "Standard")]),
    ("019", "000000000219", "SiCepat REG", "TikTok", "06/10/2026 10:20:00", [(SPION, 1, "Standard")]),
    ("020", "TKP0000000120", "IDX", "TikTok", "06/10/2026 10:30:00",
     [(SPION, 1, "Standard"), (KNALPOT, 1, "Default")]),
    ("021", "000000000221", "SiCepat REG", "Tokopedia", "06/10/2026 10:40:00", [(JOK, 3, "Default")]),
    ("022", "000000000222", "SiCepat REG", "TikTok", "06/10/2026 10:50:00", [(FOOTSTEP, 1, "Default")]),
]

# A row of another status: ignored, counted, never an order.
IGNORED1 = ("090", "JY0000000190", "J&T Express", "TikTok", "", [(COVER, 1, "Default")])
IGNORED2 = ("091", "JY0000000191", "J&T Express", "TikTok", "", [(JOK, 1, "Default")])


def _package_id(order_id: str) -> str:
    return "1000000000000000" + order_id[-3:]


def _row(spec, sku, qty, variation, *, ignored, buyer_message) -> list[str]:
    suffix, tracking, courier, channel, rts = spec[:5]
    order_id = "580000000000000" + suffix  # 18 digits, starting with 5
    seq = int(suffix)
    status, substatus = OTHER if ignored else PACKING
    name, seller_sku, product_category = PRODUCTS[sku]
    values = {column: "" for column in HEADER}
    values.update({
        "Order ID": order_id,
        "Order Status": status,
        "Order Substatus": substatus,
        "Normal or Pre-order": "Normal",
        "SKU ID": sku,
        "Seller SKU": seller_sku,
        "Product Name": name,
        "Variation": variation,
        "Quantity": str(qty),
        "Sku Quantity of return": "0",
        "SKU Unit Original Price": str(UNIT_PRICE),
        "SKU Subtotal Before Discount": str(UNIT_PRICE * qty),
        "SKU Platform Discount": "0",
        "SKU Seller Discount": "0",
        "SKU Subtotal After Discount": str(UNIT_PRICE * qty),
        "Order Amount": str(UNIT_PRICE * qty),
        "Created Time": CREATED,
        "Paid Time": PAID,
        "RTS Time": rts,
        "Fulfillment Type": "Dikirim oleh Penjual",
        "Warehouse Name": "Gudang Utama",
        "Tracking ID": tracking,
        "Delivery Option": "Reguler",
        "Shipping Provider Name": courier,
        "Buyer Message": buyer_message,
        "Buyer Username": f"buyer_{seq:03d}",
        "Recipient": f"Pembeli {seq:03d}",
        "Phone #": f"0812****{seq:02d}",
        "Zipcode": "40123",
        "Country": "Indonesia",
        "Province": "DKI Jakarta",
        "Regency and City": "Jakarta Selatan",
        "Districts": "Kebayoran Baru",
        "Villages": "Melawai",
        "Detail Address": f"Jalan Contoh No. {seq}",
        "Payment Method": "COD",
        "Weight(kg)": "1",
        "Product Category": product_category,
        "Package ID": _package_id(order_id),
        "Purchase Channel": channel,
        "Order Channel": "TikTok Shop",
    })
    for column in TRAILING_TAB:
        values[column] += "\t"
    return [values[column] for column in HEADER]


def _rows(specs, *, ignored=False) -> list[list[str]]:
    rows = []
    for spec in specs:
        lines = spec[5]
        buyer_message = spec[6] if len(spec) > 6 else ""
        for sku, qty, variation in lines:
            rows.append(_row(spec, sku, qty, variation, ignored=ignored, buyer_message=buyer_message))
    return rows


def _write(path: Path, rows: list[list[str]]) -> None:
    with open(path, "w", encoding="utf-8-sig", newline="") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(HEADER)
        writer.writerows(rows)


def main(argv: list[str] | None = None) -> None:
    args = sys.argv[1:] if argv is None else argv
    out_dir = Path(args[0]) if args else Path(__file__).resolve().parent
    out_dir.mkdir(parents=True, exist_ok=True)
    assert len(HEADER) == 65, len(HEADER)

    export1 = _rows(EXPORT1) + _rows([IGNORED1], ignored=True)
    export2 = _rows([s for s in EXPORT1 if s[0] != "008"]) + _rows(EXPORT2_NEW) + _rows([IGNORED2], ignored=True)
    _write(out_dir / "orders-1.csv", export1)
    _write(out_dir / "orders-2.csv", export2)

    slip_docs, plain_docs = label_documents()
    _write_label_pdf(out_dir / "labels-slip.pdf", slip_docs)
    _write_label_pdf(out_dir / "labels-plain.pdf", plain_docs)


# --- label PDFs (task 1.14) ----------------------------------------------------------------
# A6 298 x 420 pt pages, core font Helvetica. The slip columns and rows are placed at the
# positions seen on the real samples; labels-slip.pdf uses the orders of orders-2.csv (export 2)
# plus two made-up orders, labels-plain.pdf holds three plain labels with other Order IDs.

PAGE_WIDTH = 298.0
PAGE_HEIGHT = 420.0
SLIP_HEADER_Y = 290.0
SLIP_LINE_STEP = 8.2  # a wrapped line sits this much lower than the line above
COLUMN_X = {"Product Name": 5.8, "SKU": 129.6, "Seller SKU": 172.2, "Qty": 264.6}
COLUMN_WRAP = {"Product Name": 25, "SKU": 9, "Seller SKU": 16}
QTY_X = 268.6
QTY_TOTAL_X = 252.2
QTY_TOTAL_VALUE_X = 288.3
ORDER_ID_X = 208.1
CUSTOMER_MESSAGE_X = 10.5
CUSTOMER_MESSAGE_COLON_X = 70.0
CUSTOMER_MESSAGE_VALUE_X = 86.6
PDF_CREATION_DATE = datetime(2026, 1, 1, tzinfo=timezone.utc)  # fixed, so the bytes are stable

_DEDUCED_COURIER = {"J&T Express": "J&T Express", "SiCepat REG": "", "IDX": "IDX"}


def _label_order_id(suffix: str) -> str:
    return "580000000000000" + suffix


def _ship_by(suffix: str) -> str | None:
    if suffix == "902":
        return None
    return f"07/10/2026 {10 + int(suffix) % 6:02d}:15"


def _ship_by_iso(value: str | None) -> str | None:
    if value is None:
        return None
    return datetime.strptime(value, "%d/%m/%Y %H:%M").isoformat()


def _wrap(value: str, limit: int) -> list[str]:
    """Greedy wrap on spaces so that joining the lines with a space rebuilds the value."""
    if not value:
        return []
    lines: list[str] = []
    current = ""
    for word in value.split(" "):
        candidate = f"{current} {word}" if current else word
        if not current or len(candidate) <= limit:
            current = candidate
        else:
            lines.append(current)
            current = word
    lines.append(current)
    return lines


def label_documents() -> tuple[list[dict], list[dict]]:
    """One document per order for labels-slip.pdf and labels-plain.pdf."""
    slip = []
    for spec in [s for s in EXPORT1 if s[0] != "008"] + EXPORT2_NEW:
        suffix, tracking, courier = spec[0], spec[1], spec[2]
        message = spec[6] if len(spec) > 6 else ""
        rows = [
            {"name": PRODUCTS[sku][0], "variation": variation,
             "seller_sku": PRODUCTS[sku][1], "qty": qty}
            for sku, qty, variation in spec[5]
        ]
        slip.append({
            "order_id": _label_order_id(suffix),
            "tracking": tracking,
            "courier": _DEDUCED_COURIER[courier],
            "courier_text": courier == "J&T Express",
            "ship_by": _ship_by(suffix),
            "spaced": False,
            "rows": rows,
            "qty_total": sum(row["qty"] for row in rows),
            "has_slip": True,
            "continuation_message": message if suffix == "004" else None,
        })
    # Not in the CSV: a J&T slip whose cells wrap right after '-' and whose Qty Total is wrong.
    slip.append({
        "order_id": _label_order_id("901"),
        "tracking": "JY0000000901",
        "courier": "J&T Express",
        "courier_text": True,
        "ship_by": "07/10/2026 12:45",
        "spaced": False,
        "rows": [
            {
                "name": "Cover Knalpot Beat FI 2012-2015 Full Set",
                "name_lines": ["Cover Knalpot Beat", "FI 2012-", "2015 Full Set"],
                "variation": "Yamaha-Mio",
                "variation_lines": ["Yamaha-", "Mio"],
                "seller_sku": "SPION-SCOL-DH-CY",
                "seller_sku_lines": ["SPION-SCOL-DH-", "CY"],
                "qty": 1,
            },
            {"name": "Spion Beat Samping", "variation": "Default", "seller_sku": "", "qty": 1},
        ],
        "qty_total": 3,
        "has_slip": True,
        "continuation_message": None,
    })
    # Not in the CSV: a SiCepat slip, 12-digit tracking, no courier text -> courier unknown.
    slip.append({
        "order_id": _label_order_id("902"),
        "tracking": "000000000902",
        "courier": "",
        "courier_text": False,
        "ship_by": None,
        "spaced": False,
        "rows": [{"name": "Sepatu Standar", "variation": "honda",
                  "seller_sku": "SELL-SEP-01", "qty": 2}],
        "qty_total": 2,
        "has_slip": True,
        "continuation_message": None,
    })

    plain = [
        {"order_id": _label_order_id("701"), "tracking": "JY0000000701",
         "courier": "J&T Express", "courier_text": True, "ship_by": "07/10/2026 09:00",
         "spaced": False, "rows": [], "qty_total": None, "has_slip": False,
         "continuation_message": None},
        {"order_id": _label_order_id("702"), "tracking": "TKP0000000702",
         "courier": "IDX", "courier_text": False, "ship_by": "07/10/2026 09:30",
         "spaced": True, "rows": [], "qty_total": None, "has_slip": False,
         "continuation_message": None},
        {"order_id": _label_order_id("703"), "tracking": "000000000703",
         "courier": "", "courier_text": False, "ship_by": None,
         "spaced": False, "rows": [], "qty_total": None, "has_slip": False,
         "continuation_message": None},
    ]
    return slip, plain


def label_expectations() -> dict[str, dict]:
    """The hand-checked truth for testdata/expected-labels.json, taken from the documents."""
    slip_docs, plain_docs = label_documents()
    expected: dict[str, dict] = {}
    for doc in slip_docs:
        pages = 1 + (1 if doc["continuation_message"] is not None else 0)
        expected[doc["order_id"]] = {
            "pages": pages,
            "tracking_id": doc["tracking"],
            "courier": doc["courier"],
            "ship_by": _ship_by_iso(doc["ship_by"]),
            "has_slip": True,
            "lines": [
                [row["name"], "" if row["variation"] == "Default" else row["variation"],
                 row["seller_sku"], row["qty"]]
                for row in doc["rows"]
            ],
            "qty_total": doc["qty_total"],
            "customer_message": doc["continuation_message"] or "",
        }
    for doc in plain_docs:
        expected[doc["order_id"]] = {
            "pages": 1,
            "tracking_id": doc["tracking"],
            "courier": doc["courier"],
            "ship_by": _ship_by_iso(doc["ship_by"]),
            "has_slip": False,
            "lines": [],
            "qty_total": None,
            "customer_message": "",
        }
    return expected


def _draw_label_page(pdf: FPDF, doc: dict) -> None:
    pdf.set_font("Helvetica", size=9)
    spaced = doc["spaced"]

    def put(x: float, y: float, value: str) -> None:
        pdf.text(x, y, " ".join(value) if spaced else value)

    put(5.8, 20, "Shipping Label")
    put(5.8, 36, f"Order Id: {doc['order_id']}" if spaced else doc["order_id"])
    if doc["ship_by"]:
        put(5.8, 52, f"In transit by: {doc['ship_by']}")
    for index in range(5):
        put(5.8, 66 + index * 12, doc["tracking"])
    if doc["courier_text"]:
        put(5.8, 130, "www.jet.co.id")


def _draw_slip_header(pdf: FPDF) -> None:
    pdf.set_font("Helvetica", size=9)
    for word, x in COLUMN_X.items():
        pdf.text(x, SLIP_HEADER_Y, word)


def _draw_slip_table(pdf: FPDF, doc: dict) -> None:
    _draw_slip_header(pdf)
    y = SLIP_HEADER_Y + SLIP_LINE_STEP
    for row in doc["rows"]:
        name_lines = row.get("name_lines") or _wrap(row["name"], COLUMN_WRAP["Product Name"])
        variation_lines = row.get("variation_lines") or _wrap(row["variation"], COLUMN_WRAP["SKU"])
        if row.get("seller_sku_lines") is not None:
            seller_lines = row["seller_sku_lines"]
        else:
            seller_lines = _wrap(row["seller_sku"], COLUMN_WRAP["Seller SKU"])
        for index, line in enumerate(name_lines):
            pdf.text(COLUMN_X["Product Name"], y + index * SLIP_LINE_STEP, line)
        for index, line in enumerate(variation_lines):
            pdf.text(COLUMN_X["SKU"], y + index * SLIP_LINE_STEP, line)
        for index, line in enumerate(seller_lines):
            pdf.text(COLUMN_X["Seller SKU"], y + index * SLIP_LINE_STEP, line)
        pdf.text(QTY_X, y, str(row["qty"]))
        y += SLIP_LINE_STEP * max(len(name_lines), len(variation_lines), len(seller_lines), 1)
    y += SLIP_LINE_STEP
    pdf.text(QTY_TOTAL_X, y, "Qty Total:")
    pdf.text(QTY_TOTAL_VALUE_X, y, str(doc["qty_total"]))
    y += SLIP_LINE_STEP
    pdf.text(ORDER_ID_X, y, f"Order ID: {doc['order_id']}")


def _draw_continuation(pdf: FPDF, doc: dict) -> None:
    _draw_slip_header(pdf)
    y = SLIP_HEADER_Y + SLIP_LINE_STEP
    pdf.text(ORDER_ID_X, y, f"Order ID: {doc['order_id']}")
    y += SLIP_LINE_STEP
    pdf.text(CUSTOMER_MESSAGE_X, y, "Customer Message")
    pdf.text(CUSTOMER_MESSAGE_COLON_X, y, ":")
    pdf.text(CUSTOMER_MESSAGE_VALUE_X, y, doc["continuation_message"])


def _write_label_pdf(path: Path, docs: list[dict]) -> None:
    pdf = FPDF(unit="pt", format=(PAGE_WIDTH, PAGE_HEIGHT))
    pdf.set_auto_page_break(False)
    pdf.set_creation_date(PDF_CREATION_DATE)
    for doc in docs:
        pdf.add_page()
        _draw_label_page(pdf, doc)
        if doc["has_slip"]:
            _draw_slip_table(pdf, doc)
        if doc["continuation_message"] is not None:
            pdf.add_page()
            _draw_continuation(pdf, doc)
    pdf.output(path)


if __name__ == "__main__":
    main()
