"""Rename the label PDFs a data folder keeps in each date's input folder from the name of Packing
v0.6 and older, `{sha256}.pdf`, to the name of v0.7, `{date}_{pdf}_{short hash}.pdf`: the date,
the input's number on that date (two digits) and the first 12 characters of its SHA-256.

    python rename-inputs.py "D:\\Packing\\packing-data"
    python rename-inputs.py "D:\\Packing\\packing-data" --dry-run

Close every copy of Packing that uses the data folder first. Each file is renamed in one step, so
the script can be stopped at any time (or a file be open in another program) and run again: it
continues with the files that still have the old name. A file the date's journal does not name
is left as it is; the journal is only read.
"""

import hashlib
import json
import re
import sys
import time
from pathlib import Path

DATE_FOLDER = re.compile(r"^\d{4}-\d{2}-\d{2}$")
OLD_NAME = re.compile(r"^([0-9a-f]{64})\.pdf$")
RETRIES = 5


def input_numbers(journal: Path) -> dict:
    """The number each SHA-256 has on the date, from every `batch_saved` of the journal. A line
    that does not read (the torn last line of an interrupted write) is skipped."""
    numbers = {}
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
                numbers[pdf["sha256"]] = pdf["pdf"]
    return numbers


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
        self.renamed = 0
        self.duplicates = 0
        self.unnamed = []
        self.failed = []


def rename_date(date_dir: Path, dry_run: bool, tally: Tally):
    input_dir = date_dir / "input"
    journal = date_dir / "journal.jsonl"
    old = sorted(p for p in input_dir.iterdir() if OLD_NAME.match(p.name))
    if not old:
        return
    print(date_dir.name)
    numbers = input_numbers(journal) if journal.is_file() else {}
    for path in old:
        sha256 = OLD_NAME.match(path.name).group(1)
        pdf = numbers.get(sha256)
        if pdf is None:
            tally.unnamed.append(path)
            continue
        target = input_dir / f"{date_dir.name}_{pdf:02d}_{sha256[:12]}.pdf"
        try:
            if target.exists():
                # A copy of v0.7 stored the same bytes again: the old file is a second copy.
                if sha256_of(target) != sha256:
                    tally.failed.append((path, f"{target.name} is there with other bytes"))
                    continue
                if not dry_run:
                    with_retries(path.unlink)
                print(f"  {path.name}: already there as {target.name}, the old copy removed")
                tally.duplicates += 1
                continue
            if not dry_run:
                with_retries(lambda: path.rename(target))
            print(f"  {path.name} -> {target.name}")
            tally.renamed += 1
        except FileNotFoundError:
            continue
        except PermissionError:
            tally.failed.append((path, "in use by another program"))
        except OSError as error:
            tally.failed.append((path, str(error)))


def report(tally: Tally, dry_run: bool) -> int:
    verb = "would be renamed" if dry_run else "renamed"
    print(f"\n{tally.renamed} file(s) {verb}.")
    if tally.duplicates:
        print(f"{tally.duplicates} old copy(ies) of a file already renamed removed.")
    if tally.unnamed:
        print(f"{len(tally.unnamed)} file(s) no journal names, left as they are:")
        for path in tally.unnamed:
            print(f"  {path}")
    if tally.failed:
        print(f"{len(tally.failed)} file(s) not renamed; close the program that has them open and run again:")
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
            if (date_dir / "input").is_dir():
                rename_date(date_dir, dry_run, tally)
    except KeyboardInterrupt:
        print("\nStopped. Run the script again to continue.")
        return 130
    code = report(tally, dry_run)
    if asked:
        input("\nPress Enter to close.")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
