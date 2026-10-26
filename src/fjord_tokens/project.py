"""Load a token project, apply ordered layers and check resolved output."""

from copy import deepcopy
from dataclasses import dataclass, field
from pathlib import Path
import re

from .contract import THEMES, mode_roles, required_roles
from .color import design_seed_values
from .code_roles import check_syntax_distances, validate_bindings
from .dtcg import EXTENSION, Resolver, TokenError, color_from_hex, flatten, hex_color, read_json
from .quality import Report, check_platforms, contrast_rules
from .layout_rules import check_layout
from .usage import load_usage, usage_pairs, validate_usage


VERSION = "0.3.0"
SCHEMA_VERSION = 2


def metadata():
    return {"schemaVersion": SCHEMA_VERSION, "contractVersion": VERSION, "dtcgVersion": "2025.10"}


def compatible(version):
    if not isinstance(version, str) or not re.fullmatch(r"0\.3\.(0|[1-9][0-9]*)", version):
        raise TokenError(f"unsupported contract version {version!r}; expected a 0.3.x contract")


def check_metadata(document, source):
    extensions = document.get("$extensions", {}) if isinstance(document, dict) else {}
    metadata = extensions.get(EXTENSION) if isinstance(extensions, dict) else None
    if not isinstance(metadata, dict) or type(metadata.get("schemaVersion")) is not int or metadata["schemaVersion"] != SCHEMA_VERSION:
        raise TokenError(f"{source}: declare schemaVersion 2 in $extensions.{EXTENSION}")
    compatible(metadata.get("contractVersion"))
    if metadata.get("dtcgVersion") != "2025.10":
        raise TokenError(f"{source}: expected dtcgVersion 2025.10")


def read_layer(path):
    document = read_json(path)
    check_metadata(document, path)
    return flatten(document, str(path))


@dataclass
class Composition:
    tokens: dict
    values: dict
    provenance: dict
    mode: str
    usage: dict


@dataclass
class ValidationReport(Report):
    deprecated: list = field(default_factory=list)


def resolve_values(tokens):
    values = Resolver(tokens).all()
    for path, value in values.items():
        if path.startswith(("semantic.spacing.", "semantic.radius.", "semantic.border-width.", "semantic.font-size.")):
            if value["value"] < 0:
                raise TokenError(f"{path}: semantic dimensions must not be negative")
            if path.startswith("semantic.font-size.") and value["value"] == 0:
                raise TokenError(f"{path}: font size must be positive")
        if path.startswith("semantic.z-index.") and not float(value).is_integer():
            raise TokenError(f"{path}: stacking order must be an integer")
    check_layout(values)
    return values


class Project:
    def __init__(self, root):
        self.root = Path(root)
        self.directory = self.root / "tokens"
        self.contract = read_json(self.directory / "contract.json")
        c = self.contract
        if not isinstance(c, dict) or type(c.get("schemaVersion")) is not int or c["schemaVersion"] != SCHEMA_VERSION:
            raise TokenError("tokens/contract.json: unsupported schemaVersion; expected 2")
        compatible(c.get("version"))
        if c.get("dtcgVersion") != "2025.10":
            raise TokenError("tokens/contract.json: expected dtcgVersion 2025.10")
        if not isinstance(c.get("modes"), dict) or set(c["modes"]) != set(THEMES):
            raise TokenError("tokens/contract.json: declare night, day and aurora exactly once")
        if not isinstance(c.get("required"), dict):
            raise TokenError("tokens/contract.json: required must map token paths to types")
        for name, kind in required_roles().items():
            if c["required"].get(name) != kind:
                raise TokenError(f"tokens/contract.json: required role {name} must have type {kind}")
        if c.get("mergeOrder") != ["base", "mode", "theme", "overrides"]:
            raise TokenError("tokens/contract.json: mergeOrder must be base, mode, theme, overrides")
        if c.get("overrideNamespaces") != ["semantic", "component"]:
            raise TokenError("tokens/contract.json: only semantic and component overrides are allowed")
        self.usage = load_usage(self.source(c.get("usage")))

    def source(self, name):
        if not isinstance(name, str):
            raise TokenError("tokens/contract.json: source paths must be strings")
        path = (self.directory / name).resolve()
        if not path.is_relative_to(self.directory.resolve()):
            raise TokenError(f"tokens/contract.json: source {name!r} must stay inside tokens/")
        return path

    def check_required(self, tokens):
        for path, kind in self.contract["required"].items():
            if path not in tokens:
                raise TokenError(f"{path}: missing required role; restore its definition")
            if tokens[path]["$type"] != kind:
                raise TokenError(f"{path}: required type is {kind}, got {tokens[path]['$type']}")

    def compose(self, mode, theme=None, overrides=()):
        try:
            return self._compose(mode, theme, overrides)
        except TokenError as error:
            raise TokenError(f"[mode={mode}] {error}") from error

    def _compose(self, mode, theme=None, overrides=()):
        if mode not in THEMES:
            raise TokenError(f"unknown mode {mode!r}; choose night, day or aurora")
        base_path = self.source(self.contract.get("base"))
        tokens = read_layer(base_path)
        if set(tokens) & mode_roles():
            raise TokenError("base must not define mode roles; each mode must supply its own complete values")
        provenance = {path: "base" for path in tokens}
        mode_path = self.source(self.contract["modes"][mode])
        mode_tokens = read_layer(mode_path)
        if set(mode_tokens) != mode_roles():
            missing = sorted(mode_roles() - mode_tokens.keys())
            extra = sorted(mode_tokens.keys() - mode_roles())
            raise TokenError(f"{mode_path}: missing mode roles {missing}; unexpected roles {extra}")
        tokens.update(mode_tokens)
        provenance.update({path: f"mode:{mode}" for path in mode_tokens})
        self.check_required(tokens)
        values = resolve_values(tokens)
        theme_path = Path(theme) if theme is not None else self.source(self.contract.get("theme"))
        layers = [("theme", theme_path)] + [(f"override:{i + 1}", Path(p)) for i, p in enumerate(overrides)]
        for label, path in layers:
            patch = read_layer(path)
            for name, token in patch.items():
                if name not in tokens:
                    raise TokenError(f"{path}:{name}: unknown override path; add new roles to the source contract first")
                if name.split(".")[0] not in self.contract["overrideNamespaces"]:
                    raise TokenError(f"{path}:{name}: primitive overrides are forbidden")
                if token["$type"] != tokens[name]["$type"]:
                    raise TokenError(f"{path}:{name}: type change is forbidden")
                tokens[name] = deepcopy(token)
                provenance[name] = f"{label}:{path.name}"
            self.check_required(tokens)
            values = resolve_values(tokens)
        result = Composition(tokens, values, provenance, mode, self.usage)
        validate_usage(self.usage, result)
        return result


def contrast_value(foreground, background):
    for color in (foreground, background):
        if color.get("alpha", 1) != 1:
            raise TokenError("contrast pairs must be opaque; alpha cannot be ignored")

    def luminance(color):
        linear = [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in color["components"]]
        return sum(c * weight for c, weight in zip(linear, (0.2126, 0.7152, 0.0722)))

    low, high = sorted((luminance(foreground), luminance(background)))
    return (high + 0.05) / (low + 0.05)


def contrast_pairs(composition):
    """Yield foreground/background token paths and required ratios for this mode."""
    names = [p.removeprefix("semantic.color.") for p in composition.values if p.startswith("semantic.color.")]
    pairs = {}
    for (foreground, background, mode), minimum in contrast_rules(names).items():
        if mode != composition.mode:
            continue
        pairs[(f"semantic.color.{foreground}", f"semantic.color.{background}")] = minimum
    for foreground, background, minimum in usage_pairs(composition.usage):
        pairs[(foreground, background)] = max(minimum, pairs.get((foreground, background), 0))
    prefix = "component.button.primary."
    for name in ("background", "hover-background", "pressed-background"):
        key = prefix + "foreground", prefix + name
        pairs[key] = max(4.5, pairs.get(key, 0))
    for surface in ("bg-sunken", "bg-base", "bg-surface", "bg-raised", "bg-hover", "bg-active"):
        key = prefix + "focus-color", "semantic.color." + surface
        pairs[key] = max(3, pairs.get(key, 0))
    for (foreground, background), minimum in pairs.items():
        yield foreground, background, minimum


def check_contrast(composition):
    validate_bindings(composition)
    count = 0
    for foreground, background, minimum in contrast_pairs(composition):
        f, b = composition.values[foreground], composition.values[background]
        for label, first, second in (("", f, b), (" HEX fallback", color_from_hex(hex_color(f)), color_from_hex(hex_color(b)))):
            ratio = contrast_value(first, second)
            if ratio < minimum:
                raise TokenError(f"{foreground} on {background} [{composition.mode}{label}]: "
                                 f"{ratio:.6f}:1 is below {minimum:g}:1")
        count += 1
    check_syntax_distances(composition)
    return count


def validate_project(root):
    project = Project(root)
    base_path = project.source(project.contract.get("base"))
    base = read_json(base_path)
    check_metadata(base, base_path)
    design_seed_values(base["$extensions"][EXTENSION].get("designSeeds"))
    report = ValidationReport()
    colors = {}
    for mode in THEMES:
        composed = project.compose(mode)
        report.tokens = len(composed.tokens)
        report.contrast_checks += check_contrast(composed)
        report.deprecated.extend(f"{mode}:{path}: {token['$deprecated']}" for path, token in composed.tokens.items()
                                 if token.get("$deprecated"))
        for path, value in composed.values.items():
            if path.startswith("semantic.color."):
                colors[(path.removeprefix("semantic.color."), mode)] = hex_color(value)
    try:
        markdown = (Path(root) / "docs/platforms.md").read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise TokenError(f"docs/platforms.md: {error}") from error
    check_platforms(markdown, colors, report)
    if report.errors:
        raise TokenError("\n".join(report.errors))
    return report
