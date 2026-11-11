"""Fjord's supported DTCG 2025.10 types and reference rules."""

from copy import deepcopy
import json
import math
from pathlib import Path
import re
from urllib.parse import unquote


EXTENSION = "org.xerrion.fjord"
TYPES = {"color", "dimension", "duration", "number", "fontFamily", "fontWeight", "cubicBezier", "shadow", "typography"}
NAME = re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*")
REFERENCE = re.compile(r"\{((?:[a-z0-9-]+|\$root)(?:\.(?:[a-z0-9-]+|\$root))*)\}")
PROPERTIES = {"$value", "$type", "$description", "$deprecated", "$extensions"}
GROUP_PROPERTIES = (PROPERTIES - {"$value"}) | {"$extends", "$ref"}


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


class TokenDocument(dict):
    """Flat tokens retaining the document structure needed by local references."""

    def __init__(self, tokens, groups, document, context):
        super().__init__(tokens)
        self.groups = groups
        self.document = document
        self.context = context
        self.defined_paths = frozenset(tokens)


def _merge_objects(base, overlay):
    result = deepcopy(base)
    for key, value in overlay.items():
        if isinstance(value, dict) and isinstance(result.get(key), dict):
            result[key] = _merge_objects(result[key], value)
        else:
            result[key] = deepcopy(value)
    return result


def merge_documents(base, overlay):
    """Merge group trees and metadata, replacing complete token definitions."""
    if not isinstance(base, dict) or not isinstance(overlay, dict):
        return deepcopy(overlay)
    if "$value" in base or "$value" in overlay:
        return deepcopy(overlay)
    result = deepcopy(base)
    for key, value in overlay.items():
        if key == "$extensions" and isinstance(value, dict) and isinstance(result.get(key), dict):
            result[key] = _merge_objects(result[key], value)
        elif not key.startswith("$") or key == "$root":
            result[key] = merge_documents(result.get(key), value)
        else:
            result[key] = deepcopy(value)
    return result


def _pointer(parts):
    return "#" + "".join("/" + str(part).replace("~", "~0").replace("/", "~1") for part in parts)


def _pointer_parts(reference):
    if not isinstance(reference, str) or not reference.startswith("#"):
        raise TokenError(f"{reference!r}: external references are unsupported; use a local JSON Pointer")
    fragment = reference[1:]
    if re.search(r"%(?![0-9a-fA-F]{2})", fragment):
        raise TokenError(f"{reference!r}: malformed JSON Pointer URI escape")
    try:
        fragment = unquote(fragment, encoding="utf-8", errors="strict")
    except UnicodeError as error:
        raise TokenError(f"{reference!r}: malformed JSON Pointer URI encoding") from error
    if fragment and not fragment.startswith("/"):
        raise TokenError(f"{reference!r}: malformed JSON Pointer; expected #/path")
    parts = fragment[1:].split("/") if fragment else []
    if any(re.search(r"~(?![01])", part) for part in parts):
        raise TokenError(f"{reference!r}: malformed JSON Pointer escape; use ~0 or ~1")
    return tuple(part.replace("~1", "/").replace("~0", "~") for part in parts)


def _child(node, part, reference):
    if isinstance(node, dict) and part in node:
        return node[part]
    if isinstance(node, list):
        if not re.fullmatch(r"0|[1-9][0-9]*", part):
            raise TokenError(f"{reference}: invalid JSON Pointer array index {part!r}")
        if len(part) <= len(str(len(node))):
            index = int(part)
            if index < len(node):
                return node[index]
    raise TokenError(f"{reference}: missing JSON Pointer target at {part!r}")


def _direct(document, parts):
    node = document
    for part in parts:
        node = _child(node, part, _pointer(parts))
    return node


def _reference(value):
    if isinstance(value, str):
        match = REFERENCE.fullmatch(value)
        if match:
            return tuple(match[1].split(".")) + ("$value",)
        if "{" in value or "}" in value:
            raise TokenError("invalid alias; use one complete {token.path} reference")
    elif isinstance(value, dict) and "$ref" in value:
        if set(value) != {"$ref"}:
            raise TokenError("a JSON Pointer reference object must contain only $ref")
        return _pointer_parts(value["$ref"])
    return None


class _Groups:
    def __init__(self, document, source):
        self.document = document
        self.source = source
        self.cache = {}

    def target(self, parts, stack):
        if parts in stack:
            chain = " -> ".join(_pointer(path) for path in stack + (parts,))
            raise TokenError(f"{self.source}: group extension cycle {chain}")
        if parts in self.cache:
            return deepcopy(self.cache[parts])
        node = self.lookup(parts, stack)
        if not isinstance(node, dict) or "$value" in node or any(p.startswith("$") for p in parts):
            raise TokenError(f"{self.source}:{_pointer(parts)}: $extends target must be a group, not a token or property")
        result = self.normalize(node, parts, stack)
        self.cache[parts] = result
        return deepcopy(result)

    def lookup(self, parts, stack):
        try:
            return _direct(self.document, parts)
        except TokenError:
            if not parts:
                raise
            parent = self.lookup(parts[:-1], stack)
            if isinstance(parent, dict) and "$value" not in parent and ("$extends" in parent or "$ref" in parent):
                parent = self.target(parts[:-1], stack)
            return _child(parent, parts[-1], _pointer(parts))

    def normalize(self, node, parts=(), stack=()):
        if not isinstance(node, dict) or "$value" in node:
            return deepcopy(node)
        if parts in stack:
            chain = " -> ".join(_pointer(path) for path in stack + (parts,))
            raise TokenError(f"{self.source}: group extension cycle {chain}")
        stack += (parts,)
        result = deepcopy(node)
        if "$extends" in node or "$ref" in node:
            if "$extends" in node and "$ref" in node:
                raise TokenError(f"{self.source}:{_pointer(parts)}: use either $extends or group $ref, not both")
            reference = node.get("$extends", node.get("$ref"))
            match = REFERENCE.fullmatch(reference) if isinstance(reference, str) else None
            if match:
                target = tuple(match[1].split("."))
            else:
                if isinstance(reference, dict) and set(reference) == {"$ref"}:
                    reference = reference["$ref"]
                target = _pointer_parts(reference)
            inherited = self.target(target, stack)
            inherited.pop("$extends", None)
            inherited.pop("$ref", None)
            result = merge_documents(inherited, node)
        for key, child in list(result.items()):
            if not key.startswith("$"):
                result[key] = self.normalize(child, parts + (key,), stack)
        return result


def _collect(document, source):
    tokens, groups, defaults = {}, {}, {}

    def visit(node, parts, inherited_type=None, inherited_deprecated=False):
        location = f"{source}:{'.'.join(parts) or '<root>'}"
        if not isinstance(node, dict):
            raise TokenError(f"{location}: expected a token or group object")
        is_token = "$value" in node
        allowed = PROPERTIES if is_token else GROUP_PROPERTIES | {"$root"}
        unknown = [key for key in node if key.startswith("$") and key not in allowed]
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
        children = [key for key in node if not key.startswith("$") or key == "$root"]
        if is_token:
            if not parts or children:
                raise TokenError(f"{location}: a token cannot also contain child tokens")
            if description is None:
                raise TokenError(f"{location}: Fjord tokens require a type and description")
            token = deepcopy(node)
            if deprecated:
                token["$deprecated"] = deprecated
            path = ".".join(parts)
            tokens[path] = token
            defaults[path] = inherited_type
            return
        groups[".".join(parts)] = {key: deepcopy(value) for key, value in node.items() if key in GROUP_PROPERTIES}
        for key in children:
            if key != "$root" and not NAME.fullmatch(key):
                raise TokenError(f"{location}: invalid name {key!r}; use lower-case hyphenated segments")
            if key == "$root" and (not isinstance(node[key], dict) or "$value" not in node[key]):
                raise TokenError(f"{location}: $root must be a token with a $value")
            visit(node[key], parts + [key], token_type, deprecated)

    visit(document, [])
    return tokens, groups, defaults


def _owner(parts, tokens):
    for size in range(len(parts), 0, -1):
        path = ".".join(parts[:size])
        if path in tokens and tuple(path.split(".")) == parts[:size]:
            return path, parts[size:]
    return None, parts


def _subtype(kind, parts):
    for index, part in enumerate(parts):
        if kind in {"dimension", "duration"}:
            kind = "number" if part == "value" else None
        elif kind == "color":
            if part == "components" and index + 1 < len(parts) and re.fullmatch(r"[0-2]", parts[index + 1]):
                return "number" if index + 2 == len(parts) else None
            kind = "number" if part == "alpha" else None
        elif kind == "typography":
            kind = {"fontFamily": "fontFamily", "fontSize": "dimension", "fontWeight": "fontWeight",
                    "letterSpacing": "dimension", "lineHeight": "number"}.get(part)
        elif kind == "shadow":
            kind = "shadow" if re.fullmatch(r"0|[1-9][0-9]*", part) else {
                "color": "color", "offsetX": "dimension", "offsetY": "dimension",
                "blur": "dimension", "spread": "dimension"}.get(part)
        elif kind in {"cubicBezier", "fontFamily"} and re.fullmatch(r"0|[1-9][0-9]*", part):
            kind = "number" if kind == "cubicBezier" else "fontFamily"
        else:
            return None
        if kind is None:
            return None
    return kind


class _Types:
    def __init__(self, tokens, defaults, document):
        self.tokens = tokens
        self.defaults = defaults
        self.document = document
        self.cache = {}

    def reference_kind(self, parts, stack):
        node = _direct(self.document, parts)
        owner, tail = _owner(parts, self.tokens)
        if owner and tail and tail[0] == "$value":
            return _subtype(self.kind(owner, stack), tail[1:])
        marker = _pointer(parts)
        if marker in stack:
            raise TokenError(f"alias cycle {' -> '.join(stack + (marker,))}")
        target = _reference(node)
        if target is not None:
            return self.reference_kind(target, stack + (marker,))
        return None

    def kind(self, path, stack=()):
        if path not in self.tokens:
            raise TokenError(f"{path}: missing alias target; define the token or fix the reference")
        if path in self.cache:
            return self.cache[path]
        if path in stack:
            raise TokenError(f"{path}: alias cycle {' -> '.join(stack + (path,))}")
        token = self.tokens[path]
        kind = token.get("$type")
        if kind is None:
            target = _reference(token["$value"])
            if target is not None:
                if isinstance(token["$value"], str):
                    kind = self.kind(".".join(target[:-1]), stack + (path,))
                else:
                    kind = self.reference_kind(target, stack + (path,))
            if kind is None:
                kind = self.defaults.get(path)
        if kind is None:
            raise TokenError(f"{path}: Fjord tokens require a type and description; no explicit, alias-derived or inherited type")
        if not isinstance(kind, str) or kind not in TYPES:
            raise TokenError(f"{path}: unsupported $type {kind!r}")
        self.cache[path] = kind
        return kind


def flatten(document, source="tokens", context=None):
    """Flatten only this layer's tokens, retaining groups and local-reference context."""
    if isinstance(context, TokenDocument):
        context = _document_for(context)
    combined = merge_documents(context, document) if context is not None else deepcopy(document)
    normalizer = _Groups(combined, source)
    full_document = normalizer.normalize(combined)
    full_tokens, full_groups, defaults = _collect(full_document, source)
    if context is None:
        overlay = full_document
        tokens, groups = full_tokens, full_groups
    else:
        overlay = normalizer.normalize(document)
        tokens, groups, _ = _collect(overlay, source)
    types = _Types(full_tokens, defaults, full_document)
    try:
        for path, token in tokens.items():
            token["$type"] = types.kind(path)
            if "$deprecated" not in token and full_tokens[path].get("$deprecated"):
                token["$deprecated"] = full_tokens[path]["$deprecated"]
    except RecursionError as error:
        raise TokenError("alias nesting exceeds this tool's limit; shorten the reference chain") from error
    return TokenDocument(tokens, groups, overlay, full_document)


def nest(tokens, metadata=None):
    """Rebuild token groups, preserving unknown metadata and reference syntax."""
    result = {}
    for path, properties in sorted(getattr(tokens, "groups", {}).items(), key=lambda item: (item[0].count("."), item[0])):
        node = result
        for part in path.split(".") if path else ():
            node = node.setdefault(part, {})
        node.update(deepcopy(properties))
    for path, token in sorted(tokens.items()):
        node = result
        parts = path.split(".")
        for part in parts[:-1]:
            node = node.setdefault(part, {})
        node[parts[-1]] = deepcopy(token)
    if metadata is not None:
        extensions = result.setdefault("$extensions", {})
        original = extensions.get(EXTENSION, {})
        extensions[EXTENSION] = _merge_objects(original, metadata) if isinstance(original, dict) else deepcopy(metadata)
    return result


def _document_for(tokens):
    if not isinstance(tokens, TokenDocument):
        return nest(tokens)
    document = deepcopy(tokens.context)
    for path in tokens.defined_paths - tokens.keys():
        parts = path.split(".")
        node = document
        for part in parts[:-1]:
            node = node[part]
        node.pop(parts[-1], None)
    return merge_documents(document, nest(tokens))


def references(value, context=None):
    """Yield referenced token owners; pointers need a document to identify owners."""
    document = _document_for(context) if isinstance(context, TokenDocument) else context

    def visit(node, seen=()):
        target = _reference(node)
        if target is not None:
            if isinstance(node, str):
                yield ".".join(target[:-1])
            elif document is not None:
                owner, _ = _document_owner(target, document)
                if owner:
                    yield owner
                if target not in seen:
                    try:
                        child = _direct(document, target)
                    except TokenError:
                        return
                    yield from visit(child, seen + (target,))
            return
        if isinstance(node, dict):
            for child in node.values():
                yield from visit(child, seen)
        elif isinstance(node, list):
            for child in node:
                yield from visit(child, seen)

    yield from visit(value)


def _document_owner(parts, document):
    node = document
    for index, part in enumerate(parts):
        if part.startswith("$") and part != "$root":
            break
        if not isinstance(node, dict) or part not in node:
            break
        node = node[part]
        if isinstance(node, dict) and "$value" in node:
            return ".".join(parts[:index + 1]), parts[index + 1:]
    return None, parts


def check_bindings(tokens):
    document = _document_for(tokens)
    for path, token in tokens.items():
        level = path.split(".")[0]
        if level not in {"primitive", "semantic", "component"}:
            raise TokenError(f"{path}: expected primitive, semantic or component namespace")
        refs = list(references(token["$value"], context=document))
        target = _reference(token["$value"])
        owner, tail = _document_owner(target, document) if target is not None else (None, ())
        if level == "component" and not (owner and owner.startswith("semantic.") and tail == ("$value",)):
            raise TokenError(f"{path}: component tokens must alias a semantic role")
        allowed = {"primitive"} if level == "primitive" else {"primitive", "semantic"}
        if any(ref.split(".")[0] not in allowed for ref in refs):
            raise TokenError(f"{path}: reference crosses the token hierarchy")


def is_finite_number(value):
    return type(value) in (int, float) and math.isfinite(value)


class Resolver:
    def __init__(self, tokens):
        self.tokens = tokens
        self.document = _document_for(tokens)
        self.lookup, _, defaults = _collect(self.document, "tokens")
        self.types = _Types(self.lookup, defaults, self.document)
        self.cache = {}
        try:
            for path, token in self.lookup.items():
                token["$type"] = self.types.kind(path)
                _direct(self.document, tuple(path.split(".")))["$type"] = token["$type"]
                if path in tokens:
                    tokens[path]["$type"] = token["$type"]
        except RecursionError as error:
            raise TokenError("alias nesting exceeds this tool's limit; shorten the reference chain") from error

    def _cycle(self, marker, stack):
        if marker in stack:
            raise TokenError(f"alias cycle {' -> '.join(stack + (marker,))}")

    def resolve(self, path, stack=()):
        address = tuple(path.split(".")) + ("$value",)
        marker = _pointer(address)
        self._cycle(marker, stack)
        if path not in self.lookup:
            raise TokenError(f"{path}: missing alias target; define the token or fix the reference")
        if path in self.cache:
            return deepcopy(self.cache[path])
        token = self.lookup[path]
        value = self.value(token["$value"], token["$type"], stack + (marker,), path, address)
        self.cache[path] = value
        return deepcopy(value)

    def _target(self, parts, stack, location):
        node = _direct(self.document, parts)
        owner, tail = _owner(parts, self.lookup)
        if owner and tail == ("$value",):
            return self.resolve(owner, stack)
        marker = _pointer(parts)
        self._cycle(marker, stack)
        return self._expand(node, stack + (marker,), location, parts)

    def _expand(self, value, stack, location, address):
        target = _reference(value)
        if target is not None:
            owner, tail = _owner(address, self.lookup)
            expected = _subtype(self.types.kind(owner), tail[1:]) if owner and tail and tail[0] == "$value" else None
            actual = (self.types.kind(".".join(target[:-1])) if isinstance(value, str)
                      else self.types.reference_kind(target, ()))
            if expected is not None and actual is not None and expected != actual:
                raise TokenError(f"{location}: alias {_pointer(target)} has type {actual}, expected {expected}")
            if isinstance(value, str):
                return self.resolve(".".join(target[:-1]), stack)
            return self._target(target, stack, location)
        if isinstance(value, dict):
            result = {}
            for key, child in value.items():
                child_address = address + (key,)
                marker = _pointer(child_address)
                self._cycle(marker, stack)
                result[key] = self._expand(child, stack + (marker,), f"{location}.{key}", child_address)
            return result
        if isinstance(value, list):
            result = []
            for index, child in enumerate(value):
                child_address = address + (str(index),)
                marker = _pointer(child_address)
                self._cycle(marker, stack)
                result.append(self._expand(child, stack + (marker,), f"{location}[{index}]", child_address))
            return result
        return value

    def value(self, value, kind, stack, location, address=(), single_shadow=False):
        def fail(message):
            raise TokenError(f"{location}: {message}")

        target = _reference(value)
        if target is not None:
            target_kind = (self.types.kind(".".join(target[:-1])) if isinstance(value, str)
                           else self.types.reference_kind(target, ()))
            if target_kind is not None and target_kind != kind:
                fail(f"alias {_pointer(target)} has type {target_kind}, expected {kind}")
            resolved = self.resolve(".".join(target[:-1]), stack) if isinstance(value, str) else self._target(target, stack, location)
            return self.value(resolved, kind, stack, location, address, single_shadow)

        if kind not in {"shadow", "typography"}:
            value = self._expand(value, stack, location, address)

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
                if single_shadow:
                    fail("shadow array elements must resolve to single shadow objects, not arrays")
                if not value:
                    fail("shadow layers cannot be empty")
                if any(isinstance(v, list) for v in value):
                    fail("shadow arrays cannot contain literal nested arrays")
                return [self.value(child, kind, stack, f"{location}[{index}]", address + (str(index),), True)
                        for index, child in enumerate(value)]
            fields = {"color": "color", "offsetX": "dimension", "offsetY": "dimension", "blur": "dimension", "spread": "dimension"}
            object_fields(tuple(fields), ("inset",))
            result = {key: self.value(value[key], field_type, stack, f"{location}.{key}", address + (key,))
                      for key, field_type in fields.items()}
            inset = self._expand(value["inset"], stack, f"{location}.inset", address + ("inset",)) if "inset" in value else None
            if result["blur"]["value"] < 0 or "inset" in value and type(inset) is not bool:
                fail("shadow blur must be non-negative and inset must be boolean")
            if "inset" in value:
                result["inset"] = inset
            return result
        elif kind == "typography":
            fields = {"fontFamily": "fontFamily", "fontSize": "dimension", "fontWeight": "fontWeight", "letterSpacing": "dimension", "lineHeight": "number"}
            object_fields(tuple(fields))
            result = {key: self.value(value[key], field_type, stack, f"{location}.{key}", address + (key,))
                      for key, field_type in fields.items()}
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
