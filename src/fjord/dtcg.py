"""Fjord's supported DTCG 2025.10 types and reference rules."""

from copy import deepcopy
import json
import math
from pathlib import Path
import re


EXTENSION = "org.xerrion.fjord"
TYPES = {"color", "dimension", "duration", "number", "fontFamily", "fontWeight", "cubicBezier", "shadow", "typography"}
NAME = re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*")
REFERENCE = re.compile(r"\{([a-z0-9.-]+)\}")
PROPERTIES = {"$value", "$type", "$description", "$deprecated", "$extensions"}


class TokenError(ValueError):
    """A token source or layer violates the public contract."""


def unique_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key!r}; keep one definition")
        result[key] = value
    return result


def reject_nonfinite(value):
    raise ValueError(f"{value} is not valid JSON; use a finite value")


def read_json(path):
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"),
                          object_pairs_hook=unique_keys, parse_constant=reject_nonfinite)
    except (OSError, UnicodeError, ValueError) as error:
        raise TokenError(f"{path}: {error}") from error


def flatten(document, source="tokens"):
    result = {}

    def visit(node, parts, inherited_type=None, inherited_deprecated=False):
        location = f"{source}:{'.'.join(parts) or '<root>'}"
        if not isinstance(node, dict):
            raise TokenError(f"{location}: expected a token or group object")
        unknown = [key for key in node if key.startswith("$") and key not in PROPERTIES]
        if unknown:
            raise TokenError(f"{location}: unsupported properties {unknown}; see the Fjord DTCG profile")
        token_type = node.get("$type", inherited_type)
        if token_type is not None and (not isinstance(token_type, str) or token_type not in TYPES):
            raise TokenError(f"{location}: unsupported $type {token_type!r}")
        description = node.get("$description")
        if description is not None and (not isinstance(description, str) or not description.strip()):
            raise TokenError(f"{location}: $description must be a non-empty string")
        deprecated = node.get("$deprecated", inherited_deprecated)
        if not isinstance(deprecated, (bool, str)) or isinstance(deprecated, str) and not deprecated.strip():
            raise TokenError(f"{location}: $deprecated must be a boolean or explanation")
        if "$extensions" in node and not isinstance(node["$extensions"], dict):
            raise TokenError(f"{location}: $extensions must be an object")
        children = [key for key in node if not key.startswith("$")]
        if "$value" in node:
            if not parts or children:
                raise TokenError(f"{location}: a token cannot also contain child tokens")
            if token_type is None or description is None:
                raise TokenError(f"{location}: Fjord tokens require a type and description")
            token = deepcopy(node)
            token["$type"] = token_type
            if deprecated:
                token["$deprecated"] = deprecated
            result[".".join(parts)] = token
            return
        for key in children:
            if not NAME.fullmatch(key):
                raise TokenError(f"{location}: invalid name {key!r}; use lower-case hyphenated segments")
            visit(node[key], parts + [key], token_type, deprecated)

    visit(document, [])
    return result


def nest(tokens, metadata=None):
    result = {"$extensions": {EXTENSION: metadata}} if metadata else {}
    for path, token in sorted(tokens.items()):
        node = result
        parts = path.split(".")
        for part in parts[:-1]:
            node = node.setdefault(part, {})
        node[parts[-1]] = deepcopy(token)
    return result


def references(value):
    if isinstance(value, str):
        match = REFERENCE.fullmatch(value)
        if match:
            yield match[1]
    elif isinstance(value, dict):
        for child in value.values():
            yield from references(child)
    elif isinstance(value, list):
        for child in value:
            yield from references(child)


def check_bindings(tokens):
    for path, token in tokens.items():
        level = path.split(".")[0]
        if level not in {"primitive", "semantic", "component"}:
            raise TokenError(f"{path}: expected primitive, semantic or component namespace")
        refs = list(references(token["$value"]))
        if level == "component" and not (
            isinstance(token["$value"], str) and REFERENCE.fullmatch(token["$value"])
            and refs[0].startswith("semantic.")
        ):
            raise TokenError(f"{path}: component tokens must alias a semantic role")
        allowed = {"primitive"} if level == "primitive" else {"primitive", "semantic"}
        if any(ref.split(".")[0] not in allowed for ref in refs):
            raise TokenError(f"{path}: reference crosses the token hierarchy")


def is_finite_number(value):
    return type(value) in (int, float) and math.isfinite(value)


class Resolver:
    def __init__(self, tokens):
        self.tokens = tokens
        self.cache = {}

    def resolve(self, path, stack=()):
        if path in stack:
            raise TokenError(f"{path}: alias cycle {' -> '.join(stack + (path,))}")
        if path not in self.tokens:
            raise TokenError(f"{path}: missing alias target; define the token or fix the reference")
        if path in self.cache:
            return deepcopy(self.cache[path])
        token = self.tokens[path]
        value = self.value(token["$value"], token["$type"], stack + (path,), path)
        self.cache[path] = value
        return deepcopy(value)

    def value(self, value, kind, stack, location):
        def fail(message):
            raise TokenError(f"{location}: {message}")

        if isinstance(value, str):
            alias = REFERENCE.fullmatch(value)
            if alias:
                target = alias[1]
                if target not in self.tokens:
                    fail(f"missing alias target {target}")
                if self.tokens[target]["$type"] != kind:
                    fail(f"alias {target} has type {self.tokens[target]['$type']}, expected {kind}")
                return self.resolve(target, stack)
            if "{" in value or "}" in value:
                fail("invalid alias; use one complete {token.path} reference")

        def object_fields(required, optional=()):
            if not isinstance(value, dict) or not set(required) <= value.keys() or value.keys() - set(required) - set(optional):
                fail(f"{kind} requires fields {', '.join(required)}")

        if kind == "color":
            object_fields(("colorSpace", "components"), ("alpha", "hex"))
            components = value["components"]
            if value["colorSpace"] != "srgb" or not isinstance(components, list) or len(components) != 3:
                fail("Fjord colours require srgb and three components")
            if not all(is_finite_number(c) and 0 <= c <= 1 for c in components):
                fail("srgb components must be finite numbers from 0 to 1")
            alpha = value.get("alpha", 1)
            if not is_finite_number(alpha) or not 0 <= alpha <= 1:
                fail("alpha must be a finite number from 0 to 1")
            if "hex" in value and (not isinstance(value["hex"], str) or not re.fullmatch(r"#[0-9a-fA-F]{6}", value["hex"])):
                fail("hex fallback must use #RRGGBB")
            if "hex" in value and value["hex"].lower() != hex_color(value)[:7]:
                fail("hex fallback must match the rounded srgb components")
        elif kind in ("dimension", "duration"):
            object_fields(("value", "unit"))
            units = ("px", "rem") if kind == "dimension" else ("ms", "s")
            if not is_finite_number(value["value"]) or value["unit"] not in units:
                fail(f"{kind} requires a finite value and unit in {units}")
            if kind == "duration" and value["value"] < 0:
                fail("duration cannot be negative")
        elif kind == "number":
            if not is_finite_number(value):
                fail("expected a finite number")
        elif kind == "fontWeight":
            if not is_finite_number(value) or not 1 <= value <= 1000:
                fail("Fjord font weights must be numbers from 1 to 1000")
        elif kind == "fontFamily":
            names = [value] if isinstance(value, str) else value
            if not isinstance(names, list) or not names or any(not isinstance(n, str) or not n.strip() for n in names):
                fail("fontFamily requires a family name or a non-empty list of names")
        elif kind == "cubicBezier":
            if not isinstance(value, list) or len(value) != 4 or not all(is_finite_number(n) for n in value):
                fail("cubicBezier requires four finite numbers")
            if not 0 <= value[0] <= 1 or not 0 <= value[2] <= 1:
                fail("cubicBezier x coordinates must be between 0 and 1")
        elif kind == "shadow":
            if isinstance(value, list):
                if not value:
                    fail("shadow layers cannot be empty")
                if any(isinstance(v, list) for v in value):
                    fail("shadow arrays cannot contain literal nested arrays; use a shadow token reference")
                layers = [self.value(v, kind, stack, f"{location}[{i}]") for i, v in enumerate(value)]
                return [part for layer in layers for part in (layer if isinstance(layer, list) else [layer])]
            fields = {"color": "color", "offsetX": "dimension", "offsetY": "dimension", "blur": "dimension", "spread": "dimension"}
            object_fields(tuple(fields), ("inset",))
            result = {k: self.value(value[k], t, stack, f"{location}.{k}") for k, t in fields.items()}
            if result["blur"]["value"] < 0 or "inset" in value and type(value["inset"]) is not bool:
                fail("shadow blur must be non-negative and inset must be boolean")
            if "inset" in value:
                result["inset"] = value["inset"]
            return result
        elif kind == "typography":
            fields = {"fontFamily": "fontFamily", "fontSize": "dimension", "fontWeight": "fontWeight", "letterSpacing": "dimension", "lineHeight": "number"}
            object_fields(tuple(fields))
            result = {k: self.value(value[k], t, stack, f"{location}.{k}") for k, t in fields.items()}
            if result["fontSize"]["value"] <= 0 or result["lineHeight"] <= 0:
                fail("typography fontSize and lineHeight must be positive")
            return result
        else:
            fail(f"unsupported type {kind}")
        return deepcopy(value)

    def all(self):
        check_bindings(self.tokens)
        try:
            return {path: self.resolve(path) for path in self.tokens}
        except RecursionError as error:
            raise TokenError("alias nesting exceeds this tool's limit; shorten the reference chain") from error


def hex_color(value):
    components = value["components"] + [value.get("alpha", 1)]
    parts = [int(c * 255 + 0.5) for c in components]
    return "#" + "".join(f"{c:02x}" for c in (parts[:3] if parts[3] == 255 else parts))


def color_from_hex(value):
    if not isinstance(value, str) or not re.fullmatch(r"#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?", value):
        raise TokenError(f"invalid hex colour {value!r}")
    return {"colorSpace": "srgb", "components": [int(value[i:i + 2], 16) / 255 for i in (1, 3, 5)],
            "alpha": int(value[7:9], 16) / 255 if len(value) == 9 else 1}
