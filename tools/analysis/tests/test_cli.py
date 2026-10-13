"""__main__.py: the `prepare` command line (1.13)."""
from __future__ import annotations

import json
from datetime import date, datetime
from pathlib import Path

import pytest
from pypdf import PdfReader

from conftest import TESTDATA
from packing.__main__ import main

SLIP = TESTDATA / "labels-slip.pdf"
PLAIN = TESTDATA / "labels-plain.pdf"
ORDERS2 = TESTDATA / "orders-2.csv"
CATEGORIES = TESTDATA / "categories.toml"

FONT = Path(r"C:\Windows\Fonts\arial.ttf")
needs_fonts = pytest.mark.skipif(not FONT.is_file(), reason="Arial font not installed")

DAY = "2026-10-07"

LABEL_ONLY_RULES = """
[[category]]
code = "X"
name = "JT"
when = 'courier starts_with "J&T"'

[[category]]
code = "Q"
name = "Rest"
"""

# A first pick on the item field `name`, then the rest.
SEPATU_AND_REST = """
[[category]]
code = "A"
name = "Sepatu"
when = 'name contains "sepatu"'

[[category]]
code = "Z"
name = "Rest"
"""

ONLY_REST = """
[[category]]
code = "Z"
name = "Rest"
"""


def scripted(answers: list[str]):
    iterator = iter(answers)

    def read(prompt: str = "") -> str:
        return next(iterator)

    return read


def rules(tmp_path: Path, text: str) -> Path:
    path = tmp_path / "categories.toml"
    path.write_text(text, encoding="utf-8")
    return path


def run(tmp_path: Path, *args: str, input_fn=None) -> int:
    argv = list(args)
    return main(argv, input_fn=input_fn) if input_fn else main(argv)


def day_dir(tmp_path: Path) -> Path:
    return tmp_path / "work" / DAY


def saved_files(directory: Path) -> list[Path]:
    return sorted(p for p in directory.glob("*.pdf") if not p.name.startswith("packing-list"))


def page_count(path: Path) -> int:
    return len(PdfReader(str(path)).pages)


def state_of(directory: Path) -> dict:
    return json.loads((directory / "state.json").read_text(encoding="utf-8"))


# ------------------------------------------------------------------------- basic runs


@needs_fonts
def test_run_with_csv_writes_files_state_and_summary(tmp_path, capsys):
    code = run(
        tmp_path,
        "prepare",
        "--labels", str(SLIP),
        "--csv", str(ORDERS2),
        "--rules", str(CATEGORIES),
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    captured = capsys.readouterr()
    assert code == 0
    out = captured.out

    directory = day_dir(tmp_path)
    names = [path.name for path in saved_files(directory)]
    assert names == [
        "1 A Sepatu ×7.pdf",
        "2 B Spion & Knalpot ×8.pdf",
        "3 C Banyak unit ×4.pdf",
        "4 Z J&T ×2.pdf",
        "5 - Uncategorised ×2.pdf",
    ]
    # page counts: order 004 (in pick B) has a second, continuation page
    assert [page_count(directory / name) for name in names] == [7, 9, 4, 2, 2]

    for number in range(1, 6):
        assert (directory / f"packing-list-{number}.pdf").is_file()

    state = state_of(directory)
    assert state["day"] == DAY
    assert len(state["saved"]) == 23
    assert {entry["pdf"] for entry in state["saved"]} == {1, 2, 3, 4, 5}
    assert all(entry["invocation"] == 1 for entry in state["saved"])
    first = next(entry for entry in state["saved"] if entry["order_id"] == "580000000000000001")
    assert first["pdf"] == 1 and first["run"] == 1
    assert first["tracking_id"] == "JY0000000101"
    assert datetime.fromisoformat(first["save_time"]).utcoffset() is not None  # save time = now, not --day

    assert "Day 2026-10-07 · 1 label file · orders-2.csv" in out
    assert "Pages: 24 read · 23 orders · 2 not in the CSV" in out
    assert "Item data: slips (24 pages), CSV" in out
    assert "Pick A  Sepatu: 7 orders → 1 A Sepatu ×7.pdf" in out
    assert "Rest ?  Uncategorised: 2 orders → 5 - Uncategorised ×2.pdf" in out
    assert "Duplicates: 0" in out
    assert "Qty Total 3 differs from the sum of Qty 2" in out
    assert "packing-list-1.pdf" in out
    # summary.txt is the screen summary
    assert (directory / "summary.txt").read_text(encoding="utf-8") == out


@needs_fonts
def test_run_without_csv_uses_slips(tmp_path, capsys):
    code = run(
        tmp_path,
        "prepare",
        "--labels", str(SLIP),
        "--rules", str(CATEGORIES),
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    out = capsys.readouterr().out
    assert code == 0
    assert "Item data: slips" in out
    assert "no CSV" in out
    assert len(saved_files(day_dir(tmp_path))) == 5
    # the SiCepat label without a CSV has no courier clue
    assert "no courier could be deduced" in out


@needs_fonts
def test_run_plain_labels_without_csv_skips_the_packing_list(tmp_path, capsys):
    code = run(
        tmp_path,
        "prepare",
        "--labels", str(PLAIN),
        "--rules", str(rules(tmp_path, LABEL_ONLY_RULES)),
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    out = capsys.readouterr().out
    assert code == 0
    directory = day_dir(tmp_path)
    assert [path.name for path in saved_files(directory)] == [
        "1 X JT ×1.pdf",
        "2 Q Rest ×2.pdf",
    ]
    assert not list(directory.glob("packing-list*.pdf"))
    assert "Item data: none" in out
    assert "no item data: the packing list was skipped" in out
    assert "no item data (no packing slip and no orders CSV)" in out


# ------------------------------------------------------------------------ errors


def test_error_writes_nothing(tmp_path, capsys):
    # Plain labels + an item pick: the pick uses `name` with no item data.
    code = run(
        tmp_path,
        "prepare",
        "--labels", str(PLAIN),
        "--csv", str(ORDERS2),  # holds none of these label orders
        "--rules", str(CATEGORIES),
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    captured = capsys.readouterr()
    assert code == 1
    assert captured.out == ""
    assert "error: " in captured.err
    assert 'pick "A" uses field "name"' in captured.err
    assert not day_dir(tmp_path).exists()


def test_unknown_layout_is_an_error(tmp_path, capsys):
    code = run(
        tmp_path,
        "prepare",
        "--labels", str(SLIP),
        "--rules", str(CATEGORIES),
        "--layout", "poster",
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    assert code == 1
    assert "unknown layout" in capsys.readouterr().err
    assert not day_dir(tmp_path).exists()


def test_missing_label_file_is_an_error(tmp_path, capsys):
    code = run(
        tmp_path,
        "prepare",
        "--labels", str(tmp_path / "absent.pdf"),
        "--rules", str(CATEGORIES),
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    assert code == 1
    assert "absent.pdf" in capsys.readouterr().err


# ------------------------------------------------------ second run / numbering / redo


@needs_fonts
def test_second_invocation_skips_duplicates_and_writes_nothing(tmp_path, capsys):
    common = (
        "prepare",
        "--labels", str(SLIP),
        "--csv", str(ORDERS2),
        "--rules", str(CATEGORIES),
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    assert run(tmp_path, *common) == 0
    directory = day_dir(tmp_path)
    before = sorted(path.name for path in directory.iterdir())
    state_before = state_of(directory)

    capsys.readouterr()
    assert run(tmp_path, *common) == 0
    out = capsys.readouterr().out
    assert "nothing to save: all 23 orders were already saved today" in out
    assert sorted(path.name for path in directory.iterdir()) == before
    assert state_of(directory) == state_before


@needs_fonts
def test_numbering_continues_across_invocations(tmp_path, capsys):
    run(
        tmp_path,
        "prepare",
        "--labels", str(SLIP),
        "--csv", str(ORDERS2),
        "--rules", str(rules(tmp_path, SEPATU_AND_REST)),
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    capsys.readouterr()
    assert run(
        tmp_path,
        "prepare",
        "--labels", str(PLAIN),
        "--rules", str(rules(tmp_path, LABEL_ONLY_RULES)),
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    ) == 0
    directory = day_dir(tmp_path)
    names = [path.name for path in saved_files(directory)]
    assert names == [
        "1 A Sepatu ×7.pdf",
        "2 Z Rest ×16.pdf",
        "3 X JT ×1.pdf",
        "4 Q Rest ×2.pdf",
    ]
    state = state_of(directory)
    assert {entry["invocation"] for entry in state["saved"]} == {1, 2}
    assert {entry["pdf"] for entry in state["saved"]} == {1, 2, 3, 4}


@needs_fonts
def test_redo_forgets_the_last_invocation_and_its_files(tmp_path, capsys):
    run(
        tmp_path,
        "prepare",
        "--labels", str(SLIP),
        "--csv", str(ORDERS2),
        "--rules", str(rules(tmp_path, SEPATU_AND_REST)),
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    capsys.readouterr()
    directory = day_dir(tmp_path)
    assert (directory / "2 Z Rest ×16.pdf").is_file()
    assert (directory / "packing-list-2.pdf").is_file()

    assert run(
        tmp_path,
        "prepare",
        "--labels", str(SLIP),
        "--csv", str(ORDERS2),
        "--rules", str(rules(tmp_path, ONLY_REST)),
        "--redo",
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    ) == 0

    names = [path.name for path in saved_files(directory)]
    assert names == ["1 Z Rest ×23.pdf"]
    assert not list(directory.glob("2 *.pdf"))
    assert not (directory / "packing-list-2.pdf").exists()
    state = state_of(directory)
    # redo forgot invocation 1, so the rerun takes that slot again (next = max + 1 = 1)
    assert {entry["invocation"] for entry in state["saved"]} == {1}
    assert {entry["pdf"] for entry in state["saved"]} == {1}
    assert len(state["saved"]) == 23


# -------------------------------------------------------------------- options


@needs_fonts
def test_several_layouts_and_whole_scope_names(tmp_path):
    run(
        tmp_path,
        "prepare",
        "--labels", str(SLIP),
        "--csv", str(ORDERS2),
        "--rules", str(CATEGORIES),
        "--layout", "full,summary",
        "--packing-list", "whole",
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    directory = day_dir(tmp_path)
    assert (directory / "packing-list-full.pdf").is_file()
    assert (directory / "packing-list-summary.pdf").is_file()


@needs_fonts
def test_packing_list_none_writes_no_list_and_no_warning(tmp_path, capsys):
    run(
        tmp_path,
        "prepare",
        "--labels", str(SLIP),
        "--csv", str(ORDERS2),
        "--rules", str(CATEGORIES),
        "--packing-list", "none",
        "--day", DAY,
        "--work", str(tmp_path / "work"),
    )
    out = capsys.readouterr().out
    directory = day_dir(tmp_path)
    assert not list(directory.glob("packing-list*.pdf"))
    assert "packing list was skipped" not in out
    assert "no packing list" in out


@needs_fonts
def test_label_wildcard_expands_and_default_day(tmp_path, monkeypatch, capsys):
    monkeypatch.chdir(tmp_path)
    code = main([
        "prepare",
        "--labels", str(TESTDATA / "labels-*.pdf"),
        "--rules", str(CATEGORIES),
        "--packing-list", "none",
    ])
    out = capsys.readouterr().out
    assert code == 0
    assert "2 label files" in out
    assert (tmp_path / "work" / date.today().isoformat()).is_dir()


# ------------------------------------------------------------------ interactive


@needs_fonts
def test_interactive_menu_with_scripted_input(tmp_path, capsys):
    answers = ["1", "c", 'name contains "knalpot"', "r"]
    code = main(
        [
            "prepare",
            "--labels", str(SLIP),
            "--csv", str(ORDERS2),
            "--rules", str(CATEGORIES),
            "--interactive",
            "--day", DAY,
            "--work", str(tmp_path / "work"),
        ],
        input_fn=scripted(answers),
    )
    out = capsys.readouterr().out
    assert code == 0
    assert "23 orders left" in out

    directory = day_dir(tmp_path)
    names = [path.name for path in saved_files(directory)]
    assert names == [
        "1 A Sepatu ×7.pdf",
        "2 C name contains -knalpot- ×4.pdf",
        "3 - Uncategorised ×12.pdf",
    ]
    assert [page_count(directory / name) for name in names] == [7, 4, 13]
    assert "Pick A  Sepatu: 7 orders" in out
    assert "Rest ?  Uncategorised: 12 orders" in out
    assert len(state_of(directory)["saved"]) == 23


@needs_fonts
def test_interactive_without_item_data_stops_before_writing(tmp_path, capsys):
    # The menu shows the label-only entries; typing an item condition stops the run.
    code = main(
        [
            "prepare",
            "--labels", str(PLAIN),
            "--rules", str(rules(tmp_path, LABEL_ONLY_RULES)),
            "--interactive",
            "--day", DAY,
            "--work", str(tmp_path / "work"),
        ],
        input_fn=scripted(["c", 'name contains "spatu"']),
    )
    captured = capsys.readouterr()
    assert code == 1
    assert "error: " in captured.err
    assert 'uses field "name"' in captured.err
    assert not day_dir(tmp_path).exists()
