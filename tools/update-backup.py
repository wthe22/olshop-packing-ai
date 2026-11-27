"""The backup of an Update of Packing (D-APP-36): undo the Update, or delete its backup once the dates
look right. Each date folder keeps `update v{version}\\update.toml` and the copies beside it.

    python update-backup.py revert "D:\\Packing\\packing-data" [v0.8.0-alpha.11] [--dry-run]
    python update-backup.py delete "D:\\Packing\\packing-data" [v0.8.0-alpha.11] [--dry-run]

`revert` puts each file the Update moved back, deletes each copy and each view it wrote, and puts back each
file whose content it changed from the copy beside the list. The folder is then renamed
`update v{version} reverted`. A date whose journal has changed since the Update (a batch saved, a revert, an
amend after it) is left as it is and named: undoing the Update there would undo that work too. A file the
list names that is not where the Update left it is left and named; the rest of the date is still undone.

`delete` removes, on every date, the whole folder `update v{version}` and `update v{version} reverted` (left
by an earlier revert), and prints each with its size. It touches nothing outside those folders. A folder
Windows refuses (in use) is named and left; the rest go on. There is no way back: revert first if a date is
wrong.

`--dry-run` lists what would happen and changes nothing. When the data folder holds Updates of more than one
version, the version is given after the folder. Started with nothing (a double-click), the script asks for
the folder, revert or delete, the version only when there is more than one, and once more before a delete.

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
UPDATE_FOLDER = re.compile(r"^update v(\d+\.\d+\.\d+\S*?)( reverted)?$")
LOG = "update.toml"
JOURNAL = "journal.jsonl"
RETRIES = 5
MEGABYTE = 1024 * 1024


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


def folder_size(folder: Path) -> int:
    return sum(file.stat().st_size for file in folder.rglob("*") if file.is_file())


def megabytes(size: int) -> str:
    return f"{size / MEGABYTE:.1f} MB"


class Run:
    def __init__(self, dry_run: bool):
        self.dry_run = dry_run
        self.done = 0
        self.freed = 0
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
    run.done += 1


def delete_folder(date_dir: Path, folder: Path, run: Run):
    size = folder_size(folder)
    run.act(f"delete {folder.name} ({megabytes(size)})", lambda: shutil.rmtree(folder))
    run.done += 1
    run.freed += size


def revert_targets(date_dir: Path, version: str):
    folder = date_dir / f"update v{version}"
    return [folder] if (folder / LOG).is_file() else []


def delete_targets(date_dir: Path, version: str):
    names = (f"update v{version}", f"update v{version} reverted")
    return [date_dir / name for name in names if (date_dir / name).is_dir()]


COMMANDS = {
    "revert": (revert_targets, revert_date),
    "delete": (delete_targets, delete_folder),
}


def process(date_dir: Path, targets: list, handle, run: Run):
    print(date_dir.name)
    for folder in targets:
        try:
            handle(date_dir, folder, run)
        except PermissionError as error:
            run.problems.append((Path(error.filename or folder), "in use by another program; left"))
        except (OSError, tomllib.TOMLDecodeError) as error:
            run.problems.append((Path(getattr(error, "filename", None) or folder), str(error)))


def report(command: str, run: Run) -> int:
    would = "would be " if run.dry_run else ""
    if command == "delete":
        print(f"\n{run.done} update folder(s) {would}deleted, {megabytes(run.freed)} {would}freed.")
    else:
        print(f"\n{run.done} date(s) {would}reverted.")
    if run.problems:
        print(f"{len(run.problems)} thing(s) left as they are:")
        for path, why in run.problems:
            print(f"  {path}: {why}")
        return 1
    return 0


def date_dirs(data_dir: Path):
    return sorted(p for p in data_dir.iterdir() if p.is_dir() and DATE_FOLDER.match(p.name))


def versions_found(data_dir: Path, command: str):
    """The versions with a backup the command works on, in version order."""
    targets = COMMANDS[command][0]
    found = set()
    for date_dir in date_dirs(data_dir):
        for item in date_dir.iterdir():
            name = UPDATE_FOLDER.match(item.name)
            if name and targets(date_dir, name.group(1)):
                found.add(name.group(1))
    by_number = lambda version: [(0, int(part)) if part.isdigit() else (1, part) for part in re.split(r"[.-]", version)]
    return sorted(found, key=by_number)


def choose_version(data_dir: Path, command: str, given, asked: bool):
    """The version named, the only one there, or the one the owner picks; None to stop."""
    if given:
        version = VERSION.match(given)
        if not version:
            print(f"{given} is not a version (such as v0.8.0-alpha.11).")
        return version.group(1) if version else None
    found = versions_found(data_dir, command)
    if not found:
        print(f"No Update to {command} in {data_dir}.")
        return None
    if len(found) == 1:
        print(f"Working on the Update of v{found[0]}.\n")
        return found[0]
    print("More than one Update is there: " + ", ".join(f"v{version}" for version in found) + ".")
    if not asked:
        print(f'Name one: python update-backup.py {command} "{data_dir}" v{found[-1]}')
        return None
    picked = VERSION.match(input("Which one: ").strip())
    if not picked or picked.group(1) not in found:
        print("That is not one of them.")
        return None
    return picked.group(1)


def confirmed_delete(plan: list, version: str) -> bool:
    size = sum(folder_size(folder) for _, folders in plan for folder in folders)
    count = sum(len(folders) for _, folders in plan)
    answer = input(f"Delete {count} update folder(s) of v{version} ({megabytes(size)})? (y/n): ")
    return answer.strip().lower() == "y"


def work(command: str, data_dir: Path, given, asked: bool, dry_run: bool) -> int:
    version = choose_version(data_dir, command, given, asked)
    if version is None:
        return 2
    targets, handle = COMMANDS[command]
    plan = [(date_dir, found) for date_dir in date_dirs(data_dir) if (found := targets(date_dir, version))]
    if command == "delete" and asked and not dry_run and plan and not confirmed_delete(plan, version):
        print("Nothing deleted.")
        return 2
    run = Run(dry_run)
    try:
        for date_dir, found in plan:
            process(date_dir, found, handle, run)
    except KeyboardInterrupt:
        print("\nStopped. Run the script again to continue.")
        return 130
    return report(command, run)


def ask_command() -> str:
    answer = input("Revert or delete the backup of the Update? (r/d): ").strip().lower()
    return {"r": "revert", "d": "delete"}.get(answer[:1], "")


def main(argv) -> int:
    words = [arg for arg in argv if arg != "--dry-run"]
    unknown = [arg for arg in words if arg.startswith("--")]
    if unknown:
        print(f"Unknown option {unknown[0]}; the option is --dry-run.")
        return 2
    asked = not words
    if asked:
        data_dir = Path(input("Data folder: ").strip().strip('"'))
        command = ask_command()
        given = None
    else:
        command = words[0]
        data_dir = Path(words[1]) if len(words) > 1 else Path()
        given = words[2] if len(words) > 2 else None
    if command not in COMMANDS or (not asked and len(words) < 2):
        print('Usage: python update-backup.py revert|delete "<data folder>" [v<version>] [--dry-run]')
        code = 2
    elif not data_dir.is_dir():
        print(f"{data_dir} is not a folder.")
        code = 2
    else:
        code = work(command, data_dir, given, asked, "--dry-run" in argv)
    if asked:
        input("\nPress Enter to close.")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
