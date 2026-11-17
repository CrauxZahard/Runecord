"""Rewrite the blocking `match shared.get/post/...` handlers in net/http into
transport-neutral `get_then` / `send_then_json` / `fire_then` callbacks.

The parser walks the call's parentheses and the block's braces, so multi-line
calls are handled. Only the two shapes the codebase uses are rewritten;
anything else is left alone and reported.
"""

import re
import sys
from pathlib import Path

START = re.compile(r"^    match shared\.(get|post|put|patch|delete_json|delete)\(", re.M)

VERBS = {
    "get": ("HttpMethod::Get", False),
    "post": ("HttpMethod::Post", True),
    "put": ("HttpMethod::Put", True),
    "patch": ("HttpMethod::Patch", True),
    "delete_json": ("HttpMethod::Delete", False),
    "delete": ("HttpMethod::Delete", False),
}

DECODE_BODY = re.compile(
    r"^\s*Ok\(v\) => match serde_json::from_value::<(?P<type>.+)>\(v\) \{\s*"
    r"Ok\((?P<ok>\w+)\) => (?P<event>.*),\s*"
    r"Err\(e\) => shared\.emit\(NetEvent::Error\(e\.to_string\(\)\)\),\s*"
    r"\},\s*"
    r"Err\(e\) => shared\.emit\(NetEvent::Error\(e\)\),\s*$",
    re.S,
)

PLAIN_BODY = re.compile(
    r"^\s*Ok\((?:_)\) => (?P<event>.*),\s*"
    r"Err\(e\) => shared\.emit\(NetEvent::Error\(e\)\),\s*$",
    re.S,
)

BLOCK_DECODE = re.compile(
    r"^\s*Ok\(v\) => \{\s*"
    r"if let Ok\((?P<ok>\w+)\) = serde_json::from_value::<(?P<type>.+)>\(v\) \{\s*"
    r"(?P<event>shared\.emit\(.*?\);)\s*"
    r"\}\s*"
    r"\}\s*"
    r"Err\(e\) => shared\.emit\(NetEvent::Error\(e\)\),\s*$",
    re.S,
)


def match_paren(text, open_index):
    depth = 0
    for index in range(open_index, len(text)):
        char = text[index]
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if depth == 0:
                return index
    return -1


def match_brace(text, open_index):
    depth = 0
    for index in range(open_index, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
    return -1


def split_args(text):
    depth = 0
    for index, char in enumerate(text):
        if char in "(<[{":
            depth += 1
        elif char in ")>]}":
            depth -= 1
        elif char == "," and depth == 0:
            return text[:index].strip(), text[index + 1 :].strip()
    return text.strip(), None


def convert(path: Path):
    text = path.read_text(encoding="utf-8")
    out = []
    cursor = 0
    converted = 0
    skipped = 0
    while True:
        found = START.search(text, cursor)
        if not found:
            out.append(text[cursor:])
            break
        out.append(text[cursor : found.start()])
        verb = found.group(1)
        method, has_body = VERBS[verb]
        open_paren = found.end() - 1
        close_paren = match_paren(text, open_paren)
        open_brace = text.find("{", close_paren)
        close_brace = match_brace(text, open_brace)
        if close_paren < 0 or open_brace < 0 or close_brace < 0:
            out.append(text[found.start() : found.end()])
            cursor = found.end()
            skipped += 1
            continue

        args = text[open_paren + 1 : close_paren]
        body = text[open_brace + 1 : close_brace]
        path_arg, body_arg = split_args(args)
        path_expr = path_arg.lstrip("&").strip()
        body_expr = body_arg.lstrip("&").strip().rstrip(",") if body_arg else "serde_json::Value::Null"

        decode = DECODE_BODY.match(body)
        plain = PLAIN_BODY.match(body)
        block = BLOCK_DECODE.match(body)
        if decode or block:
            source = decode or block
            kind = source.group("type").strip()
            binding = source.group("ok")
            event = source.group("event").replace("shared.emit(", "done.emit(").rstrip().rstrip(",;")
            if method == "HttpMethod::Get":
                replacement = (
                    "    let done = shared.clone();\n"
                    f"    get_then(shared, {path_expr}.to_string(), move |result: Result<{kind}, String>| match result {{\n"
                    f"        Ok({binding}) => {event},\n"
                    "        Err(e) => done.emit(NetEvent::Error(e)),\n"
                    "    });"
                )
            else:
                replacement = (
                    "    let done = shared.clone();\n"
                    f"    send_then_json(shared, {method}, {path_expr}.to_string(), {body_expr}, move |result: Result<{kind}, String>| match result {{\n"
                    f"        Ok({binding}) => {event},\n"
                    "        Err(e) => done.emit(NetEvent::Error(e)),\n"
                    "    });"
                )
            out.append(replacement)
            cursor = close_brace + 1
            converted += 1
            continue

        if plain:
            event = plain.group("event").replace("shared.emit(", "done.emit(").rstrip().rstrip(",;")
            replacement = (
                "    let done = shared.clone();\n"
                f"    fire_then(shared, {method}, {path_expr}.to_string(), move |result| match result {{\n"
                f"        Ok(()) => {event},\n"
                "        Err(e) => done.emit(NetEvent::Error(e)),\n"
                "    });"
            )
            out.append(replacement)
            cursor = close_brace + 1
            converted += 1
            continue

        out.append(text[found.start() : close_brace + 1])
        cursor = close_brace + 1
        skipped += 1

    path.write_text("".join(out), encoding="utf-8")
    return converted, skipped


if __name__ == "__main__":
    total_ok = 0
    total_skipped = 0
    for name in sys.argv[1:]:
        file = Path(name)
        ok, skipped = convert(file)
        total_ok += ok
        total_skipped += skipped
        print(f"{file.name}: converted={ok} untouched={skipped}")
    print(f"total converted={total_ok} untouched={total_skipped}")

