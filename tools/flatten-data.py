"""Convert a data folder of Packing v0.7 and older to the flat date folders of v0.8: every file of a
date directly in its date folder, no input and output folders.

    python flatten-data.py "D:\\Packing\\packing-data"
    python flatten-data.py "D:\\Packing\\packing-data" --dry-run

In each date folder:

- each label PDF of input\\ becomes `{batch} Input {pdf} {short hash}.pdf`, once for every batch
  whose record names its bytes (found by hashing the file, so the names of v0.6 and v0.7 both work);
- each file of output\\ moves up unchanged: the journal names them;
- output\\.partial\\ (an operation that stopped) moves up as .partial\\, which the program settles
  when the date opens;
- the summaries made on request take the new names: `Day Summary.pdf`, `Day Summary - {account}.pdf`
  and `{batch} Summary - {account}.pdf`;
- session.toml, orders.csv and order-contents.toml are deleted: the program writes them again from
  the journal when the date opens;
- input\\ and output\\ are removed once empty.

Close every copy of Packing that uses the data folder first. A file is copied to a `.partial` name,
checked and renamed, and an old file goes only after its new one is in place, so the script can be
stopped at any time and run again: it continues with what is left. The journal is only read. A file
no journal names is left where it is.
"""

import hashlib
import json
import re
import shutil
import sys
import time
from pathlib import Path

DATE_FOLDER = re.compile(r"^\d{4}-\d{2}-\d{2}$")
DAY_SUMMARY = re.compile(r"^day summary\.pdf$")
DAY_SUMMARY_OF = re.compile(r"^day summary (.+)\.pdf$")
ACCOUNT_SUMMARY = re.compile(r"^(\d+) summary (.+)\.pdf$")
VIEWS = ("session.toml", "orders.csv", "order-contents.toml")
RETRIES = 5


def input_names(journal: Path) -> dict:
    """The new names of each SHA-256 on the date, from every `batch_saved` of the journal. A line
    that does not read (the torn last line of an interrupted write) is skipped."""
    names = {}
    with journal.open("r", encoding="utf-8", errors="replace") as lines:
        for line in lines:
            _, _, text = line.partition(" ")
            try:
                record = json.loads(text)
            except ValueError:
                continue
            if record.get("type") != "batch_saved":
                continue
            for pdf in record.get("inputs", []):
                sha256 = pdf["sha256"]
                name = f"{record['batch']} Input {pdf['pdf']:02d} {sha256[:12]}.pdf"
                names.setdefault(sha256, set()).add(name)
    return {sha256: sorted(found) for sha256, found in names.items()}


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as file:
        for chunk in iter(lambda: file.read(1 << 16), b""):
            digest.update(chunk)
    return digest.hexdigest()


def with_retries(step):
    """Run `step`, trying again a few times while Windows says the file is in use."""
    for attempt in range(RETRIES):
        try:
            return step()
        except PermissionError:
            if attempt == RETRIES - 1:
                raise
            time.sleep(1 + attempt)


class Tally:
    def __init__(self):
        self.moved = 0
        self.unnamed = []
        self.failed = []


def place(source: Path, target: Path, dry_run: bool, keep_source: bool, tally: Tally) -> bool:
    """Put `source`'s bytes at `target`: copied to a `.partial` name, checked and renamed when
    `keep_source`, else moved. A `target` with the same bytes is kept; one with other bytes stops."""
    if target.exists():
        if target.is_file() and sha256_of(target) == sha256_of(source):
            return True
        tally.failed.append((source, f"{target.name} is there with other bytes"))
        return False
    print(f"  {source.parent.name}\\{source.name} -> {target.name}")
    tally.moved += 1
    if dry_run:
        return True
    if keep_source:
        partial = target.with_name(target.name + ".partial")
        with_retries(lambda: shutil.copyfile(source, partial))
        if sha256_of(partial) != sha256_of(source):
            partial.unlink()
            tally.failed.append((source, "the copy does not match"))
            return False
        with_retries(lambda: partial.rename(target))
    else:
        with_retries(lambda: source.rename(target))
    return True


def flatten_inputs(date_dir: Path, dry_run: bool, tally: Tally):
    input_dir = date_dir / "input"
    if not input_dir.is_dir():
        return
    journal = date_dir / "journal.jsonl"
    names = input_names(journal) if journal.is_file() else {}
    for path in sorted(input_dir.iterdir()):
        if not path.is_file():
            continue
        if path.name.endswith(".partial"):
            print(f"  input\\{path.name}: an interrupted copy, removed")
            if not dry_run:
                with_retries(path.unlink)
            continue
        targets = names.get(sha256_of(path))
        if not targets:
            tally.unnamed.append(path)
            continue
        placed = all(place(path, date_dir / name, dry_run, True, tally) for name in targets)
        if placed and not dry_run:
            with_retries(path.unlink)


def flatten_outputs(date_dir: Path, dry_run: bool, tally: Tally):
    output_dir = date_dir / "output"
    if not output_dir.is_dir():
        return
    for path in sorted(output_dir.iterdir()):
        if path.is_file():
            place(path, date_dir / path.name, dry_run, False, tally)
    partial = output_dir / ".partial"
    if partial.is_dir():
        target = date_dir / ".partial"
        if target.exists():
            tally.failed.append((partial, "the date folder already holds a .partial folder"))
        else:
            print("  output\\.partial -> .partial")
            if not dry_run:
                with_retries(lambda: partial.rename(target))


def new_summary_name(name: str):
    """The v0.8 name of a summary made on request, or None for any other file."""
    if DAY_SUMMARY.match(name):
        return "Day Summary.pdf"
    found = DAY_SUMMARY_OF.match(name)
    if found:
        return f"Day Summary - {found.group(1)}.pdf"
    found = ACCOUNT_SUMMARY.match(name)
    if found:
        return f"{found.group(1)} Summary - {found.group(2)}.pdf"
    return None


def rename_summaries(date_dir: Path, dry_run: bool, tally: Tally):
    for path in sorted(date_dir.iterdir()):
        new = new_summary_name(path.name) if path.is_file() else None
        if new is None or new == path.name:
            continue
        print(f"  {path.name} -> {new}")
        tally.moved += 1
        if not dry_run:
            with_retries(lambda: path.rename(date_dir / new))


def remove_views(date_dir: Path, dry_run: bool):
    for name in VIEWS:
        view = date_dir / name
        if view.is_file():
            print(f"  {name}: removed, written again when the date opens")
            if not dry_run:
                with_retries(view.unlink)


def remove_if_empty(folder: Path, dry_run: bool):
    if folder.is_dir() and not any(folder.iterdir()):
        print(f"  {folder.name}\\ removed")
        if not dry_run:
            folder.rmdir()


def flatten_date(date_dir: Path, dry_run: bool, tally: Tally):
    if not ((date_dir / "input").is_dir() or (date_dir / "output").is_dir()):
        return
    print(date_dir.name)
    try:
        flatten_inputs(date_dir, dry_run, tally)
        flatten_outputs(date_dir, dry_run, tally)
        rename_summaries(date_dir, dry_run, tally)
        remove_views(date_dir, dry_run)
    except PermissionError as error:
        tally.failed.append((Path(error.filename or date_dir), "in use by another program"))
    except OSError as error:
        tally.failed.append((Path(error.filename or date_dir), str(error)))
    remove_if_empty(date_dir / "input", dry_run)
    remove_if_empty(date_dir / "output", dry_run)


def report(tally: Tally, dry_run: bool) -> int:
    verb = "would be moved or renamed" if dry_run else "moved or renamed"
    print(f"\n{tally.moved} file(s) {verb}.")
    if tally.unnamed:
        print(f"{len(tally.unnamed)} file(s) no journal names, left as they are:")
        for path in tally.unnamed:
            print(f"  {path}")
    if tally.failed:
        print(f"{len(tally.failed)} file(s) not done; close the program that has them open and run again:")
        for path, why in tally.failed:
            print(f"  {path}: {why}")
        return 1
    return 0


def main(argv) -> int:
    dry_run = "--dry-run" in argv
    folders = [arg for arg in argv if arg != "--dry-run"]
    asked = not folders
    data_dir = Path(folders[0] if folders else input("Data folder: ").strip().strip('"'))
    if not data_dir.is_dir():
        print(f"{data_dir} is not a folder.")
        return 2
    tally = Tally()
    try:
        for date_dir in sorted(p for p in data_dir.iterdir() if DATE_FOLDER.match(p.name)):
            flatten_date(date_dir, dry_run, tally)
    except KeyboardInterrupt:
        print("\nStopped. Run the script again to continue.")
        return 130
    code = report(tally, dry_run)
    if asked:
        input("\nPress Enter to close.")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
