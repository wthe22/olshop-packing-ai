"""The committed testdata matches testdata/make_testdata.py and is internally consistent."""
import csv
import json
import subprocess
import sys
import tomllib

from conftest import TESTDATA


def test_generator_matches_committed_csvs(tmp_path):
    script = TESTDATA / "make_testdata.py"
    subprocess.run(
        [sys.executable, str(script), str(tmp_path)],
        check=True, capture_output=True, text=True,
    )
    for name in ("orders-1.csv", "orders-2.csv"):
        assert (tmp_path / name).read_bytes() == (TESTDATA / name).read_bytes(), name


def test_categories_file_shape():
    data = tomllib.loads((TESTDATA / "categories.toml").read_text(encoding="utf-8"))
    categories = data["category"]
    assert 3 <= len(categories) <= 4
    assert all(entry["code"] and entry["name"] for entry in categories)
    # the last category still has a condition, so "?" can be reached
    assert "when" in categories[-1]


def test_expected_json_lists_every_order_in_a_csv():
    expected = json.loads((TESTDATA / "expected.json").read_text(encoding="utf-8"))
    assert len(expected["batches"]) == 2
    assert expected["batches"][0]["orders"] + expected["batches"][1]["orders"] == len(expected["orders"])

    present = set()
    for name in ("orders-1.csv", "orders-2.csv"):
        with open(TESTDATA / name, encoding="utf-8-sig", newline="") as handle:
            for row in csv.DictReader(handle):
                present.add(row["Order ID"].strip(" \t"))
    assert set(expected["orders"]) <= present

    # export 2 keeps export 1's open orders and adds the new ones
    with open(TESTDATA / "orders-1.csv", encoding="utf-8-sig", newline="") as handle:
        first = {row["Order ID"].strip(" \t") for row in csv.DictReader(handle)}
    assert set(expected["orders"]) - first  # there are orders new in export 2
