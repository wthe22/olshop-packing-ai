"""Write testdata/orders-1.csv and orders-2.csv (made-up data, deterministic).

Run:  python testdata/make_testdata.py [output_dir]

The two files are two exports of one day (06/10/2026), so export 2 holds the still-open
orders of export 1 plus the orders that are new in export 2. Real values are never used.
Only the orders CSVs are written; categories.toml and expected.json are hand-written.
"""
from __future__ import annotations

import csv
import sys
from pathlib import Path

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
    ("006", "TK00000000106", "IDX", "TikTok", "06/10/2026 08:40:00",
     [(SPION, 1, "Standard"), (KNALPOT, 1, "Default")]),
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
    ("020", "TK00000000120", "IDX", "TikTok", "06/10/2026 10:30:00",
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


if __name__ == "__main__":
    main()
