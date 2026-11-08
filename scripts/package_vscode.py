"""Build a deterministic local, theme-only VSIX without installing or publishing it."""

import argparse
from io import BytesIO
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re
import sys
from xml.etree import ElementTree as XML
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

from fjord_tokens.adapters import write_adapter_files
from fjord_tokens.dtcg import TokenError, reject_nonfinite, unique_keys


ROOT = Path(__file__).resolve().parents[1]
VSIX_NAMESPACE = "http://schemas.microsoft.com/developer/vsx-schema/2011"
CONTENT_NAMESPACE = "http://schemas.openxmlformats.org/package/2006/content-types"
SEMVER = r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
EXECUTABLE_FIELDS = {"main", "browser", "activationEvents", "scripts", "dependencies", "devDependencies",
                     "extensionDependencies", "extensionPack", "enabledApiProposals"}


def json_document(text, name):
    try:
        result = json.loads(text, object_pairs_hook=unique_keys, parse_constant=reject_nonfinite)
    except (TypeError, ValueError) as error:
        raise TokenError(f"{name}: invalid generated JSON: {error}") from error
    if not isinstance(result, dict):
        raise TokenError(f"{name}: generated JSON must be an object")
    return result


def text_value(value, field):
    if not isinstance(value, str) or not value.strip():
        raise TokenError(f"package.json: {field} must be a non-empty string")
    if any(not (character in "\t\n\r" or 0x20 <= ord(character) <= 0xD7FF
                or 0xE000 <= ord(character) <= 0xFFFD or 0x10000 <= ord(character) <= 0x10FFFF)
           for character in value):
        raise TokenError(f"package.json: {field} contains an invalid XML character")
    return value


def theme_path(value):
    """Accept only canonical, relative JSON files directly under themes/."""
    if not isinstance(value, str):
        raise TokenError("package.json: each theme path must be a string")
    normalized = value[2:] if value.startswith("./") else value
    path = PurePosixPath(normalized)
    if (path.is_absolute() or ".." in path.parts or "\\" in normalized or path.as_posix() != normalized
            or len(path.parts) != 2 or path.parts[0] != "themes"
            or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*\.json", path.name)):
        raise TokenError(f"package.json: unsafe theme contribution path {value!r}")
    return normalized


def extension_manifest(files):
    for name in ("package.json", "README.md", "LICENSE"):
        if not isinstance(files.get(name), str) or not files[name].strip():
            raise TokenError(f"VSIX: missing generated text file {name}")
    manifest = json_document(files["package.json"], "package.json")
    if manifest.get("name") not in ("fjord-themes", "fjord-themes-work", "fjord-themes-gaming") or manifest.get("publisher") != "fjord":
        raise TokenError("package.json: expected local extension identity fjord.fjord-themes")
    version = manifest.get("version")
    if not isinstance(version, str) or not re.fullmatch(SEMVER, version):
        raise TokenError("package.json: version must be a stable semantic version such as 0.1.0")
    text_value(manifest.get("displayName"), "displayName")
    text_value(manifest.get("description"), "description")
    engines = manifest.get("engines")
    if (not isinstance(engines, dict) or not isinstance(engines.get("vscode"), str)
            or not re.fullmatch(r"\^" + SEMVER, engines["vscode"])):
        raise TokenError("package.json: engines.vscode must declare a stable caret range such as ^1.100.0")
    if EXECUTABLE_FIELDS & manifest.keys():
        raise TokenError("package.json: theme-only VSIX cannot declare code, dependencies or activation hooks")
    contributes = manifest.get("contributes")
    if not isinstance(contributes, dict) or set(contributes) != {"themes"}:
        raise TokenError("package.json: theme-only VSIX must contribute only themes")
    themes = contributes["themes"]
    if not isinstance(themes, list) or len(themes) != 3:
        raise TokenError("package.json: contribute exactly the three Fjord modes")
    paths, labels = set(), set()
    for index, theme in enumerate(themes):
        if not isinstance(theme, dict):
            raise TokenError(f"package.json: theme contribution {index} must be an object")
        label = text_value(theme.get("label"), f"contributes.themes[{index}].label")
        if theme.get("uiTheme") not in ("vs", "vs-dark"):
            raise TokenError(f"package.json: theme {label!r} must declare vs or vs-dark")
        path = theme_path(theme.get("path"))
        if path in paths or label in labels:
            raise TokenError("package.json: theme paths and labels must be unique")
        paths.add(path)
        labels.add(label)
        if not isinstance(files.get(path), str):
            raise TokenError(f"VSIX: missing generated theme {path}")
        json_document(files[path], path)
    expected = {f"themes/fjord-{mode}-color-theme.json" for mode in ("night", "day", "aurora")}
    if paths != expected:
        raise TokenError("package.json: theme paths must identify Fjord Night, Day and Aurora")
    return manifest, sorted(paths)


def xml_bytes(element):
    XML.indent(element, space="  ")
    return XML.tostring(element, encoding="utf-8", xml_declaration=True) + b"\n"


def vsix_manifest(manifest):
    """Emit the package protocol used by Microsoft's vsce 4.0.0.

    See https://github.com/microsoft/vscode-vsce/blob/v4.0.0/src/package.ts#L1418-L1543.
    Identity.Id is the unqualified extension name; Publisher is separate.
    """
    package = XML.Element("PackageManifest", {"Version": "2.0.0", "xmlns": VSIX_NAMESPACE})
    metadata = XML.SubElement(package, "Metadata")
    XML.SubElement(metadata, "Identity", {"Language": "en-US", "Id": manifest["name"],
                                         "Version": manifest["version"], "Publisher": manifest["publisher"]})
    XML.SubElement(metadata, "DisplayName").text = manifest["displayName"]
    XML.SubElement(metadata, "Description", {"xml:space": "preserve"}).text = manifest["description"]
    XML.SubElement(metadata, "Tags").text = "theme,Fjord"
    XML.SubElement(metadata, "Categories").text = "Themes"
    XML.SubElement(metadata, "GalleryFlags")
    properties = XML.SubElement(metadata, "Properties")
    XML.SubElement(properties, "Property", {"Id": "Microsoft.VisualStudio.Code.Engine",
                                             "Value": manifest["engines"]["vscode"]})
    XML.SubElement(properties, "Property", {"Id": "Microsoft.VisualStudio.Code.ExecutesCode", "Value": "false"})
    XML.SubElement(metadata, "License").text = "extension/LICENSE"
    installation = XML.SubElement(package, "Installation")
    XML.SubElement(installation, "InstallationTarget", {"Id": "Microsoft.VisualStudio.Code"})
    XML.SubElement(package, "Dependencies")
    assets = XML.SubElement(package, "Assets")
    for kind, path in (("Microsoft.VisualStudio.Code.Manifest", "extension/package.json"),
                       ("Microsoft.VisualStudio.Services.Content.Details", "extension/README.md"),
                       ("Microsoft.VisualStudio.Services.Content.License", "extension/LICENSE")):
        XML.SubElement(assets, "Asset", {"Type": kind, "Path": path, "Addressable": "true"})
    return xml_bytes(package)


def content_types():
    # Use OPC extension names without the leading dot, as documented for VSIX:
    # https://learn.microsoft.com/visualstudio/extensibility/the-structure-of-the-content-types-dot-xml-file
    element = XML.Element("Types", {"xmlns": CONTENT_NAMESPACE})
    for extension, kind in (("json", "application/json"), ("md", "text/markdown"), ("vsixmanifest", "text/xml")):
        XML.SubElement(element, "Default", {"Extension": extension, "ContentType": kind})
    XML.SubElement(element, "Override", {"PartName": "/extension/LICENSE", "ContentType": "text/plain"})
    return xml_bytes(element)


def package_files(files):
    """Package only declared theme data and documentation, excluding the report."""
    manifest, themes = extension_manifest(files)
    contents = {"extension/" + name: files[name].encode("utf-8")
                for name in ("package.json", "README.md", "LICENSE", *themes)}
    contents["extension.vsixmanifest"] = vsix_manifest(manifest)
    contents["[Content_Types].xml"] = content_types()
    result = BytesIO()
    with ZipFile(result, "w", compression=ZIP_DEFLATED, compresslevel=9) as archive:
        for name, content in sorted(contents.items()):
            info = ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.compress_type = ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content, compresslevel=9)
    return result.getvalue()


def bundle(root=ROOT):
    """Freshly validate/generate the adapter in memory, then return VSIX bytes."""
    root = Path(root).resolve()
    location = root / "adapters/vscode/generate.py"
    spec = importlib.util.spec_from_file_location("fjord_vscode_package_generator", location)
    generator = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(generator)
    return package_files(generator.build_bundle(root))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("--root", type=Path, default=ROOT, help="Fjord source root")
    parser.add_argument("--output", type=Path, help="destination; default: dist/release/fjord-themes-<version>.vsix")
    args = parser.parse_args(argv)
    try:
        content = bundle(args.root)
        with ZipFile(BytesIO(content)) as archive:
            version = json.loads(archive.read("extension/package.json"))["version"]
        destination = args.output or args.root / "dist/release" / f"fjord-themes-{version}.vsix"
        write_adapter_files(args.root, destination.parent, {destination.name: content})
        print(f"Prepared local VSIX: {destination}")
    except (TokenError, OSError, UnicodeError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
