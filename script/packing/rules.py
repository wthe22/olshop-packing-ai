"""categories.toml and the condition language (04-rules-file)."""
from __future__ import annotations

import re
import tomllib
from dataclasses import dataclass
from datetime import date, datetime
from pathlib import Path
from typing import Any

from .orders import Order


class RulesError(Exception):
    """Invalid rules file or condition; the message names the place (file/category, line, col)."""


class Condition:
    """Parsed condition (AST root). Concrete node classes are defined by the implementation."""


@dataclass(frozen=True)
class TextComparison(Condition):
    field: str
    op: str  # contains | equals | starts_with
    value: str


@dataclass(frozen=True)
class NumberComparison(Condition):
    field: str
    op: str  # = | != | < | <= | > | >=
    value: int


@dataclass(frozen=True)
class TimeComparison(Condition):
    field: str
    op: str  # = | != | < | <= | > | >=
    value: str  # canonical: "YYYY-MM-DD", "HH:MM" or "YYYY-MM-DD HH:MM"


@dataclass(frozen=True)
class Not(Condition):
    operand: Condition


@dataclass(frozen=True)
class And(Condition):
    operands: tuple[Condition, ...]  # flattened: no direct And child


@dataclass(frozen=True)
class Or(Condition):
    operands: tuple[Condition, ...]  # flattened: no direct Or child


@dataclass(frozen=True)
class Category:
    code: str
    name: str
    when: Condition | None  # None only on the last category, and on UNCATEGORISED


UNCATEGORISED = Category(code="?", name="Uncategorised", when=None)


# field -> (type, level). level "app" exists only when parse_condition/app filters allow it.
_FIELDS: dict[str, tuple[str, str]] = {
    "name": ("text", "item"),
    "display_name": ("text", "item"),
    "variation": ("text", "item"),
    "sku_id": ("text", "item"),
    "seller_sku": ("text", "item"),
    "product_category": ("text", "item"),
    "line_quantity": ("number", "item"),
    "total_quantity": ("number", "order"),
    "distinct_items": ("number", "order"),
    "courier": ("text", "order"),
    "channel": ("text", "order"),
    "tracking_id": ("text", "order"),
    "ship_by": ("time", "order"),
    "paid_time": ("time", "order"),
    "rts_time": ("time", "order"),
    "created_time": ("time", "order"),
    "category": ("text", "app"),
    "batch": ("number", "app"),
    "group": ("number", "app"),
}

# Fields whose value comes only from the orders CSV; the CLI stops the run when a pick uses one
# and no CSV was given (03-pc-script "Errors and warnings").
CSV_ONLY_FIELDS = frozenset(
    {"sku_id", "product_category", "channel", "paid_time", "rts_time", "created_time"}
)
# Fields that live on a line, so a comparison on them is "some line matches".
ITEM_FIELDS = frozenset(
    {"name", "display_name", "variation", "sku_id", "seller_sku", "product_category", "line_quantity"}
)

_TEXT_OPS = ("contains", "equals", "starts_with")
_NUMBER_OPS = ("=", "!=", "<", "<=", ">", ">=")
_TIME_ERROR = (
    'expected a date "YYYY-MM-DD", a time "HH:MM" or a date and time "YYYY-MM-DD HH:MM"'
)
_DATE_FORM = re.compile(r"\d{4}-\d{2}-\d{2}")
_CLOCK_FORM = re.compile(r"\d{2}:\d{2}")
_DATETIME_FORM = re.compile(r"(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2})")
_IDENT_STOP = frozenset(' \t\r\n()"=<>!')

# ---------------------------------------------------------------- tokenizer


@dataclass(frozen=True)
class _Token:
    kind: str  # word | string | op | lparen | rparen | eof
    text: str
    line: int
    col: int


def _tokenize(text: str) -> list[_Token]:
    tokens: list[_Token] = []
    i = 0
    n = len(text)
    line = 1
    col = 1

    def advance(k: int = 1) -> None:
        nonlocal i, line, col
        for _ in range(k):
            if i < n and text[i] == "\n":
                line += 1
                col = 1
            else:
                col += 1
            i += 1

    while i < n:
        ch = text[i]
        if ch in " \t\r\n":
            advance()
            continue
        if ch == "(":
            tokens.append(_Token("lparen", "(", line, col))
            advance()
            continue
        if ch == ")":
            tokens.append(_Token("rparen", ")", line, col))
            advance()
            continue
        if ch == '"':
            start_line, start_col = line, col
            advance()  # opening quote
            chars: list[str] = []
            closed = False
            while i < n:
                c = text[i]
                if c == '"':
                    advance()
                    closed = True
                    break
                if c == "\\":
                    esc_line, esc_col = line, col
                    advance()
                    if i < n:
                        e = text[i]
                        if e == '"':
                            chars.append('"')
                        elif e == "\\":
                            chars.append("\\")
                        else:
                            raise RulesError(
                                f'line {esc_line}, col {esc_col}: bad escape sequence "\\{e}"'
                            )
                        advance()
                    continue
                chars.append(c)
                advance()
            if not closed:
                raise RulesError(f"line {start_line}, col {start_col}: unterminated string")
            tokens.append(_Token("string", "".join(chars), start_line, start_col))
            continue
        if ch in "=<>!":
            nxt = text[i + 1] if i + 1 < n else ""
            if nxt == "=" and ch != "=":
                tokens.append(_Token("op", ch + "=", line, col))
                advance(2)
            else:
                tokens.append(_Token("op", ch, line, col))
                advance()
            continue
        start_line, start_col = line, col
        start = i
        while i < n and text[i] not in _IDENT_STOP:
            advance()
        tokens.append(_Token("word", text[start:i], start_line, start_col))

    tokens.append(_Token("eof", "", line, col))
    return tokens


# ------------------------------------------------------------------- parser


def _valid_calendar_date(text: str) -> bool:
    try:
        date.fromisoformat(text)
    except ValueError:
        return False
    return True


def _valid_clock(text: str) -> bool:
    return 0 <= int(text[:2]) <= 23 and 0 <= int(text[3:5]) <= 59


def _normalise_time_value(text: str) -> str | None:
    """Return the canonical form of a date/time value, or None when it is malformed.

    Only the three grammar forms are accepted and each is already canonical
    (`YYYY-MM-DD`, `HH:MM`, `YYYY-MM-DD HH:MM` with one space), so the canonical form is the
    value itself; anything else is rejected rather than repaired.
    """
    value = text
    if _DATE_FORM.fullmatch(value):
        return value if _valid_calendar_date(value) else None
    if _CLOCK_FORM.fullmatch(value):
        return value if _valid_clock(value) else None
    match = _DATETIME_FORM.fullmatch(value)
    if match and _valid_calendar_date(match[1]) and _valid_clock(match[2]):
        return value
    return None


class _Parser:
    def __init__(self, tokens: list[_Token], app_fields: bool) -> None:
        self._tokens = tokens
        self._pos = 0
        self._app_fields = app_fields

    def _peek(self) -> _Token:
        return self._tokens[self._pos]

    def _advance(self) -> _Token:
        token = self._tokens[self._pos]
        self._pos += 1
        return token

    def _at_word(self, word: str) -> bool:
        token = self._peek()
        return token.kind == "word" and token.text.lower() == word

    def parse_or(self) -> Condition:
        operands = [self.parse_and()]
        while self._at_word("or"):
            self._advance()
            operands.append(self.parse_and())
        if len(operands) == 1:
            return operands[0]
        flat: list[Condition] = []
        for operand in operands:
            if isinstance(operand, Or):
                flat.extend(operand.operands)
            else:
                flat.append(operand)
        return Or(tuple(flat))

    def parse_and(self) -> Condition:
        operands = [self.parse_not()]
        while self._at_word("and"):
            self._advance()
            operands.append(self.parse_not())
        if len(operands) == 1:
            return operands[0]
        flat: list[Condition] = []
        for operand in operands:
            if isinstance(operand, And):
                flat.extend(operand.operands)
            else:
                flat.append(operand)
        return And(tuple(flat))

    def parse_not(self) -> Condition:
        if self._at_word("not"):
            self._advance()
            return Not(self.parse_not())
        return self.parse_primary()

    def parse_primary(self) -> Condition:
        token = self._peek()
        if token.kind == "lparen":
            self._advance()
            expr = self.parse_or()
            closing = self._peek()
            if closing.kind != "rparen":
                raise RulesError(f"line {closing.line}, col {closing.col}: expected ')'")
            self._advance()
            return expr
        return self.parse_comparison()

    def parse_comparison(self) -> Condition:
        token = self._peek()
        if token.kind != "word":
            raise RulesError(f'line {token.line}, col {token.col}: unknown field "{token.text}"')
        field = token.text.lower()
        if field not in _FIELDS:
            raise RulesError(f'line {token.line}, col {token.col}: unknown field "{token.text}"')
        ftype, level = _FIELDS[field]
        if level == "app" and not self._app_fields:
            raise RulesError(
                f'line {token.line}, col {token.col}: field "{field}" is allowed '
                f"in app scan filters only"
            )
        self._advance()
        if ftype == "text":
            op_token = self._peek()
            if op_token.kind != "word" or op_token.text.lower() not in _TEXT_OPS:
                raise RulesError(
                    f"line {op_token.line}, col {op_token.col}: expected a text operator "
                    f'(contains, equals, starts_with) after "{field}"'
                )
            self._advance()
            value_token = self._peek()
            if value_token.kind != "string":
                raise RulesError(
                    f"line {value_token.line}, col {value_token.col}: "
                    f'expected a string after "{field} {op_token.text.lower()}"'
                )
            self._advance()
            return TextComparison(field, op_token.text.lower(), value_token.text)
        if ftype == "time":
            op_token = self._peek()
            if op_token.kind != "op" or op_token.text not in _NUMBER_OPS:
                raise RulesError(
                    f"line {op_token.line}, col {op_token.col}: expected a number operator "
                    f'(=, !=, <, <=, >, >=) after "{field}"'
                )
            self._advance()
            value_token = self._peek()
            value = _normalise_time_value(value_token.text) if value_token.kind == "string" else None
            if value is None:
                raise RulesError(f"line {value_token.line}, col {value_token.col}: {_TIME_ERROR}")
            self._advance()
            return TimeComparison(field, op_token.text, value)
        op_token = self._peek()
        if op_token.kind != "op" or op_token.text not in _NUMBER_OPS:
            raise RulesError(
                f"line {op_token.line}, col {op_token.col}: expected a number operator "
                f'(=, !=, <, <=, >, >=) after "{field}"'
            )
        self._advance()
        value_token = self._peek()
        if value_token.kind != "word" or not value_token.text.isdigit():
            raise RulesError(
                f"line {value_token.line}, col {value_token.col}: "
                f'expected an integer after "{field} {op_token.text}"'
            )
        self._advance()
        return NumberComparison(field, op_token.text, int(value_token.text))


def parse_condition(text: str, *, app_fields: bool = False) -> Condition:
    """Raise RulesError 'line L, col C: expected …'. app_fields allows category/batch/group."""
    tokens = _tokenize(text)
    if tokens[0].kind == "eof":
        raise RulesError("line 1, col 1: expected a condition")
    parser = _Parser(tokens, app_fields)
    node = parser.parse_or()
    trailing = parser._peek()
    if trailing.kind != "eof":
        raise RulesError(
            f"line {trailing.line}, col {trailing.col}: unexpected text after the condition"
        )
    return node


# ----------------------------------------------------------------- printer


def _precedence(node: Condition) -> int:
    if isinstance(node, Or):
        return 1
    if isinstance(node, And):
        return 2
    if isinstance(node, Not):
        return 3
    return 4  # comparisons


def _escape(value: str) -> str:
    return value.replace("\\", "\\\\").replace('"', '\\"')


def _format(node: Condition, min_prec: int) -> str:
    if isinstance(node, TextComparison):
        text = f'{node.field} {node.op} "{_escape(node.value)}"'
    elif isinstance(node, NumberComparison):
        text = f"{node.field} {node.op} {node.value}"
    elif isinstance(node, TimeComparison):
        text = f'{node.field} {node.op} "{_escape(node.value)}"'
    elif isinstance(node, Not):
        text = "not " + _format(node.operand, 3)
    elif isinstance(node, And):
        text = " and ".join(_format(o, 2) for o in node.operands)
    elif isinstance(node, Or):
        text = " or ".join(_format(o, 1) for o in node.operands)
    else:
        raise TypeError(f"not a condition node: {node!r}")
    if _precedence(node) < min_prec:
        text = f"({text})"
    return text


def format_condition(cond: Condition) -> str:
    """Canonical text: lowercase keywords, one space around operators, minimal parentheses."""
    return _format(cond, 0)


# ---------------------------------------------------------------- evaluator


def _text_match(field_value: str, op: str, literal: str) -> bool:
    value = field_value.strip().lower()
    want = literal.lower()
    if op == "contains":
        return want in value
    if op == "equals":
        return value == want
    if op == "starts_with":
        return value.startswith(want)
    raise ValueError(f"unknown text operator: {op!r}")


def _number_match(field_value: int, op: str, literal: int) -> bool:
    if op == "=":
        return field_value == literal
    if op == "!=":
        return field_value != literal
    if op == "<":
        return field_value < literal
    if op == "<=":
        return field_value <= literal
    if op == ">":
        return field_value > literal
    if op == ">=":
        return field_value >= literal
    raise ValueError(f"unknown number operator: {op!r}")


def _time_literal(value: str) -> tuple[str, Any]:
    """(kind, literal) for comparisons: date, clock (hour, minute) or datetime."""
    if _DATETIME_FORM.fullmatch(value):
        return "datetime", datetime.strptime(value, "%Y-%m-%d %H:%M")
    if _DATE_FORM.fullmatch(value):
        return "date", date.fromisoformat(value)
    return "clock", (int(value[:2]), int(value[3:5]))


def _time_match(field_value: datetime | None, op: str, literal: str) -> bool:
    # No value means no match, for every operator (04-rules-file "Date and time values").
    if field_value is None:
        return False
    kind, right = _time_literal(literal)
    if kind == "date":
        left: Any = field_value.date()
    elif kind == "clock":
        left = (field_value.hour, field_value.minute)
    else:
        # Exact moment: the field keeps its seconds, so 14:00:30 < "14:00" is false.
        left = field_value
    return _number_match(left, op, right)


def _app_value(app_values: dict[str, Any] | None, field: str) -> Any:
    if app_values is None or field not in app_values:
        raise RulesError(f'no app value for "{field}"')
    return app_values[field]


def evaluate(cond: Condition, order: Order, app_values: dict[str, Any] | None = None) -> bool:
    if isinstance(cond, Not):
        return not evaluate(cond.operand, order, app_values)
    if isinstance(cond, And):
        return all(evaluate(o, order, app_values) for o in cond.operands)
    if isinstance(cond, Or):
        return any(evaluate(o, order, app_values) for o in cond.operands)
    if isinstance(cond, TextComparison):
        level = _FIELDS[cond.field][1]
        if level == "item":
            return any(
                _text_match(getattr(line, cond.field), cond.op, cond.value) for line in order.lines
            )
        if level == "order":
            return _text_match(getattr(order, cond.field), cond.op, cond.value)
        return _text_match(_app_value(app_values, cond.field), cond.op, cond.value)
    if isinstance(cond, NumberComparison):
        if cond.field == "line_quantity":
            return any(_number_match(line.quantity, cond.op, cond.value) for line in order.lines)
        if cond.field in ("total_quantity", "distinct_items"):
            return _number_match(getattr(order, cond.field), cond.op, cond.value)
        return _number_match(_app_value(app_values, cond.field), cond.op, cond.value)
    if isinstance(cond, TimeComparison):
        return _time_match(getattr(order, cond.field), cond.op, cond.value)
    raise TypeError(f"not a condition node: {cond!r}")


def fields_used(cond: Condition) -> frozenset[str]:
    """Every field name appearing in the condition (app and item fields included)."""
    if isinstance(cond, (TextComparison, NumberComparison, TimeComparison)):
        return frozenset({cond.field})
    if isinstance(cond, Not):
        return fields_used(cond.operand)
    if isinstance(cond, (And, Or)):
        used: set[str] = set()
        for operand in cond.operands:
            used |= fields_used(operand)
        return frozenset(used)
    raise TypeError(f"not a condition node: {cond!r}")


# ------------------------------------------------------------- rules file


_CATEGORY_KEYS = frozenset({"code", "name", "when"})
_CODE_RE = re.compile(r"^[A-Za-z0-9]{1,3}$")
_CONDITION_POS_RE = re.compile(r"^line (\d+), col (\d+): (.*)$", re.DOTALL)
_CATEGORY_HEADER_RE = re.compile(r"^[ \t]*\[\[[ \t]*category[ \t]*\]\]", re.MULTILINE)
_WHEN_KEY_RE = re.compile(r"^[ \t]*when[ \t]*=", re.MULTILINE)


def _locate_when_value(text: str, index: int) -> tuple[int, int] | None:
    """1-based (line, col) of the condition text for the index-th [[category]] table.

    tomllib reports no positions, so the raw file text is scanned: find the index-th
    `[[category]]` header, then its `when` key, then the start of the string content. For `'''`
    and `\"\"\"` a newline right after the opening delimiter is skipped (TOML). For a basic
    string with escapes the column is taken after the opening quote and may be slightly off.
    """
    headers = list(_CATEGORY_HEADER_RE.finditer(text))
    if index >= len(headers):
        return None
    start = headers[index].start()
    end = headers[index + 1].start() if index + 1 < len(headers) else len(text)
    chunk = text[start:end]
    key = _WHEN_KEY_RE.search(chunk)
    if key is None:
        return None
    pos = chunk.index("=", key.start()) + 1
    while pos < len(chunk) and chunk[pos] in " \t":
        pos += 1
    if chunk.startswith("'''", pos) or chunk.startswith('"""', pos):
        content = pos + 3
        if content < len(chunk) and chunk[content] == "\n":
            content += 1
    elif pos < len(chunk) and chunk[pos] in "'\"":
        content = pos + 1
    else:
        return None
    absolute = start + content
    line = text.count("\n", 0, absolute) + 1
    col = absolute - text.rfind("\n", 0, absolute)
    return line, col


def load_rules(path: Path) -> list[Category]:
    file_text = path.read_text(encoding="utf-8")
    try:
        data = tomllib.loads(file_text)
    except tomllib.TOMLDecodeError as e:
        raise RulesError(f"{path.name}: {e}") from e

    fname = path.name
    for key in data:
        if key != "category":
            raise RulesError(f'{fname}: unknown key "{key}"')

    raw = data.get("category")
    if raw is None:
        raise RulesError(f"{fname}: no categories")
    if not isinstance(raw, list):
        raise RulesError(f'{fname}: "category" must be an array of tables')
    if not raw:
        raise RulesError(f"{fname}: no categories")

    categories: list[Category] = []
    seen: set[str] = set()
    count = len(raw)
    for index, entry in enumerate(raw):
        if not isinstance(entry, dict):
            raise RulesError(f"{fname}: category #{index + 1}: must be a table")
        code_value = entry.get("code")
        # Name the category by code when there is one, else by its position.
        cid = f'"{code_value}"' if isinstance(code_value, str) and code_value else f"#{index + 1}"

        for key in entry:
            if key not in _CATEGORY_KEYS:
                raise RulesError(f'{fname}: category {cid}: unknown key "{key}"')

        if "code" not in entry:
            raise RulesError(f'{fname}: category {cid}: missing "code"')
        if not isinstance(code_value, str):
            raise RulesError(f'{fname}: category {cid}: "code" must be a string')
        if code_value == "":
            raise RulesError(f'{fname}: category {cid}: "code" must not be empty')
        if not _CODE_RE.match(code_value):
            raise RulesError(f'{fname}: category {cid}: "code" must be 1-3 letters or digits')
        if code_value in seen:
            raise RulesError(f'{fname}: category {cid}: duplicate "code" "{code_value}"')
        seen.add(code_value)

        if "name" not in entry:
            raise RulesError(f'{fname}: category {cid}: missing "name"')
        name_value = entry["name"]
        if not isinstance(name_value, str):
            raise RulesError(f'{fname}: category {cid}: "name" must be a string')
        if name_value == "":
            raise RulesError(f'{fname}: category {cid}: "name" must not be empty')

        if "when" in entry and not isinstance(entry["when"], str):
            raise RulesError(f'{fname}: category {cid}: "when" must be a string')
        when_text = entry.get("when")
        if when_text is None:
            if index != count - 1:
                raise RulesError(
                    f'{fname}: category {cid}: missing "when" on a category that is not last'
                )
            when_node: Condition | None = None
        else:
            try:
                when_node = parse_condition(when_text)
            except RulesError as e:
                where = _locate_when_value(file_text, index)
                match = _CONDITION_POS_RE.match(str(e))
                if where is not None and match is not None:
                    cond_line, cond_col = int(match[1]), int(match[2])
                    start_line, start_col = where
                    if cond_line == 1:
                        file_line, file_col = start_line, start_col + cond_col - 1
                    else:
                        file_line, file_col = start_line + cond_line - 1, cond_col
                    raise RulesError(
                        f"{fname}, line {file_line}, col {file_col}: "
                        f"category {cid} (when): {match[3]}"
                    ) from e
                raise RulesError(f"{fname}: category {cid} (when): {e}") from e

        categories.append(Category(code=code_value, name=name_value, when=when_node))
    return categories


def categorize(order: Order, categories: list[Category]) -> Category:
    """First matching category, else UNCATEGORISED."""
    for category in categories:
        if category.when is None or evaluate(category.when, order):
            return category
    return UNCATEGORISED
