r"""Copy a v0.4 data folder into the v0.5 layout, once (docs/software-design/data/migration.md).

    py tools\migrate-v0.4.py D:\Packing        the program folder (it holds packing-settings.toml)

Needs Python 3.11 or later and `py -m pip install xxhash`. The v0.4 folders are only read.
"""

import json
import re
import shutil
import sys
import tomllib
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path

import xxhash

SETTINGS = "packing-settings.toml"
DATE_NAME = re.compile(r"^\d{4}-\d{2}-\d{2}$")
APP = "migrate-v0.4"
KNOWN_TYPES = {
    "started", "batch_saved", "batch_reverted", "print_sent", "printed_marked",
    "output_restored", "reprint_made", "recovered",
}


class Refused(Exception):
    pass


def seal(json_bytes):
    return format(xxhash.xxh3_64_intdigest(json_bytes), "016x")


def line_of(record):
    body = json.dumps(record, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    return seal(body).encode() + b" " + body + b"\n"


def read_journal(path, date):
    """The records of a v0.4 journal; a torn last line is left out, any other damage refuses."""
    lines = path.read_bytes().split(b"\n")
    torn_tail = lines.pop() != b""  # what follows the last newline: a last line the append did not finish
    records = []
    for number, line in enumerate(lines, 1):
        is_last = number == len(lines) and not torn_tail
        record = _record_of(line)
        if record is None:
            if is_last:
                break
            raise Refused(f"{path}: line {number} is damaged")
        if record["seq"] != number or record["type"] not in KNOWN_TYPES:
            raise Refused(f"{path}: line {number} is damaged (seq or type)")
        records.append(record)
    if not records or records[0]["type"] != "started" or records[0]["date"] != date:
        raise Refused(f"{path}: it does not start with `started` of {date}")
    return records


def _record_of(line):
    if len(line) < 18 or line[16:17] != b" " or line[:16].decode("ascii", "replace") != seal(line[17:]):
        return None
    try:
        return json.loads(line[17:])
    except ValueError:
        return None


@dataclass
class LiveBatch:
    saved: dict
    first_time: str
    sent: dict = field(default_factory=dict)      # group -> time of the mark
    printed: dict = field(default_factory=dict)
    restored: dict = field(default_factory=dict)  # file -> output_restored record


@dataclass
class WorkspaceDate:
    workspace: str
    date: str
    folder: Path
    batches: dict   # old number -> LiveBatch
    reprints: list


def fold(workspace, date, folder):
    batches, reprints = {}, []
    for record in read_journal(folder / "journal.jsonl", date):
        kind, number = record["type"], record.get("batch")
        if kind == "batch_saved":
            earlier = batches.get(number)
            first = earlier.first_time if earlier and record["amends"] else record["time"]
            batches[number] = LiveBatch(record, first)
        elif kind == "batch_reverted":
            batches.pop(number, None)
        elif kind in ("print_sent", "printed_marked", "output_restored"):
            batch = batches.get(number)
            if batch is None:
                raise Refused(f"{folder / 'journal.jsonl'}: line {record['seq']} names batch {number}, not live")
            if kind == "output_restored":
                batch.restored[record["file"]] = record
            else:
                marks, key = (batch.sent, "sent") if kind == "print_sent" else (batch.printed, "printed")
                if record[key]:
                    marks[record["group"]] = record["time"]
                else:
                    marks.pop(record["group"], None)
        elif kind == "reprint_made":
            reprints.append(record)
    return WorkspaceDate(workspace, date, folder, batches, reprints)


def find_workspaces(data_dir):
    found = []
    for folder in sorted(p for p in data_dir.iterdir() if p.is_dir()):
        if DATE_NAME.match(folder.name):
            continue
        dates = [d for d in sorted(folder.iterdir()) if d.is_dir() and DATE_NAME.match(d.name)]
        if (folder / "settings.toml").is_file() or any((d / "journal.jsonl").is_file() for d in dates):
            found.append((folder, dates))
    return found


def read_all(data_dir, existing_accounts):
    """Every workspace's folds and shops, or Refused before anything is written."""
    shops, shop_owner, by_date = {}, {}, {}
    for folder, dates in find_workspaces(data_dir):
        name = folder.name
        if name.lower() in existing_accounts:
            raise Refused(f"the account \"{name}\" is already in {SETTINGS}")
        settings = folder / "settings.toml"
        shops[name] = tomllib.loads(settings.read_text("utf-8")).get("shops", []) if settings.is_file() else []
        for shop in shops[name]:
            if shop_owner.setdefault(shop.lower(), name) != name:
                raise Refused(f"the shop \"{shop}\" is listed by \"{shop_owner[shop.lower()]}\" and \"{name}\"")
        for date_dir in dates:
            if (date_dir / "output" / ".partial").exists():
                raise Refused(f"{date_dir}: output/.partial exists; open that date in v0.4 once")
            if not (date_dir / "journal.jsonl").is_file():
                continue
            if (data_dir / date_dir.name).exists():
                raise Refused(f"the date {date_dir.name} already has a folder in {data_dir}")
            by_date.setdefault(date_dir.name, []).append(fold(name, date_dir.name, date_dir))
    for date, parts in by_date.items():
        seen = {}
        for part in parts:
            for batch in part.batches.values():
                for order in batch.saved["orders"]:
                    other = seen.setdefault(order["order_id"], part.workspace)
                    if other != part.workspace:
                        raise Refused(f"{date}: order {order['order_id']} is in live batches of \"{other}\" and \"{part.workspace}\"")
    return shops, by_date


def when(text):
    return datetime.fromisoformat(text)


def batch_order(parts):
    """The live batches of a date, as (workspace date, old number, batch), in the order of first save."""
    return sorted(
        ((p, old, b) for p in parts for old, b in p.batches.items()),
        key=lambda t: (when(t[2].first_time), t[0].workspace, t[1]),
    )


def renumber(file_name, old, new):
    return re.sub(rf"^{old}(?=[- ])", str(new), file_name)


def copy_file(source, target, left_out, what):
    if source.is_file():
        shutil.copy2(source, target)
    else:
        left_out.append(f"{what} is missing: {source.name}")


def new_batch_record(part, old, batch, new, inputs, target, new_number, left_out):
    saved = dict(batch.saved)
    pdf_ids = {}
    new_inputs = []
    for item in saved["inputs"]:
        sha = item["sha256"]
        if sha not in inputs:
            inputs[sha] = len(inputs) + 1
            copy_file(part.folder / "input" / f"{sha}.pdf", target / "input" / f"{sha}.pdf", left_out, "an input")
        pdf_ids[item["pdf"]] = inputs[sha]
        new_inputs.append({**item, "pdf": inputs[sha]})

    def batch_of(number):
        return new_number.get((part.workspace, part.date, number), number)

    def left_out_copy(copy):
        kept = copy["kept"]
        kept = {**kept, "pdf": pdf_ids[kept["pdf"]]} if "pdf" in kept else {**kept, "batch": batch_of(kept["batch"])}
        return {**copy, "pdf": pdf_ids[copy["pdf"]], "kept": kept}

    outputs = []
    for output in saved["outputs"]:
        file = renumber(output["file"], old, new)
        if output["group"] is not None:  # the summary PDFs are written again by the program
            copy_file(part.folder / "output" / output["file"], target / "output" / file, left_out, "an output")
        restored = batch.restored.get(output["file"], output)
        outputs.append({**output, "file": file, "size": restored["size"], "sha256": restored["sha256"]})

    saved.update(
        batch=new,
        amends=False,
        inputs=new_inputs,
        orders=[{**o, "account": part.workspace, "shop": None, "pdf": pdf_ids[o["pdf"]]} for o in saved["orders"]],
        left_out=[left_out_copy(c) for c in saved["left_out"]],
        outputs=outputs,
    )
    for key in ("warnings", "errors"):
        saved[key] = [{**f, "pdf": None if f["pdf"] is None else pdf_ids[f["pdf"]]} for f in saved[key]]
    return saved


def mark_records(batch, new):
    marks = []
    for group in sorted(set(batch.sent) | set(batch.printed)):
        if group in batch.sent:
            marks.append({"type": "print_sent", "time": batch.sent[group], "batch": new, "group": group, "sent": True})
        if group in batch.printed:
            marks.append({"type": "printed_marked", "time": batch.printed[group], "batch": new, "group": group, "printed": True})
    return marks


def write_date(data_dir, date, parts, new_number):
    """Write one date folder from its workspaces; returns what the report needs."""
    target = data_dir / date
    (target / "input").mkdir(parents=True)
    (target / "output").mkdir()
    left_out, inputs, marks = [], {}, []
    order = batch_order(parts)
    first_time = order[0][2].first_time if order else min(r["time"] for p in parts for r in p.reprints)
    records = [{"type": "started", "time": first_time, "schema": "2.0", "date": date}]

    for new, (part, old, batch) in enumerate(order, 1):
        records.append(new_batch_record(part, old, batch, new, inputs, target, new_number, left_out))
        marks += mark_records(batch, new)
    records += marks

    reprints = sorted(
        ((p, r) for p in parts for r in p.reprints),
        key=lambda t: (when(t[1]["time"]), t[0].workspace, t[1]["seq"]),
    )
    for number, (part, made) in enumerate(reprints, 1):
        file = re.sub(r"^reprint \d+", f"reprint {number}", made["file"])
        copy_file(part.folder / "output" / made["file"], target / "output" / file, left_out, "a reprint")
        orders = [{**o, "batch": new_number.get((part.workspace, o["date"], o["batch"]), o["batch"])} for o in made["orders"]]
        records.append({**made, "number": number, "file": file, "orders": orders})

    body = b"".join(line_of(with_header(r, seq)) for seq, r in enumerate(records, 1))
    (target / "journal.jsonl").write_bytes(body)
    return order, reprints, left_out


def with_header(record, seq):
    fields = {k: v for k, v in record.items() if k not in ("seq", "time", "app", "type")}
    return {"seq": seq, "time": record["time"], "app": APP, "type": record["type"], **fields}


def toml_string(text):
    return json.dumps(text, ensure_ascii=False)


def account_table(name, shops):
    listed = ", ".join(toml_string(s) for s in shops)
    return f"\n[[accounts]]\nname = {toml_string(name)}\nshops = [{listed}]\n"


def main(argv):
    if len(argv) != 2:
        sys.exit(__doc__)
    program = Path(argv[1])
    settings_path = program / SETTINGS
    settings = tomllib.loads(settings_path.read_text("utf-8"))
    data_dir = program / settings["data_dir"]  # an absolute data_dir replaces the program folder
    existing = {a["name"].lower() for a in settings.get("accounts", [])}
    try:
        shops, by_date = read_all(data_dir, existing)
    except Refused as why:
        sys.exit(f"Nothing was written. {why}")

    new_number = {
        (p.workspace, p.date, old): n
        for date, parts in by_date.items()
        for n, (p, old, _) in enumerate(batch_order(parts), 1)
    }
    for date in sorted(by_date):
        order, reprints, left_out = write_date(data_dir, date, by_date[date], new_number)
        accounts = sorted({p.workspace for p, _, _ in order})
        orders = sum(len(b.saved["orders"]) for _, _, b in order)
        print(f"{date}: {len(order)} batches, {orders} orders, {len(reprints)} reprints, accounts: {', '.join(accounts)}")
        for text in left_out:
            print(f"  left out: {text}")

    with settings_path.open("a", encoding="utf-8", newline="") as out:
        for name, listed in shops.items():
            out.write(account_table(name, listed))
    print(f"{len(shops)} accounts added to {SETTINGS}: {', '.join(shops)}")


if __name__ == "__main__":
    main(sys.argv)
