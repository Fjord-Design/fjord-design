"""Validate the pinned Phosphor Fill source assets and licence."""

import hashlib
from pathlib import Path
import re
import xml.etree.ElementTree as ET

from .dtcg import EXTENSION, TokenError, read_json
from .project import Project


NAMES = set("check close info success warning danger add remove search expand next arrow-right settings file copy loading".split())


def load_icons(root):
    project = Project(root)
    base = read_json(project.source(project.contract["base"]))
    if base["$extensions"][EXTENSION].get("iconAssets") != "assets/icons/manifest.json":
        raise TokenError("iconAssets: reference assets/icons/manifest.json")
    directory = Path(root) / "assets/icons"
    manifest = read_json(directory / "manifest.json")
    if (not isinstance(manifest, dict) or type(manifest.get("schemaVersion")) is not int
            or manifest["schemaVersion"] != 1 or manifest.get("family") != "Phosphor"
            or manifest.get("weight") != "fill" or manifest.get("license") != "MIT"):
        raise TokenError("icons: expected a pinned Phosphor Fill manifest with the original MIT licence")
    licence = (directory / "LICENSE").read_bytes()
    if (hashlib.sha256(licence).hexdigest() != manifest.get("licenseSha256")
            or b"Phosphor Icons" not in licence or b"Permission is hereby granted" not in licence):
        raise TokenError("icons: original MIT licence is missing or changed")
    records = manifest.get("icons")
    if (not isinstance(records, list) or any(not isinstance(item, dict) or not isinstance(item.get("name"), str) for item in records)
            or len(records) != len(NAMES) or {item["name"] for item in records} != NAMES):
        raise TokenError("icons: retain each of the 16 approved icon names exactly once")
    for item in records:
        name = item["name"]
        if item.get("file") != name + ".svg":
            raise TokenError(f"icons.{name}: filename must match the approved name")
        path = (directory / item["file"]).resolve()
        if not path.is_relative_to(directory.resolve()):
            raise TokenError(f"icons.{name}: file must stay inside assets/icons")
        data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != item.get("sha256"):
            raise TokenError(f"icons.{name}: SVG hash differs from the pinned source")
        try:
            svg = ET.fromstring(data)
        except ET.ParseError as error:
            raise TokenError(f"icons.{name}: invalid SVG: {error}") from error
        if (svg.tag != "{http://www.w3.org/2000/svg}svg" or svg.get("viewBox") != "0 0 256 256"
                or svg.get("fill") != "currentColor" or set(svg.attrib) != {"viewBox", "fill"}):
            raise TokenError(f"icons.{name}: retain the original 256-unit viewBox and currentColor fill")
        for child in svg:
            if (child.tag != "{http://www.w3.org/2000/svg}path" or len(child)
                    or set(child.attrib) - {"d", "fill-rule", "clip-rule"}):
                raise TokenError(f"icons.{name}: only static filled paths are permitted")
            if not re.fullmatch(r"[MmZzLlHhVvCcSsQqTtAa0-9.,+\s-]+", child.get("d", "")):
                raise TokenError(f"icons.{name}: invalid path data")
        if not len(svg):
            raise TokenError(f"icons.{name}: supply a visible filled path")
    return manifest
