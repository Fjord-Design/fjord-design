"""Validate declarative platform mappings without importing platform runtimes."""

from copy import deepcopy
from pathlib import Path
import re

from .contract import THEMES
from .dtcg import TokenError, hex_color, read_json
from .project import Project, check_contrast, metadata, validate_project


FIELDS = {"schemaVersion", "id", "target", "modes", "mapping", "requiredRoles", "unsupported", "fallbacks"}
FALLBACKS = {"colorSpace", "fonts", "radius", "shadows", "motion", "interaction"}
PROTECTED = ("tokens", "src", "docs", "assets", "examples", "tests", "reference", "scripts", "patches",
             "adapters", "licenses", ".github", ".git")


def nonempty(value):
    return isinstance(value, str) and bool(value.strip())


def load_adapter(root, manifest_path):
    """Read a schema-1 manifest; target keys are opaque, flat platform names."""
    path = Path(manifest_path)
    if not path.is_absolute():
        path = Path(root) / path
    document = read_json(path)

    def fail(message):
        raise TokenError(f"{path}: {message}")

    if not isinstance(document, dict) or set(document) != FIELDS:
        fail("declare exactly schemaVersion, id, target, modes, mapping, requiredRoles, unsupported and fallbacks")
    if type(document["schemaVersion"]) is not int or document["schemaVersion"] != 1:
        fail("unsupported adapter schemaVersion; expected 1")
    if not nonempty(document["id"]) or not re.fullmatch(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*", document["id"]):
        fail("id must use lower-case hyphenated words")
    target = document["target"]
    if (not isinstance(target, dict) or set(target) != {"application", "version", "os"}
            or not all(nonempty(value) for value in target.values())):
        fail("target must declare non-empty application, version and os strings")
    modes = document["modes"]
    if (not isinstance(modes, list) or not modes or not all(isinstance(mode, str) for mode in modes)
            or len(modes) != len(set(modes)) or any(mode not in THEMES for mode in modes)):
        fail("modes must list unique supported modes: night, day, aurora")
    mapping = document["mapping"]
    if (not isinstance(mapping, dict) or not mapping
            or not all(nonempty(key) and nonempty(value) and value.startswith("semantic.")
                       for key, value in mapping.items())):
        fail("mapping must map non-empty target keys to semantic token paths")
    required = document["requiredRoles"]
    if (not isinstance(required, list) or not required or not all(nonempty(role) for role in required)
            or len(required) != len(set(required))):
        fail("requiredRoles must list unique semantic token paths")
    unsupported = document["unsupported"]
    if (not isinstance(unsupported, dict)
            or not all(nonempty(role) and role.startswith("semantic.") and nonempty(reason)
                       for role, reason in unsupported.items())):
        fail("unsupported must map semantic token paths to explanations")
    overlap = set(mapping.values()) & unsupported.keys()
    if overlap:
        fail(f"roles cannot be both mapped and unsupported: {sorted(overlap)}")
    missing = set(required) - set(mapping.values())
    if missing:
        fail(f"missing required mappings: {sorted(missing)}")
    fallbacks = document["fallbacks"]
    if (not isinstance(fallbacks, dict) or set(fallbacks) != FALLBACKS
            or not all(nonempty(value) for value in fallbacks.values())):
        fail("fallbacks must explain colorSpace, fonts, radius, shadows, motion and interaction")
    return document


def map_adapter(root, manifest_path, theme=None, overrides=()):
    """Return all validated mode mappings and a portable coverage report.

    Colour values are emitted as sRGB HEX. Other semantic values retain their
    resolved DTCG representation. No files or user settings are written.
    """
    manifest = load_adapter(root, manifest_path)
    validate_project(root)
    project = Project(root)
    result = {}
    report = {"schemaVersion": 1, "adapter": manifest["id"], "target": manifest["target"],
              "contractVersion": metadata()["contractVersion"], "fallbacks": manifest["fallbacks"], "modes": {}}
    sources = set(manifest["mapping"].values()) | set(manifest["unsupported"])
    for mode in manifest["modes"]:
        composition = project.compose(mode, theme, overrides)
        check_contrast(composition)
        unknown = sources - composition.tokens.keys()
        if unknown:
            raise TokenError(f"adapter {manifest['id']} [{mode}]: unknown semantic roles {sorted(unknown)}")
        values = {}
        mapped = {}
        for target, role in sorted(manifest["mapping"].items()):
            token = composition.tokens[role]
            value = composition.values[role]
            value = hex_color(value) if token["$type"] == "color" else deepcopy(value)
            values[target] = value
            mapped[target] = {"role": role, "type": token["$type"], "value": value,
                              "source": composition.provenance[role], "original": deepcopy(token["$value"])}
        colour_roles = {role for role in composition.tokens if role.startswith("semantic.color.")}
        report["modes"][mode] = {
            "mapped": mapped,
            "unsupported": dict(sorted(manifest["unsupported"].items())),
            "unmapped": sorted(colour_roles - sources),
        }
        result[mode] = values
    return result, report


def write_adapter_files(root, output_dir, files, inputs=()):
    """Write a prevalidated bundle, refusing source paths and unsafe filenames.

    All destinations are checked before the first write. Each individual file
    is replaced atomically by the shared CLI writer; the bundle is not a
    transaction across files if the filesystem fails during writing.
    """
    from .cli import write_binary_output

    root = Path(root).resolve()
    output = Path(output_dir).resolve()
    protected = [root / name for name in PROTECTED]
    protected.extend(path for path in root.iterdir() if path.is_file())
    protected.extend(Path(path).resolve() for path in inputs)
    pending = []
    for name, content in files.items():
        relative = Path(name)
        if relative.is_absolute() or ".." in relative.parts or not relative.parts or relative == Path("."):
            raise TokenError(f"unsafe adapter output name {name!r}")
        destination = (output / relative).resolve()
        if not destination.is_relative_to(output):
            raise TokenError(f"adapter output {name!r} escapes its destination")
        if any(destination == path.resolve() or path.is_dir() and destination.is_relative_to(path.resolve())
               for path in protected):
            raise TokenError(f"{destination}: adapter output must not replace project sources or inputs")
        if not isinstance(content, (bytes, str)):
            raise TokenError(f"{name}: adapter output content must be text or bytes")
        pending.append((destination, content.encode("utf-8") if isinstance(content, str) else content))
    for destination, content in pending:
        write_binary_output(destination, content, protected)
