"""Validate pinned self-hosted font assets and derive typography CSS from tokens."""

import hashlib
import json
from pathlib import Path
import re
import struct

from .dtcg import EXTENSION, TokenError, number, read_json
from .project import Project


ROLES = ("type-display type-title-1 type-title-2 type-title-3 type-title-4 type-lead type-body "
         "type-body-strong type-label type-small type-caption type-overline type-code type-code-small type-code-italic").split()
FAMILIES = {"Noto Sans", "Raleway", "Cascadia Code", "Open Sans", "Fira Code"}
GENERIC = {"system-ui", "sans-serif", "serif", "monospace", "ui-monospace", "-apple-system"}


def asset_path(directory, name):
    if not isinstance(name, str):
        raise TokenError("font asset paths must be strings")
    result = (directory / name).resolve()
    if not result.is_relative_to(directory.resolve()):
        raise TokenError(f"font asset {name!r} must stay inside assets/fonts")
    return result


def check_file(directory, name, digest):
    path = asset_path(directory, name)
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != digest:
        raise TokenError(f"{path}: font asset hash changed; inspect and update the pinned evidence")
    return data


def load_fonts(root):
    project = Project(root)
    base = read_json(project.source(project.contract.get("base")))
    if base["$extensions"][EXTENSION].get("fontAssets") != "assets/fonts/manifest.json":
        raise TokenError("fontAssets: reference assets/fonts/manifest.json")
    directory = Path(root) / "assets/fonts"
    manifest = read_json(directory / "manifest.json")
    inspection = read_json(directory / "inspection.json")
    if manifest.get("schemaVersion") != 1 or manifest.get("fontDisplay") != "swap":
        raise TokenError("font manifest: expected schemaVersion 1 and fontDisplay swap")
    families = manifest.get("families")
    if (not isinstance(families, list) or any(not isinstance(f, dict) or not isinstance(f.get("family"), str) for f in families)
            or {family.get("family") for family in families} != FAMILIES or len(families) != 5):
        raise TokenError("font manifest: retain the five approved font families exactly once")
    inspected_faces = inspection.get("faces")
    if (not isinstance(inspected_faces, list) or any(not isinstance(face, dict) or not isinstance(face.get("file"), str)
                                                    for face in inspected_faces)
            or not isinstance(inspection.get("probe"), str)
            or not set("æøåÆØÅéÉ0123456789Il1O0o.,;:!?[]{}()") <= set(inspection["probe"])):
        raise TokenError("font inspection: retain the Danish, numeric, punctuation and confusable glyph probe")
    evidence = {face["file"]: face for face in inspected_faces}
    if len(evidence) != len(inspected_faces):
        raise TokenError("font inspection: duplicate face evidence")
    seen = set()
    for family in families:
        if (family.get("license") != "OFL-1.1" or not isinstance(family.get("licenseFile"), str)
                or not isinstance(family.get("licenseSha256"), str)):
            raise TokenError(f"{family['family']}: document the original OFL-1.1 licence")
        license_text = check_file(directory, family["licenseFile"], family["licenseSha256"]).decode("utf-8")
        if "SIL OPEN FONT LICENSE" not in license_text or "1.1" not in license_text:
            raise TokenError(f"{family['family']}: original font licence is missing")
        styles = set()
        faces = family.get("faces")
        if not isinstance(faces, list) or any(not isinstance(face, dict) or not {"file", "style", "weight", "sha256"} <= face.keys() for face in faces):
            raise TokenError(f"{family['family']}: define font file, style, weight and SHA256 for every face")
        for face in faces:
            name, style, weights = face["file"], face["style"], face["weight"]
            if not isinstance(name, str) or not re.fullmatch(r"[a-z0-9-]+/[A-Za-z0-9-]+\.woff2", name):
                raise TokenError(f"{name}: use a family directory and a WOFF2 filename")
            if name in seen or style not in {"normal", "italic"} or style in styles:
                raise TokenError(f"{name}: duplicate font face or unsupported style")
            seen.add(name)
            styles.add(style)
            if (not isinstance(weights, list) or len(weights) != 2
                    or any(type(v) is not int for v in weights) or not 1 <= weights[0] <= weights[1] <= 1000):
                raise TokenError(f"{name}: define an ordered CSS weight range from 1 to 1000")
            data = check_file(directory, name, face["sha256"])
            if len(data) < 48 or data[:4] != b"wOF2" or struct.unpack_from(">I", data, 8)[0] != len(data):
                raise TokenError(f"{name}: invalid WOFF2 signature or declared size")
            if face.get("bytes") != len(data):
                raise TokenError(f"{name}: recorded byte size differs from the font file")
            measured = evidence.get(name)
            if (not measured or measured.get("sha256") != face["sha256"]
                    or measured.get("family") != family["family"] or measured.get("style") != style
                    or measured.get("weight") != weights or measured.get("variable") is not True
                    or measured.get("missingProbeCodepoints") != []):
                raise TokenError(f"{name}: family/style/weight/glyph inspection is missing or stale")
        required = {"normal"} if family["family"] == "Fira Code" else {"normal", "italic"}
        if styles != required:
            raise TokenError(f"{family['family']}: expected styles {sorted(required)}")
    if seen != evidence.keys():
        raise TokenError("font inspection: face list differs from the font manifest")
    shared = Project(root).compose("night")
    for mode in ("night", "day", "aurora"):
        composition = project.compose(mode)
        for role in ROLES:
            path = "semantic.typography." + role
            if path not in composition.values:
                raise TokenError(f"{path}: missing typography role")
            value = composition.values[path]
            style = composition.tokens[path].get("$extensions", {}).get(EXTENSION, {}).get("fontStyle")
            if value != shared.values[path] or style != shared.tokens[path].get("$extensions", {}).get(EXTENSION, {}).get("fontStyle"):
                raise TokenError(f"{path}: share typography values and styles across all modes")
            matches = [face for family in families if family["family"] == value["fontFamily"][0]
                       for face in family["faces"] if face["style"] == style]
            if (not number(value["fontWeight"]) or not matches
                    or not any(face["weight"][0] <= value["fontWeight"] <= face["weight"][1] for face in matches)):
                raise TokenError(f"{path}: no bundled primary face supports its style and weight")
            if value["fontSize"]["value"] <= 0 or not number(value["lineHeight"]) or value["lineHeight"] <= 0:
                raise TokenError(f"{path}: font size and line height must be positive")
    return manifest


def family_css(families):
    return ", ".join(family if family in GENERIC else json.dumps(family, ensure_ascii=False) for family in families)


def rem(value):
    amount = value["value"] / 16 if value["unit"] == "px" else value["value"]
    return f"{amount:g}rem"


def typography_css(root, manifest):
    composition = Project(root).compose("night")
    result = ["/* Generated from Fjord tokens and the pinned font manifest. */"]
    for family in manifest["families"]:
        for face in family["faces"]:
            result.append("@font-face {" + f"font-family:{json.dumps(family['family'])};"
                          + f"src:url(\"./{face['file']}\") format(\"woff2\");font-style:{face['style']};"
                          + f"font-weight:{face['weight'][0]} {face['weight'][1]};font-display:swap;"
                          + (f"unicode-range:{face['unicodeRange']};" if "unicodeRange" in face else "") + "}")
    stacks = {name: composition.values["semantic.font." + name] for name in ("sans", "display", "mono")}
    result.append(":root {" + "".join(f"--fjord-font-{name}:{family_css(stack)};" for name, stack in stacks.items())
                  + "--fjord-code-ligatures:1;--fjord-code-alternates:normal;}")
    result.extend([
        '@font-feature-values "Cascadia Code" { @styleset { fjord-cursive:1; } }',
        ':root[data-fjord-ligatures="off"] {--fjord-code-ligatures:0;}',
        ':root[data-fjord-cursive="on"] {--fjord-code-alternates:styleset(fjord-cursive);}',
        ':root[data-fjord-fonts="fallback"] {--fjord-font-sans:"Open Sans",system-ui,sans-serif;'
        '--fjord-font-display:"Noto Sans","Open Sans",system-ui,sans-serif;--fjord-font-mono:"Fira Code",ui-monospace,monospace;}',
        ':root[data-fjord-fonts="system"] {--fjord-font-sans:system-ui,sans-serif;'
        '--fjord-font-display:system-ui,sans-serif;--fjord-font-mono:ui-monospace,Menlo,Consolas,monospace;}',
    ])
    for role in ROLES:
        path = "semantic.typography." + role
        value = composition.values[path]
        style = composition.tokens[path]["$extensions"][EXTENSION]["fontStyle"]
        stack = next((name for name, families in stacks.items() if families == value["fontFamily"]), None)
        if stack is None:
            raise TokenError(f"{path}: bind fontFamily to one of the shared semantic font stacks")
        result.append(f".fjord-{role} {{font-family:var(--fjord-font-{stack});font-size:{rem(value['fontSize'])};"
                      + f"font-weight:{value['fontWeight']};font-style:{style};line-height:{value['lineHeight']};"
                      + f"letter-spacing:{rem(value['letterSpacing'])};font-synthesis:none;"
                      + ('font-feature-settings:"calt" var(--fjord-code-ligatures),"liga" var(--fjord-code-ligatures);'
                         if stack == "mono" else "")
                      + ('font-variant-alternates:var(--fjord-code-alternates);' if style == "italic" and stack == "mono" else "") + "}")
    result.append('[data-fjord-ligatures="off"] .fjord-code {font-variant-ligatures:none;}')
    return "\n".join(result) + "\n"


def font_bundle(root, manifest):
    directory = Path(root) / "assets/fonts"
    files = {"typography.css": typography_css(root, manifest).encode("utf-8")}
    for name in ("manifest.json", "inspection.json"):
        files[name] = (directory / name).read_bytes()
    for family in manifest["families"]:
        files[family["licenseFile"]] = asset_path(directory, family["licenseFile"]).read_bytes()
        for face in family["faces"]:
            files[face["file"]] = asset_path(directory, face["file"]).read_bytes()
    return files
