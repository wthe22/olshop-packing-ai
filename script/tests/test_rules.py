"""Tests for rules.py: condition language and categories.toml (04-rules-file)."""
from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

import pytest

from packing.orders import Line, Order
from packing.rules import (
    CSV_ONLY_FIELDS,
    ITEM_FIELDS,
    UNCATEGORISED,
    RulesError,
    categorize,
    evaluate,
    fields_used,
    format_condition,
    load_rules,
    parse_condition,
)

REPO = Path(__file__).resolve().parents[2]
TESTDATA = REPO / "testdata"

FIXTURE = json.loads((TESTDATA / "conditions.json").read_text(encoding="utf-8"))


def _dt(value: str | None) -> datetime | None:
    return datetime.fromisoformat(value) if value else None


def _order(data: dict) -> Order:
    lines = tuple(
        Line(
            sku_id=line["sku_id"],
            quantity=line["quantity"],
            name=line["name"],
            variation=line["variation"],
            seller_sku=line["seller_sku"],
            product_category=line["product_category"],
            display_name=line["display_name"],
        )
        for line in data["lines"]
    )
    return Order(
        order_id=data["order_id"],
        tracking_id=data["tracking_id"],
        rts_time=_dt(data.get("rts_time")),
        courier=data["courier"],
        channel=data["channel"],
        lines=lines,
        paid_time=_dt(data.get("paid_time")),
        created_time=_dt(data.get("created_time")),
        ship_by=_dt(data.get("ship_by")),
    )


ORDERS = {key: _order(value) for key, value in FIXTURE["orders"].items()}


def _line(name: str, quantity: int = 1, product_category: str = "Sepeda Motor") -> Line:
    return Line(
        sku_id="1730000000000000099",
        quantity=quantity,
        name=name,
        variation="",
        seller_sku="",
        product_category=product_category,
        display_name=name,
    )


# ------------------------------------------------------------------ fixture


CASES = FIXTURE["cases"]


@pytest.mark.parametrize(
    "case",
    CASES,
    ids=[f"{i}:{case['text'][:40]!r}" for i, case in enumerate(CASES)],
)
def test_conditions_fixture(case: dict) -> None:
    app_fields = case.get("app_fields", False)
    if "error" in case:
        with pytest.raises(RulesError) as excinfo:
            parse_condition(case["text"], app_fields=app_fields)
        assert str(excinfo.value) == case["error"]
        return

    cond = parse_condition(case["text"], app_fields=app_fields)
    canonical = format_condition(cond)
    assert canonical == case["canonical"]
    # Round trip: parse(format(parse(t))) == parse(t).
    assert parse_condition(canonical, app_fields=app_fields) == cond
    if "fields" in case:
        assert sorted(fields_used(cond)) == sorted(case["fields"])

    app_values = case.get("app_values")
    for key, expected in case["values"].items():
        assert evaluate(cond, ORDERS[key], app_values) == expected, key


def test_fixture_shape() -> None:
    assert isinstance(FIXTURE["about"], str) and FIXTURE["about"]
    assert len(FIXTURE["orders"]) >= 2
    for case in CASES:
        assert "text" in case
        assert ("error" in case) ^ ("canonical" in case and "values" in case)
        assert set(case["values"]) <= set(FIXTURE["orders"]) if "values" in case else True
        if "fields" in case:
            assert isinstance(case["fields"], list) and case["fields"]


def test_field_constants() -> None:
    assert CSV_ONLY_FIELDS == frozenset(
        {"sku_id", "product_category", "channel", "paid_time", "rts_time", "created_time"}
    )
    assert ITEM_FIELDS == frozenset(
        {"name", "display_name", "variation", "sku_id", "seller_sku", "product_category",
         "line_quantity"}
    )


def test_fields_used_collects_every_field() -> None:
    cond = parse_condition(
        'paid_time < "14:00" and (name contains "x" or not courier starts_with "J&T")'
    )
    assert fields_used(cond) == frozenset({"paid_time", "name", "courier"})
    assert fields_used(parse_condition("total_quantity = 1")) == frozenset({"total_quantity"})
    assert fields_used(parse_condition('not tracking_id starts_with "JY"')) == frozenset(
        {"tracking_id"}
    )


# ------------------------------------------------------------ repo categories


def test_repo_categories_load() -> None:
    cats = load_rules(REPO / "categories.toml")
    assert [c.code for c in cats] == ["A", "B", "Z"]
    assert [c.name for c in cats] == ["Sepatu", "Spion & Knalpot", "Lainnya"]
    assert cats[-1].when is None
    assert all(c.when is not None for c in cats[:-1])


def test_repo_category_conditions_round_trip() -> None:
    cats = load_rules(REPO / "categories.toml")
    by_code = {c.code: c for c in cats}
    assert format_condition(by_code["B"].when) == (
        'not name contains "sepatu" and (name contains "spion" or name contains "knalpot")'
    )
    for cat in cats:
        if cat.when is not None:
            text = format_condition(cat.when)
            assert format_condition(parse_condition(text)) == text


def test_categorize_first_match_and_uncategorised() -> None:
    cats = load_rules(REPO / "categories.toml")
    assert categorize(ORDERS["o2"], cats).code == "A"  # sepatu
    assert categorize(ORDERS["o1"], cats).code == "B"  # spion
    assert categorize(ORDERS["o3"], cats).code == "B"

    other = Order(
        "580000000000000010", "JY0000000010", None, "JNE", "TikTok",
        (_line("Busi Iridium", product_category="Sparepart Motor"),),
    )
    assert categorize(other, cats).code == "Z"
    assert categorize(other, cats[:-1]) is UNCATEGORISED


def test_categorize_returns_category_object() -> None:
    cats = load_rules(REPO / "categories.toml")
    assert categorize(ORDERS["o1"], cats) is cats[1]


# ------------------------------------------------------------- validation


def _toml(tmp_path: Path, text: str) -> Path:
    path = tmp_path / "categories.toml"
    path.write_text(text, encoding="utf-8")
    return path


VALID_COND = "when = 'name contains \"x\"'\n"

VALIDATION_CASES = [
    (
        "no categories",
        "# a comment only\n",
        "categories.toml: no categories",
    ),
    (
        "empty category array",
        "category = []\n",
        "categories.toml: no categories",
    ),
    (
        "unknown top-level key",
        'title = "x"\n[[category]]\ncode = "A"\nname = "N"\n' + VALID_COND,
        'categories.toml: unknown key "title"',
    ),
    (
        "unknown key in category",
        '[[category]]\ncode = "A"\nname = "N"\nfoo = 1\n' + VALID_COND,
        'categories.toml: category "A": unknown key "foo"',
    ),
    (
        "missing code",
        '[[category]]\nname = "N"\n' + VALID_COND,
        'categories.toml: category #1: missing "code"',
    ),
    (
        "empty code",
        '[[category]]\ncode = ""\nname = "N"\n' + VALID_COND,
        'categories.toml: category #1: "code" must not be empty',
    ),
    (
        "code not 1-3 letters/digits",
        '[[category]]\ncode = "ABCD"\nname = "N"\n' + VALID_COND,
        'categories.toml: category "ABCD": "code" must be 1-3 letters or digits',
    ),
    (
        "duplicate code",
        '[[category]]\ncode = "A"\nname = "One"\n' + VALID_COND
        + '[[category]]\ncode = "A"\nname = "Two"\n' + VALID_COND,
        'categories.toml: category "A": duplicate "code" "A"',
    ),
    (
        "missing name",
        '[[category]]\ncode = "A"\n' + VALID_COND,
        'categories.toml: category "A": missing "name"',
    ),
    (
        "empty name",
        '[[category]]\ncode = "A"\nname = ""\n' + VALID_COND,
        'categories.toml: category "A": "name" must not be empty',
    ),
    (
        "when missing on non-last",
        '[[category]]\ncode = "A"\nname = "One"\n'
        '[[category]]\ncode = "B"\nname = "Two"\n' + VALID_COND,
        'categories.toml: category "A": missing "when" on a category that is not last',
    ),
    (
        "when not a string",
        '[[category]]\ncode = "A"\nname = "N"\nwhen = 1\n',
        'categories.toml: category "A": "when" must be a string',
    ),
    (
        "condition error",
        '[[category]]\ncode = "B"\nname = "N"\nwhen = \'name foo "y"\'\n',
        'categories.toml, line 4, col 14: category "B" (when): '
        'expected a text operator (contains, equals, starts_with) after "name"',
    ),
    (
        "condition error line 2",
        '[[category]]\ncode = "B"\nname = "N"\n'
        'when = \'\'\'name contains "x"\nand name foo "y"\'\'\'\n',
        'categories.toml, line 5, col 10: category "B" (when): '
        'expected a text operator (contains, equals, starts_with) after "name"',
    ),
]


@pytest.mark.parametrize(
    "text,expected",
    [(text, expected) for _, text, expected in VALIDATION_CASES],
    ids=[name for name, _, _ in VALIDATION_CASES],
)
def test_load_rules_validation(tmp_path: Path, text: str, expected: str) -> None:
    path = _toml(tmp_path, text)
    with pytest.raises(RulesError) as excinfo:
        load_rules(path)
    assert str(excinfo.value) == expected


def test_load_rules_toml_syntax_error(tmp_path: Path) -> None:
    path = _toml(tmp_path, "[[category]\n")
    with pytest.raises(RulesError) as excinfo:
        load_rules(path)
    message = str(excinfo.value)
    assert message.startswith("categories.toml: ")
    assert "line 1" in message


def test_load_rules_time_error_uses_the_file_position(tmp_path: Path) -> None:
    path = _toml(
        tmp_path,
        '[[category]]\ncode = "A"\nname = "One"\nwhen = \'name contains "x"\'\n'
        '[[category]]\ncode = "B"\nname = "Two"\nwhen = \'rts_time < "13:99"\'\n',
    )
    with pytest.raises(RulesError) as excinfo:
        load_rules(path)
    assert str(excinfo.value) == (
        'categories.toml, line 8, col 20: category "B" (when): expected a date "YYYY-MM-DD", '
        'a time "HH:MM" or a date and time "YYYY-MM-DD HH:MM"'
    )


def test_load_rules_last_category_may_omit_when(tmp_path: Path) -> None:
    path = _toml(
        tmp_path,
        '[[category]]\ncode = "A"\nname = "One"\n' + VALID_COND
        + '[[category]]\ncode = "Z"\nname = "Rest"\n',
    )
    cats = load_rules(path)
    assert [c.code for c in cats] == ["A", "Z"]
    assert cats[-1].when is None


def test_evaluate_category_needs_app_values() -> None:
    cond = parse_condition('category equals "A"', app_fields=True)
    with pytest.raises(RulesError):
        evaluate(cond, ORDERS["o1"])
