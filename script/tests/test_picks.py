"""picks.py: merge, checks, picks, runs, day state, duplicate guard, redo, file names (1.10)."""
from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

import pytest

from conftest import TESTDATA
from packing.labels import LabelOrder, LabelPage, LabelSet
from packing.orders import CsvResult, Line, Order, read_orders
from packing.picks import (
    DayState,
    Duplicate,
    PickError,
    Run,
    SavedEntry,
    SavedPdf,
    build_orders,
    check_fields,
    csv_is_used,
    drop_saved,
    load_state,
    make_picks,
    make_saved_pdf,
    matching_orders,
    redo,
    save_state,
    saved_entries,
    saved_pdf_file_name,
)
from packing.rules import UNCATEGORISED, Category, parse_condition
from packing.slip import SlipLine

SLIP = TESTDATA / "labels-slip.pdf"
PLAIN = TESTDATA / "labels-plain.pdf"
ORDERS2 = TESTDATA / "orders-2.csv"
CATEGORIES = TESTDATA / "categories.toml"


def _slip_orders():
    return LabelSet.open([SLIP]).orders()


def _plain_orders():
    return LabelSet.open([PLAIN]).orders()


def _order(order_id: str, lines: tuple[Line, ...] = (), **kwargs) -> Order:
    values = {
        "tracking_id": "JY" + order_id[-10:],
        "rts_time": None,
        "courier": "J&T Express",
        "channel": "TikTok",
        "lines": lines,
    }
    values.update(kwargs)
    return Order(order_id=order_id, **values)


def _line(name: str, quantity: int = 1, variation: str = "") -> Line:
    from packing.orders import display_name

    return Line(
        sku_id="1730000000000000001",
        quantity=quantity,
        name=name,
        variation=variation,
        seller_sku="",
        product_category="",
        display_name=display_name(name, variation),
    )


def _label_order(
    order_id: str,
    *,
    tracking_id: str = "JY0000000101",
    courier: str = "J&T Express",
    ship_by: datetime | None = None,
    lines: tuple[SlipLine, ...] = (),
    qty_total: int | None = None,
    customer_message: str = "",
) -> LabelOrder:
    return LabelOrder(
        order_id=order_id,
        pages=(LabelPage(file=Path("label.pdf"), index=0, order_id=order_id),),
        tracking_id=tracking_id,
        courier=courier,
        ship_by=ship_by,
        has_slip=bool(lines),
        lines=lines,
        qty_total=qty_total,
        customer_message=customer_message,
    )


# ------------------------------------------------------------------------- build_orders


def test_build_orders_csv_wins_over_label_and_slip():
    orders, _ = build_orders(_slip_orders(), read_orders(ORDERS2))
    assert len(orders) == 23  # the orders come from the labels
    assert list(orders)[0] == "580000000000000001"
    assert list(orders)[-1] == "580000000000000902"

    order = orders["580000000000000002"]  # label has no courier text; CSV fills it
    assert order.courier == "SiCepat REG"
    assert order.channel == "TikTok"
    assert order.lines[0].sku_id == "1730000000000000001"
    assert order.rts_time == datetime(2026, 10, 6, 8, 10, 0)
    assert order.ship_by == datetime(2026, 10, 7, 12, 15)  # always from the label


def test_build_orders_without_csv_uses_slip_and_label_values():
    orders, _ = build_orders(_slip_orders(), None)
    order = orders["580000000000000001"]
    assert order.courier == "J&T Express"  # deduced from the label
    assert order.channel == ""
    assert order.rts_time is None and order.paid_time is None
    assert order.lines[0].sku_id == ""  # slip lines carry no SKU ID
    assert order.lines[0].product_category == ""
    assert order.lines[0].display_name == "Sepatu Standar Samping Motor — honda"

    not_in_csv = orders["580000000000000901"]  # only on a label page
    assert not_in_csv.lines[0].sku_id == ""
    assert not_in_csv.ship_by == datetime(2026, 10, 7, 12, 45)


def test_build_orders_warnings_on_testdata():
    _, warnings = build_orders(_slip_orders(), read_orders(ORDERS2))
    assert any("901" in w and "Qty Total 3" in w and "2" in w for w in warnings)
    assert any("no courier" in w and "580000000000000902" in w for w in warnings)
    assert not any("holds none" in w for w in warnings)


def test_build_orders_without_csv_lists_the_undeduced_couriers():
    _, warnings = build_orders(_slip_orders(), None)
    courier_warning = next(w for w in warnings if "no courier" in w)
    assert "580000000000000902" in courier_warning
    assert "580000000000000002" in courier_warning  # SiCepat label without a CSV


def test_build_orders_csv_holding_none_warns_once_and_uses_labels():
    orders, warnings = build_orders(_plain_orders(), read_orders(ORDERS2))
    hold = next(w for w in warnings if "holds none" in w)
    assert "3 label orders" in hold
    assert orders["580000000000000701"].channel == ""  # the CSV is ignored
    assert any("no item data" in w for w in warnings)


def test_build_orders_no_item_data_warning():
    _, warnings = build_orders(_plain_orders(), None)
    assert any("no item data" in w for w in warnings)


def test_csv_is_used():
    assert csv_is_used(_slip_orders(), read_orders(ORDERS2)) is True
    assert csv_is_used(_plain_orders(), read_orders(ORDERS2)) is False
    assert csv_is_used(_slip_orders(), None) is False


def test_build_orders_warns_on_a_tracking_and_a_line_difference():
    label_orders = {
        "580000000000000001": _label_order(
            "580000000000000001",
            tracking_id="JY0000000101",
            lines=(SlipLine("Spion Beat", "Standard", "S1", 1),),
            qty_total=1,
        )
    }
    csv = CsvResult(
        orders={
            "580000000000000001": _order(
                "580000000000000001",
                lines=(_line("Spion Beat", 2),),
                tracking_id="JY0000000199",
                courier="IDX",
            )
        },
        kept_rows=1,
        ignored_rows=0,
    )
    orders, warnings = build_orders(label_orders, csv)
    assert any("tracking_id differs" in w and "JY0000000199" in w and "JY0000000101" in w for w in warnings)
    assert any("lines differ" in w for w in warnings)
    assert orders["580000000000000001"].tracking_id == "JY0000000199"


def test_build_orders_warns_about_csv_orders_without_a_label_page():
    csv = CsvResult(
        orders={
            "580000000000000001": _order("580000000000000001"),
            "580000000000000099": _order("580000000000000099"),
        },
        kept_rows=2,
        ignored_rows=0,
    )
    label_orders = {"580000000000000001": _label_order("580000000000000001")}
    orders, warnings = build_orders(label_orders, csv)
    assert list(orders) == ["580000000000000001"]  # not added to the orders
    assert any("580000000000000099" in w and "no label page" in w for w in warnings)


def test_build_orders_prefers_the_csv_buyer_message():
    label_orders = {
        "580000000000000004": _label_order(
            "580000000000000004", customer_message="Tolong bubble wrap", lines=(SlipLine("Spion", "", "", 1),)
        )
    }
    csv = CsvResult(
        orders={"580000000000000004": _order("580000000000000004", lines=(_line("Spion"),), buyer_message="CSV text")},
        kept_rows=1,
        ignored_rows=0,
    )
    orders, _ = build_orders(label_orders, csv)
    assert orders["580000000000000004"].buyer_message == "CSV text"  # the CSV wins


# -------------------------------------------------------------------------- check_fields


def test_check_fields_csv_only_field_without_csv():
    cats = [Category(code="B", name="N", when=parse_condition('paid_time < "14:00"'))]
    with pytest.raises(PickError) as excinfo:
        check_fields(cats, has_csv=False, has_items=True)
    assert str(excinfo.value) == 'pick "B" uses field "paid_time", which needs the orders CSV'


def test_check_fields_item_field_without_item_data():
    cats = [Category(code="A", name="N", when=parse_condition('name contains "x"'))]
    with pytest.raises(PickError) as excinfo:
        check_fields(cats, has_csv=True, has_items=False)
    assert str(excinfo.value) == (
        'pick "A" uses field "name", which needs item data (a packing slip or the orders CSV)'
    )


def test_check_fields_ok_and_skips_the_rest_entry():
    cats = [
        Category(code="Z", name="Rest", when=None),
        Category(code="A", name="N", when=parse_condition('courier starts_with "J&T"')),
    ]
    check_fields(cats, has_csv=False, has_items=False)  # no error


# ------------------------------------------------------------------------------ picks


def test_make_picks_matches_expected_picks_json():
    expected = json.loads((TESTDATA / "expected-picks.json").read_text(encoding="utf-8"))
    orders, _ = build_orders(_slip_orders(), read_orders(ORDERS2))
    from packing.rules import load_rules

    pdfs = make_picks(orders, load_rules(CATEGORIES), 1)

    assert [pdf.number for pdf in pdfs] == [entry["number"] for entry in expected]
    assert [pdf.category.code for pdf in pdfs] == [entry["code"] for entry in expected]
    assert [pdf.category.name for pdf in pdfs] == [entry["name"] for entry in expected]
    for pdf, entry in zip(pdfs, expected):
        assert len(pdf.orders) == entry["orders"]
        runs = [[order.order_id for order in run.orders] for run in pdf.runs]
        assert runs == entry["runs"]
    assert pdfs[-1].category is UNCATEGORISED


def test_make_picks_entry_with_none_takes_the_rest_and_skips_zero_entries():
    orders = {
        "580000000000000001": _order("580000000000000001", lines=(_line("Sepatu"),)),
        "580000000000000002": _order("580000000000000002", lines=(_line("Spion"),)),
    }
    cats = [
        Category(code="B", name="Spion", when=parse_condition('name contains "spion"')),
        Category(code="N", name="Never", when=parse_condition('name contains "absent"')),
        Category(code="Z", name="Rest", when=None),
    ]
    pdfs = make_picks(orders, cats, 4)
    assert [pdf.number for pdf in pdfs] == [4, 5]
    assert [pdf.category.code for pdf in pdfs] == ["B", "Z"]  # the zero-order entry is skipped
    assert [order.order_id for order in pdfs[1].orders] == ["580000000000000001"]


def test_make_picks_numbering_continues_from_first_number():
    orders = {"580000000000000001": _order("580000000000000001", lines=(_line("x"),))}
    pdfs = make_picks(orders, [Category(code="Z", name="R", when=None)], 7)
    assert pdfs[0].number == 7


def test_runs_group_by_contents_and_orders_without_lines_are_one_run():
    same = _line("Spion Beat", 1, "honda")
    orders = [
        _order("580000000000000001", lines=(same,)),
        _order("580000000000000002", lines=(same,)),
        _order("580000000000000003", lines=()),
        _order("580000000000000004", lines=()),
    ]
    pdf = make_saved_pdf(1, UNCATEGORISED, orders)
    assert [len(run.orders) for run in pdf.runs] == [2, 2]
    # both runs hold two orders; the empty contents () sorts first
    assert [order.order_id for order in pdf.runs[0].orders] == [
        "580000000000000003",
        "580000000000000004",
    ]
    assert [order.order_id for order in pdf.runs[1].orders] == [
        "580000000000000001",
        "580000000000000002",
    ]


def test_runs_sorted_by_count_desc_then_contents_asc():
    orders = [
        _order("580000000000000001", lines=(_line("Bravo"),)),
        _order("580000000000000002", lines=(_line("Alpha"),)),
        _order("580000000000000003", lines=(_line("Alpha"),)),
    ]
    pdf = make_saved_pdf(1, UNCATEGORISED, orders)
    assert pdf.runs[0].orders[0].lines[0].name == "Alpha"  # two orders wins over one
    assert pdf.runs[1].orders[0].lines[0].name == "Bravo"


def test_matching_orders_none_matches_all():
    orders = [_order("580000000000000001"), _order("580000000000000002")]
    assert matching_orders(orders, None) == orders


# ------------------------------------------------------------------------ day state


def _entry(order_id: str, invocation: int, pdf: int, run: int = 1) -> SavedEntry:
    return SavedEntry(
        order_id=order_id,
        tracking_id="JY0000000101",
        invocation=invocation,
        pdf=pdf,
        run=run,
        save_time="2026-10-07T07:40:12+07:00",
    )


def test_load_state_missing_file_is_empty(tmp_path):
    state = load_state(tmp_path / "2026-10-07")
    assert state.day == "2026-10-07"
    assert state.saved == []
    assert state.next_pdf_number == 1
    assert state.next_invocation == 1


def test_save_and_load_state_round_trip(tmp_path):
    day_dir = tmp_path / "2026-10-07"
    state = DayState(day="2026-10-07", saved=[_entry("580000000000000001", 1, 1, 5)])
    save_state(day_dir, state)

    data = json.loads((day_dir / "state.json").read_text(encoding="utf-8"))
    assert data["day"] == "2026-10-07"
    assert data["saved"][0] == {
        "order_id": "580000000000000001",
        "tracking_id": "JY0000000101",
        "invocation": 1,
        "pdf": 1,
        "run": 5,
        "save_time": "2026-10-07T07:40:12+07:00",
    }
    assert load_state(day_dir) == state


def test_next_numbers_continue_across_invocations():
    state = DayState(day="2026-10-07", saved=[_entry("a", 1, 1), _entry("b", 1, 2), _entry("c", 2, 3)])
    assert state.next_pdf_number == 4
    assert state.next_invocation == 3


def test_drop_saved_returns_duplicates_with_first_save_time():
    orders = {
        "580000000000000001": _order("580000000000000001"),
        "580000000000000002": _order("580000000000000002"),
    }
    state = DayState(day="2026-10-07", saved=[_entry("580000000000000001", 1, 1)])
    kept, duplicates = drop_saved(orders, state)
    assert list(kept) == ["580000000000000002"]
    assert duplicates == [
        Duplicate("580000000000000001", "JY0000000101", "2026-10-07T07:40:12+07:00")
    ]


def test_drop_saved_keeps_the_first_save_time_of_an_order_saved_twice():
    state = DayState(
        day="2026-10-07",
        saved=[
            _entry("580000000000000001", 1, 1),
            SavedEntry("580000000000000001", "JY0000000101", 2, 3, 1, "2026-10-07T09:00:00+07:00"),
        ],
    )
    _, duplicates = drop_saved({"580000000000000001": _order("580000000000000001")}, state)
    assert duplicates[0].save_time == "2026-10-07T07:40:12+07:00"


def test_redo_forgets_the_last_invocation_and_names_the_numbers():
    state = DayState(
        day="2026-10-07",
        saved=[_entry("a", 1, 1), _entry("b", 1, 2), _entry("c", 2, 3)],
    )
    kept, numbers = redo(state)
    assert numbers == [3]
    assert [entry.order_id for entry in kept.saved] == ["a", "b"]
    assert kept.next_pdf_number == 3


def test_redo_with_no_state_is_a_no_op():
    state = DayState(day="2026-10-07")
    kept, numbers = redo(state)
    assert numbers == []
    assert kept is state


def test_saved_entries_record_pdf_and_run_numbers():
    order = _order("580000000000000001")
    pdf = SavedPdf(number=3, category=UNCATEGORISED, runs=(Run(5, (order,)),))
    entries = saved_entries([pdf], invocation=2, save_time="2026-10-07T07:40:12+07:00")
    assert entries == [SavedEntry("580000000000000001", order.tracking_id, 2, 3, 5, "2026-10-07T07:40:12+07:00")]


# ------------------------------------------------------------------- file names


def _pdf_with(number: int, code: str, name: str, orders: int) -> SavedPdf:
    category = Category(code=code, name=name, when=None)
    run = Run(1, tuple(_order(f"58{i:016d}") for i in range(orders)))
    return SavedPdf(number=number, category=category, runs=(run,))


def test_saved_pdf_file_name_spec_example():
    assert saved_pdf_file_name(_pdf_with(3, "Z", "Lainnya", 2)) == "3 Z Lainnya ×2.pdf"


def test_saved_pdf_file_name_uncategorised_code_becomes_dash():
    assert saved_pdf_file_name(_pdf_with(5, "?", "Uncategorised", 2)) == "5 - Uncategorised ×2.pdf"


def test_saved_pdf_file_name_illegal_characters_become_dash():
    name = 'A\\B/C:D*E?F"G<H>I|J'
    assert saved_pdf_file_name(_pdf_with(1, "A", name, 1)) == "1 A A-B-C-D-E-F-G-H-I-J ×1.pdf"


def test_saved_pdf_file_name_cuts_the_name_at_100_characters():
    long_name = "Produk " * 40
    filename = saved_pdf_file_name(_pdf_with(1, "A", long_name, 3))
    assert filename.startswith("1 A ")
    assert filename.endswith(" ×3.pdf")
    name_part = filename[len("1 A ") : -len(" ×3.pdf")]
    assert len(name_part) == 100
    assert name_part.endswith("…")
