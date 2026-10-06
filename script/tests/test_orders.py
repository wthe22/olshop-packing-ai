import csv
from datetime import datetime

import pytest

from conftest import SAMPLES, TESTDATA, needs_samples
from packing.orders import CsvError, Line, REQUIRED_COLUMNS, display_name, read_orders, signature

ORDERS1 = TESTDATA / "orders-1.csv"
ORDERS2 = TESTDATA / "orders-2.csv"


def _write_csv(path, header, rows):
    with open(path, "w", encoding="utf-8-sig", newline="") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(header)
        writer.writerows(rows)


def _row(**overrides):
    values = {
        "Order ID": "580000000000000001",
        "Order Status": "Perlu dikirim",
        "Order Substatus": "Menunggu pengambilan",
        "Tracking ID": "JY0000000101",
        "RTS Time": "06/10/2026 08:00:00",
        "Shipping Provider Name": "J&T Express",
        "Purchase Channel": "TikTok",
        "SKU ID": "1730000000000000001",
        "Quantity": "1",
        "Product Name": "Sepatu | x",
        "Variation": "Default",
        "Seller SKU": "S1",
        "Product Category": "Aksesoris Sepeda Motor",
    }
    values.update(overrides)
    return [values[name] for name in REQUIRED_COLUMNS]


def test_export1_counts():
    result = read_orders(ORDERS1)
    assert result.kept_rows == 16
    assert result.ignored_rows == 1
    assert len(result.orders) == 14


def test_export2_counts():
    result = read_orders(ORDERS2)
    assert result.kept_rows == 24
    assert result.ignored_rows == 1
    assert len(result.orders) == 21


def test_orders_keep_first_seen_order():
    result = read_orders(ORDERS1)
    assert list(result.orders)[0] == "580000000000000001"
    assert list(result.orders)[-1] == "580000000000000014"


def test_multi_line_order():
    order = read_orders(ORDERS1).orders["580000000000000006"]
    assert [line.sku_id for line in order.lines] == [
        "1730000000000000002",
        "1730000000000000003",
    ]
    assert order.distinct_items == 2
    assert order.total_quantity == 2


def test_reordered_rows_give_the_same_signature():
    orders = read_orders(ORDERS1).orders
    six, seven = orders["580000000000000006"], orders["580000000000000007"]
    # rows are in a different order ...
    assert [line.sku_id for line in six.lines] == [
        "1730000000000000002",
        "1730000000000000003",
    ]
    assert [line.sku_id for line in seven.lines] == [
        "1730000000000000003",
        "1730000000000000002",
    ]
    # ... but the packing list is identical
    assert six.signature == seven.signature == "1730000000000000002×1+1730000000000000003×1"


def test_display_name_cuts_at_the_pipe():
    line = read_orders(ORDERS1).orders["580000000000000004"].lines[0]
    assert line.name == "Spion Beat | Kaca Spion Motor Honda Beat"
    assert line.display_name == "Spion Beat — Standard"
    assert line.variation == "Standard"


def test_default_variation_is_empty_and_left_out():
    line = read_orders(ORDERS1).orders["580000000000000008"].lines[0]
    assert line.variation == ""
    assert line.display_name == "Knalpot Racing Beat Full System"


def test_variation_case_is_kept_in_the_display_name():
    orders = read_orders(ORDERS1).orders
    assert orders["580000000000000001"].lines[0].display_name == "Sepatu Standar Samping Motor — honda"
    assert orders["580000000000000002"].lines[0].display_name == "Sepatu Standar Samping Motor — HONDA"


def test_display_name_and_signature_helpers():
    assert display_name("Spion Beat | extra", "Default") == "Spion Beat"
    assert display_name("  A | B ", "  ") == "A"
    first = Line("1730000000000000001", 1, "n", "", "", "", "n")
    second = Line("1730000000000000002", 2, "n", "", "", "", "n")
    assert signature([second, first]) == "1730000000000000001×1+1730000000000000002×2"


def test_rts_time_is_parsed_and_first_row_wins():
    orders = read_orders(ORDERS1).orders
    assert orders["580000000000000001"].rts_time == datetime(2026, 10, 6, 8, 0, 0)
    assert orders["580000000000000001"].courier == "J&T Express"
    assert orders["580000000000000005"].channel == "Tokopedia"


def test_missing_required_column_is_named(tmp_path):
    path = tmp_path / "missing.csv"
    header = [name for name in REQUIRED_COLUMNS if name != "Tracking ID"]
    _write_csv(path, header, [["x"] * len(header)])
    with pytest.raises(CsvError) as excinfo:
        read_orders(path)
    assert "Tracking ID" in str(excinfo.value)


def test_empty_tracking_id_lists_the_order(tmp_path):
    path = tmp_path / "no-tracking.csv"
    rows = [
        _row(**{"Order ID": "580000000000000001", "Tracking ID": "\t"}),
        _row(**{"Order ID": "580000000000000002", "Tracking ID": "JY0000000102\t"}),
    ]
    _write_csv(path, list(REQUIRED_COLUMNS), rows)
    with pytest.raises(CsvError) as excinfo:
        read_orders(path)
    message = str(excinfo.value)
    assert "580000000000000001" in message
    assert "580000000000000002" not in message
    assert "shipment" in message


def test_empty_sku_id_lists_the_order(tmp_path):
    path = tmp_path / "no-sku.csv"
    rows = [_row(**{"SKU ID": "\t"})]
    _write_csv(path, list(REQUIRED_COLUMNS), rows)
    with pytest.raises(CsvError) as excinfo:
        read_orders(path)
    assert "580000000000000001" in str(excinfo.value)


def test_ignored_rows_are_not_orders(tmp_path):
    path = tmp_path / "other-status.csv"
    rows = [
        _row(),
        _row(**{"Order ID": "580000000000000099", "Order Status": "Selesai", "Order Substatus": "Selesai"}),
    ]
    _write_csv(path, list(REQUIRED_COLUMNS), rows)
    result = read_orders(path)
    assert result.kept_rows == 1
    assert result.ignored_rows == 1
    assert list(result.orders) == ["580000000000000001"]


@needs_samples
def test_reads_a_real_sample_without_printing_it(capsys):
    result = read_orders(SAMPLES / "orders-06-1.csv")
    assert len(result.orders) > 0
    captured = capsys.readouterr()
    assert captured.out == ""
    assert captured.err == ""
