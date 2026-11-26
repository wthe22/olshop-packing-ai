"""Remove the `v0.7 backup\\` folders Packing keeps when its Update moves a date folder of v0.7 to the
flat date folders of v0.8. A date's backup is deleted only when every file in it is in the date folder.

    python remove-v07-backup.py "D:\\Packing\\packing-data"             remove each backup that checks out
    python remove-v07-backup.py "D:\\Packing\\packing-data" --dry-run   list what would happen, changing nothing

Started with no data folder (a double-click), it asks for the folder.

A backup checks out when its date was converted to the end (the backup holds `backup complete.txt`, and
the date folder has no input\\ or output\\ left) and each file of it is in the date folder with the same
bytes, or the journal names newer bytes for it (an amend replaced it). A label PDF goes up under the name
of every batch whose record names its bytes (found by hashing the file); a summary made on request needs
only its new name there, as the program may have made it again; the replaced files and output\\.partial\\
are not checked. Otherwise it names what is missing and keeps the backup. A file no journal names stays in
its backup until the owner moves it.

Close every copy of Packing that uses the data folder first.
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
BACKUP = "v0.7 backup"
COMPLETE = "backup complete.txt"
RETRIES = 5


class Journal:
    """What the journal names: each input's names in the date folder by SHA-256, and each output file
    of a live batch or a reprint with the bytes its newest record names (keyed by the lower-case name)."""

    def __init__(self, path: Path):
        self.inputs = {}
        self.outputs = {}
        live = {}
        reprints = {}
        if path.is_file():
            for record in records(path):
                kind = record.get("type")
                batch = record.get("batch")
                if kind == "batch_saved":
                    for pdf in record.get("inputs", []):
                        sha256 = pdf["sha256"]
                        name = f"{batch} Input {pdf['pdf']:02d} {sha256[:12]}.pdf"
                        self.inputs.setdefault(sha256, set()).add(name)
                    live[batch] = {o["file"].lower(): o["sha256"] for o in record.get("outputs", [])}
                elif kind == "batch_reverted":
                    live.pop(batch, None)
                elif kind == "output_restored" and batch in live:
                    live[batch][record["file"].lower()] = record["sha256"]
                elif kind == "reprint_made":
                    reprints[record["file"].lower()] = record["sha256"]
        for files in live.values():
            self.outputs.update(files)
        self.outputs.update(reprints)


def records(path: Path):
    """The records of a journal; a line that does not read (a torn last line) is skipped."""
    with path.open("r", encoding="utf-8", errors="replace") as lines:
        for line in lines:
            _, _, text = line.partition(" ")
            try:
                yield json.loads(text)
            except ValueError:
                continue


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


class Run:
    def __init__(self, dry_run: bool):
        self.dry_run = dry_run
        self.removed = 0
        self.problems = []


def backup_sources(backup: Path, journal: Journal):
    """Each file of the backup with its names in the date folder and the bytes those must hold:
    `None` for a summary made on request, which the program may have made again. A label PDF no
    journal names has no name."""
    sources = []
    input_dir = backup / "input"
    if input_dir.is_dir():
        for path in sorted(p for p in input_dir.iterdir() if p.is_file()):
            sha256 = sha256_of(path)
            sources.append((path, sorted(journal.inputs.get(sha256, ())), sha256))
    output_dir = backup / "output"
    loose = [p for p in sorted(backup.iterdir()) if p.is_file() and new_summary_name(p.name)]
    outputs = sorted(p for p in output_dir.iterdir() if p.is_file()) if output_dir.is_dir() else []
    for path in outputs + loose:
        name = new_summary_name(path.name) or path.name
        sources.append((path, [name], journal.outputs.get(name.lower())))
    return sources


def remove_backup(date_dir: Path, journal: Journal, run: Run):
    backup = date_dir / BACKUP
    if not (backup / COMPLETE).is_file() or (date_dir / "input").is_dir() or (date_dir / "output").is_dir():
        run.problems.append((backup, "the date is not converted yet; start Packing and choose Update"))
        return
    present = {
        sha256_of(p)
        for p in date_dir.rglob("*")
        if p.is_file() and BACKUP not in p.relative_to(date_dir).parts
    }

    def kept(source: Path, names, wanted) -> bool:
        if not names:
            return False  # no journal names it: the owner decides
        if new_summary_name(source.name):
            return (date_dir / names[0]).is_file()  # may have been made again: its name is enough
        have = sha256_of(source)
        # An output the journal names newer bytes for was replaced by the program (an amend).
        return have in present or wanted is not None and wanted != have

    missing = [source for source, names, wanted in backup_sources(backup, journal) if not kept(source, names, wanted)]
    if missing:
        for path in missing:
            run.problems.append((path, "not in the date folder; the backup is kept"))
        return
    print(f"  {BACKUP}\\ {'would be removed' if run.dry_run else 'removed'}")
    run.removed += 1
    if not run.dry_run:
        with_retries(lambda: shutil.rmtree(backup))


def process(date_dir: Path, run: Run):
    if not (date_dir / BACKUP).is_dir():
        return
    print(date_dir.name)
    try:
        remove_backup(date_dir, Journal(date_dir / "journal.jsonl"), run)
    except PermissionError as error:
        run.problems.append((Path(error.filename or date_dir), "in use by another program"))
    except OSError as error:
        run.problems.append((Path(error.filename or date_dir), str(error)))


def report(run: Run) -> int:
    print(f"\n{run.removed} backup(s) {'would be removed' if run.dry_run else 'removed'}.")
    if run.problems:
        print(f"{len(run.problems)} file(s) kept; after fixing each, run again:")
        for path, why in run.problems:
            print(f"  {path}: {why}")
        return 1
    return 0


def main(argv) -> int:
    flags = {"--dry-run"}
    folders = [arg for arg in argv if arg not in flags]
    unknown = [arg for arg in folders if arg.startswith("--")]
    if unknown:
        print(f"Unknown option {unknown[0]}; the option is --dry-run.")
        return 2
    asked = not folders
    data_dir = Path(folders[0] if folders else input("Data folder: ").strip().strip('"'))
    if not data_dir.is_dir():
        print(f"{data_dir} is not a folder.")
        return 2
    run = Run("--dry-run" in argv)
    try:
        for date_dir in sorted(p for p in data_dir.iterdir() if p.is_dir() and DATE_FOLDER.match(p.name)):
            process(date_dir, run)
    except KeyboardInterrupt:
        print("\nStopped. Run the script again to continue.")
        return 130
    code = report(run)
    if asked:
        input("\nPress Enter to close.")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
