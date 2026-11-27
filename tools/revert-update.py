"""Undo an Update of Packing on every date of a data folder, from each date's
`update v{version}\\update.toml` (D-APP-36): each file it moved goes back, each copy and each view it
wrote is deleted, and each file whose content it changed is put back from the copy beside the list.
The folder is then renamed `update v{version} reverted`.

    python revert-update.py "D:\\Packing\\packing-data" --dry-run   list what would happen, changing nothing
    python revert-update.py "D:\\Packing\\packing-data"             undo the Update

When the data folder holds Updates of more than one version, it names them and one is given after the
folder (`... "D:\\Packing\\packing-data" v0.8.0-alpha.10`). Started with nothing (a double-click), it asks
for the folder, and for the version only when there is more than one.

A date whose journal has changed since the Update (a batch saved, a revert, an amend after it) is left
as it is and named: undoing the Update there would undo that work too. A file the list names that is
not where the Update left it is left and named; the rest of the date is still undone.

Close every copy of Packing that uses the data folder first. Needs Python 3.11 or later.
"""

import hashlib
import re
import shutil
import sys
import time
from pathlib import Path

try:
    import tomllib
except ImportError:
    print("This script needs Python 3.11 or later.")
    sys.exit(2)

DATE_FOLDER = re.compile(r"^\d{4}-\d{2}-\d{2}$")
VERSION = re.compile(r"^v?(\d+\.\d+\.\d+\S*)$")
UPDATE_FOLDER = re.compile(r"^update v(\d+\.\d+\.\d+\S*)$")
LOG = "update.toml"
JOURNAL = "journal.jsonl"
RETRIES = 5


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


def same(one: Path, other: Path) -> bool:
    """Windows sees `day summary.pdf` and `Day Summary.pdf` as one file."""
    return str(one).lower() == str(other).lower()


class Run:
    def __init__(self, dry_run: bool):
        self.dry_run = dry_run
        self.reverted = 0
        self.problems = []
        # A dry run's moves, so a later step sees the folder as the earlier ones would leave it.
        self.gone = set()
        self.made = set()

    def act(self, what: str, step):
        print(f"  {what}")
        if not self.dry_run:
            with_retries(step)

    def exists(self, path: Path) -> bool:
        key = str(path).lower()
        return key in self.made or key not in self.gone and path.exists()

    def moved(self, there: Path, back: Path):
        if self.dry_run:
            self.gone.add(str(there).lower())
            self.made.add(str(back).lower())


def move_back(date_dir: Path, step: dict, run: Run):
    there, back = date_dir / step["to"], date_dir / step["from"]
    if not run.exists(there):
        if not run.exists(back):
            run.problems.append((there, "not there; the Update may have stopped before it"))
        return
    if run.exists(back) and not same(back, there):
        run.problems.append((back, f"is there again; {step['to']} is left"))
        return

    def move():
        back.parent.mkdir(parents=True, exist_ok=True)
        there.rename(back)

    run.act(f"move {step['to']} back to {step['from']}", move)
    run.moved(there, back)


def delete_copy(date_dir: Path, step: dict, run: Run):
    copy = date_dir / step["to"]
    if not copy.is_file():
        return
    if sha256_of(copy) != step["sha256"]:
        run.problems.append((copy, "changed since the Update; left"))
        return
    run.act(f"delete the copy {step['to']}", copy.unlink)


def delete_written(date_dir: Path, step: dict, run: Run):
    written = date_dir / step["file"]
    if written.is_file():
        run.act(f"delete {step['file']}, which the Update wrote", written.unlink)


def put_back(date_dir: Path, update_dir: Path, step: dict, run: Run):
    copy = update_dir / step["file"]
    if not copy.is_file() or sha256_of(copy) != step["sha256"]:
        run.problems.append((copy, "missing or changed; the file it keeps is not put back"))
        return
    target = date_dir / step["file"]
    run.act(f"put back {step['file']}", lambda: shutil.copy2(copy, target))


def revert_date(date_dir: Path, update_dir: Path, run: Run):
    log = tomllib.loads((update_dir / LOG).read_text(encoding="utf-8"))
    journal = date_dir / JOURNAL
    after = log.get("journal_after")
    if after and journal.is_file() and sha256_of(journal) != after:
        run.problems.append((date_dir, "its journal has changed since the Update; the date is left as it is"))
        return
    problems = len(run.problems)
    for step in reversed(log.get("step", [])):
        kind = step.get("kind")
        if kind == "moved":
            move_back(date_dir, step, run)
        elif kind == "copied":
            delete_copy(date_dir, step, run)
        elif kind == "written":
            delete_written(date_dir, step, run)
        elif kind == "kept":
            put_back(date_dir, update_dir, step, run)
        else:
            run.problems.append((update_dir / LOG, f"a step of kind {kind} this script does not know; left"))
    if len(run.problems) > problems:
        run.problems.append((update_dir, "keeps its name until the date is undone whole; run again after fixing"))
        return
    reverted = update_dir.with_name(f"{update_dir.name} reverted")
    if reverted.exists():
        run.problems.append((reverted, "is there already; the update folder keeps its name"))
        return
    run.act(f"rename {update_dir.name} to {reverted.name}", lambda: update_dir.rename(reverted))
    run.reverted += 1


def process(date_dir: Path, folder_name: str, run: Run):
    update_dir = date_dir / folder_name
    if not (update_dir / LOG).is_file():
        return
    print(date_dir.name)
    try:
        revert_date(date_dir, update_dir, run)
    except PermissionError as error:
        run.problems.append((Path(error.filename or date_dir), "in use by another program"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        run.problems.append((Path(getattr(error, "filename", None) or update_dir), str(error)))


def report(run: Run) -> int:
    print(f"\n{run.reverted} date(s) {'would be reverted' if run.dry_run else 'reverted'}.")
    if run.problems:
        print(f"{len(run.problems)} thing(s) left as they are:")
        for path, why in run.problems:
            print(f"  {path}: {why}")
        return 1
    return 0


def date_dirs(data_dir: Path):
    return sorted(p for p in data_dir.iterdir() if p.is_dir() and DATE_FOLDER.match(p.name))


def versions_found(data_dir: Path):
    """The versions whose Update is in the data folder and not undone yet, by name."""
    found = set()
    for date_dir in date_dirs(data_dir):
        for item in date_dir.iterdir():
            name = UPDATE_FOLDER.match(item.name)
            if name and (item / LOG).is_file():
                found.add(name.group(1))
    by_number = lambda version: [(0, int(part)) if part.isdigit() else (1, part) for part in re.split(r"[.-]", version)]
    return sorted(found, key=by_number)


def choose_version(data_dir: Path, given, asked: bool):
    """The version named, the only one there, or the one the owner picks; None to stop."""
    if given:
        version = VERSION.match(given)
        if not version:
            print(f"{given} is not a version (such as v0.8.0-alpha.10).")
        return version.group(1) if version else None
    found = versions_found(data_dir)
    if not found:
        print(f"No Update to undo in {data_dir}.")
        return None
    if len(found) == 1:
        print(f"Undoing the Update of v{found[0]}.\n")
        return found[0]
    print("More than one Update is there: " + ", ".join(f"v{version}" for version in found) + ".")
    if not asked:
        print(f'Name one: python revert-update.py "{data_dir}" v{found[-1]}')
        return None
    picked = VERSION.match(input("Which one: ").strip())
    if not picked or picked.group(1) not in found:
        print("That is not one of them.")
        return None
    return picked.group(1)


def main(argv) -> int:
    flags = {"--dry-run"}
    words = [arg for arg in argv if arg not in flags]
    unknown = [arg for arg in words if arg.startswith("--")]
    if unknown:
        print(f"Unknown option {unknown[0]}; the option is --dry-run.")
        return 2
    asked = not words
    data_dir = Path(words[0] if words else input("Data folder: ").strip().strip('"'))
    if not data_dir.is_dir():
        print(f"{data_dir} is not a folder.")
        code = 2
    else:
        code = undo(data_dir, words[1] if len(words) > 1 else None, asked, "--dry-run" in argv)
    if asked:
        input("\nPress Enter to close.")
    return code


def undo(data_dir: Path, given, asked: bool, dry_run: bool) -> int:
    version = choose_version(data_dir, given, asked)
    if version is None:
        return 2
    run = Run(dry_run)
    folder_name = f"update v{version}"
    try:
        for date_dir in date_dirs(data_dir):
            process(date_dir, folder_name, run)
    except KeyboardInterrupt:
        print("\nStopped. Run the script again to continue.")
        return 130
    return report(run)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
