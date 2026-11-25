"""Convert a data folder of Packing v0.7 to the flat date folders of v0.8: every file of a date directly
in its date folder, no input and output folders. Apart from --remove-backup, a file is deleted only when
another file of the date folder holds the same bytes, or when it is an unfinished copy of this script.

    python flatten-data.py "D:\\Packing\\packing-data" --backup        convert, keeping the old files
    python flatten-data.py "D:\\Packing\\packing-data"                 convert by moving the old files
    python flatten-data.py "D:\\Packing\\packing-data" --remove-backup the backup, once checked
    add --dry-run to any of them to list what would happen, changing nothing

In each date folder that still has input\\ or output\\:

- with --backup, input\\, output\\ and the summaries made on request move unchanged into
  `v0.7 backup\\`, and the new files are copied from there; without it, the files are moved;
- each label PDF of input\\ becomes `{batch} Input {pdf} {short hash}.pdf`, once for every batch whose
  record names its bytes (found by hashing the file);
- each file of output\\ goes up unchanged: the journal names them;
- output\\.partial\\ (an operation that stopped) becomes .partial\\, which the program settles;
- the summaries made on request take the new names: `Day Summary.pdf`, `Day Summary - {account}.pdf`
  and `{batch} Summary - {account}.pdf`.

The journal is only read. The views (session.toml and the others) are left: the program writes them again
when the date opens, as their version is older.

A date folder with `v0.7 backup\\` is converted again from the backup on every run: a file missing, or
holding other bytes than the journal names for it, is copied again; a wrong file is moved into
`v0.7 backup\\replaced\\`, never deleted. A file the journal names with bytes the backup does not hold
(written by the program after the conversion) is left. A date folder without input\\, output\\ and a
backup is converted already and is not touched.

--remove-backup deletes a date's backup only when every file in it is in the date folder with the same
bytes, or the journal names newer bytes for it (an amend replaced it). A summary made on request needs
only its name there, as the program may have made it again; the replaced files and output\\.partial\\ are
not checked. Otherwise it names what is missing and keeps the backup.

Close every copy of Packing that uses the data folder first. Every copy goes to a temporary name, is
checked and renamed, so the script can be stopped at any time and run again.
"""

import hashlib
import json
import os
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
REPLACED = "replaced"
TEMP = ".flatten-partial"
RETRIES = 5


class Journal:
    """What the journal names: each input's new names by SHA-256, and each output file of a live
    batch or a reprint with the bytes its newest record names (keyed by the lower-case name)."""

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


def shown(path: Path) -> str:
    """The path from inside its date folder, as the lines of the run print it."""
    parts = path.parts
    at = max(i for i, part in enumerate(parts) if DATE_FOLDER.match(part))
    return "\\".join(parts[at + 1 :])


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
        self.done = 0
        self.unnamed = []
        self.problems = []

    def act(self, text: str, step):
        print(f"  {text}")
        self.done += 1
        if not self.dry_run:
            with_retries(step)

    def copy(self, source: Path, target: Path, sha256: str) -> bool:
        """Copy `source` to `target` through a temporary name, checked against `sha256`."""
        temp = target.with_name(target.name + TEMP)
        self.act(f"{shown(source)} -> {target.name} (copied)", lambda: None)
        if self.dry_run:
            return True
        with_retries(lambda: shutil.copyfile(source, temp))
        if sha256_of(temp) != sha256:
            temp.unlink()
            self.problems.append((source, "the copy does not match"))
            return False
        with_retries(lambda: os.replace(temp, target))
        return True


def old_items(date_dir: Path):
    """What v0.7 left in the date folder: input\\, output\\ and the summaries made on request."""
    items = [date_dir / name for name in ("input", "output") if (date_dir / name).is_dir()]
    items += [p for p in sorted(date_dir.iterdir()) if p.is_file() and new_summary_name(p.name)]
    return items


def make_backup(date_dir: Path, run: Run):
    """Move the v0.7 files unchanged into the backup, then mark it complete and hand output\\.partial\\
    to the program. Until the mark is there nothing is converted, so a stopped run continues here."""
    backup = date_dir / BACKUP
    if (backup / COMPLETE).is_file():
        return
    if not run.dry_run:
        backup.mkdir(exist_ok=True)
    for item in old_items(date_dir):
        run.act(f"{item.name} -> {BACKUP}\\{item.name}", lambda item=item: item.rename(backup / item.name))
    partial = backup / "output" / ".partial"
    if not run.dry_run:
        (backup / COMPLETE).write_text("Every file of Packing v0.7 of this date is in this folder.\n")
    if partial.is_dir() and not (date_dir / ".partial").exists():
        run.act("output\\.partial -> .partial", lambda: shutil.copytree(partial, date_dir / ".partial"))


def backup_sources(backup: Path, journal: Journal, run: Run):
    """Each file of the backup with its name in the date folder and the bytes that name must hold:
    `None` for a summary made on request, which the program may have made again."""
    sources = []
    input_dir = backup / "input"
    if input_dir.is_dir():
        for path in sorted(p for p in input_dir.iterdir() if p.is_file() and not p.name.endswith(TEMP)):
            sha256 = sha256_of(path)
            names = journal.inputs.get(sha256)
            if not names:
                run.unnamed.append(path)
            for name in sorted(names or ()):
                sources.append((path, name, sha256))
    output_dir = backup / "output"
    loose = [p for p in sorted(backup.iterdir()) if p.is_file() and new_summary_name(p.name)]
    outputs = sorted(p for p in output_dir.iterdir() if p.is_file()) if output_dir.is_dir() else []
    for path in outputs + loose:
        name = new_summary_name(path.name) or path.name
        sources.append((path, name, journal.outputs.get(name.lower())))
    return sources


def from_backup(date_dir: Path, journal: Journal, run: Run):
    backup = date_dir / BACKUP
    for source, name, wanted in backup_sources(backup, journal, run):
        target = date_dir / name
        have = sha256_of(source)
        if wanted is not None and wanted != have:
            continue  # the journal names newer bytes for this file: the program's, not the backup's
        if target.is_file():
            if wanted is None or sha256_of(target) == wanted:
                continue
            aside = backup / REPLACED / name
            if not run.dry_run:
                aside.parent.mkdir(exist_ok=True)
            run.act(f"{name} holds other bytes -> {BACKUP}\\{REPLACED}\\", lambda: os.replace(target, aside))
        run.copy(source, target, have)


def move_up(source: Path, target: Path, run: Run):
    """Move `source` to `target`. A name that differs only in case is the same file on Windows: it
    is renamed. A source whose bytes another file at `target` already holds is a second copy."""
    if not target.exists() or source.samefile(target):
        run.act(f"{shown(source)} -> {target.name}", lambda: source.rename(target))
    elif target.is_file() and sha256_of(target) == sha256_of(source):
        run.act(f"{shown(source)}: a second copy of {target.name}, removed", source.unlink)
    else:
        run.problems.append((source, f"{target.name} is there with other bytes; both are kept"))


def in_place(date_dir: Path, journal: Journal, run: Run):
    input_dir = date_dir / "input"
    if input_dir.is_dir():
        for path in sorted(p for p in input_dir.iterdir() if p.is_file()):
            if path.name.endswith(TEMP):
                run.act(f"input\\{path.name}: a stopped copy of this script", path.unlink)
                continue
            sha256 = sha256_of(path)
            names = sorted(journal.inputs.get(sha256, ()))
            if not names:
                run.unnamed.append(path)
                continue
            copied = all(
                (date_dir / name).is_file() and sha256_of(date_dir / name) == sha256
                or run.copy(path, date_dir / name, sha256)
                for name in names[1:]
            )
            if copied:
                move_up(path, date_dir / names[0], run)
    output_dir = date_dir / "output"
    if output_dir.is_dir():
        for path in sorted(p for p in output_dir.iterdir() if p.is_file()):
            move_up(path, date_dir / (new_summary_name(path.name) or path.name), run)
        partial = output_dir / ".partial"
        if partial.is_dir() and not (date_dir / ".partial").exists():
            run.act("output\\.partial -> .partial", lambda: partial.rename(date_dir / ".partial"))
    for path in [p for p in sorted(date_dir.iterdir()) if p.is_file() and new_summary_name(p.name)]:
        move_up(path, date_dir / new_summary_name(path.name), run)
    for folder in (input_dir, output_dir):
        if folder.is_dir() and not any(folder.iterdir()):
            run.act(f"{folder.name}\\ (empty) removed", folder.rmdir)


def remove_backup(date_dir: Path, journal: Journal, run: Run):
    backup = date_dir / BACKUP
    if not (backup / COMPLETE).is_file() or (date_dir / "input").is_dir() or (date_dir / "output").is_dir():
        run.problems.append((backup, "the conversion is not finished; run the script without --remove-backup"))
        return
    present = {
        sha256_of(p)
        for p in date_dir.rglob("*")
        if p.is_file() and BACKUP not in p.relative_to(date_dir).parts
    }
    def kept(source: Path, name: str, wanted) -> bool:
        if new_summary_name(source.name):
            return (date_dir / name).is_file()  # may have been made again: its name is enough
        have = sha256_of(source)
        # An output the journal names newer bytes for was replaced by the program (an amend).
        return have in present or wanted is not None and wanted != have

    missing = [
        source for source, name, wanted in backup_sources(backup, journal, run) if not kept(source, name, wanted)
    ] + [p for p in run.unnamed if p.is_relative_to(backup)]
    if missing:
        for path in missing:
            run.problems.append((path, "not in the date folder; the backup is kept"))
        return
    run.act(f"{BACKUP}\\ removed", lambda: shutil.rmtree(backup))


def process(date_dir: Path, args, run: Run):
    backup = date_dir / BACKUP
    has_old = (date_dir / "input").is_dir() or (date_dir / "output").is_dir()
    if not (backup.is_dir() or has_old):
        return  # converted already, with no backup
    print(date_dir.name)
    journal = Journal(date_dir / "journal.jsonl")
    try:
        if args["remove"]:
            if backup.is_dir():
                remove_backup(date_dir, journal, run)
            else:
                print("  not converted with a backup; nothing to remove")
        elif backup.is_dir() or args["backup"]:
            make_backup(date_dir, run)
            if run.dry_run and not (backup / COMPLETE).is_file():
                print("  then each new file is copied from the backup")
            else:
                from_backup(date_dir, journal, run)
        else:
            in_place(date_dir, journal, run)
    except PermissionError as error:
        run.problems.append((Path(error.filename or date_dir), "in use by another program"))
    except OSError as error:
        run.problems.append((Path(error.filename or date_dir), str(error)))


def report(run: Run) -> int:
    print(f"\n{run.done} step(s) {'would be done' if run.dry_run else 'done'}.")
    if run.unnamed:
        print(f"{len(run.unnamed)} file(s) no journal names, left where they are:")
        for path in run.unnamed:
            print(f"  {path}")
    if run.problems:
        print(f"{len(run.problems)} file(s) not done; after fixing each, run again:")
        for path, why in run.problems:
            print(f"  {path}: {why}")
        return 1
    return 0


def main(argv) -> int:
    flags = {"--dry-run", "--backup", "--remove-backup"}
    args = {"dry_run": "--dry-run" in argv, "backup": "--backup" in argv, "remove": "--remove-backup" in argv}
    folders = [arg for arg in argv if arg not in flags]
    if args["backup"] and args["remove"]:
        print("--backup and --remove-backup do not go together.")
        return 2
    asked = not folders
    data_dir = Path(folders[0] if folders else input("Data folder: ").strip().strip('"'))
    if not data_dir.is_dir():
        print(f"{data_dir} is not a folder.")
        return 2
    run = Run(args["dry_run"])
    try:
        for date_dir in sorted(p for p in data_dir.iterdir() if p.is_dir() and DATE_FOLDER.match(p.name)):
            process(date_dir, args, run)
    except KeyboardInterrupt:
        print("\nStopped. Run the script again to continue.")
        return 130
    code = report(run)
    if asked:
        input("\nPress Enter to close.")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
