"""Behavioral coverage for DTCG references, group inheritance and portable output."""

from copy import deepcopy
import json
from pathlib import Path
import shutil
import subprocess
import sys

import pytest

from fjord.contract import MODES
from fjord.dtcg import EXTENSION, Resolver, TokenError, color_from_hex, flatten, nest, read_json
from fjord.export import export
from fjord.project import Project, metadata


ROOT = Path(__file__).resolve().parents[1]
VENDOR = "com.example.design-tool"


def token(kind, value, **properties):
    definition = {"$value": value, "$description": "Conformance fixture."}
    if kind is not None:
        definition["$type"] = kind
    return dict(definition, **properties)


def dimension(value):
    return {"value": value, "unit": "px"}


def shadow():
    return {"color": color_from_hex("#00000080"), "offsetX": dimension(0),
            "offsetY": dimension(1), "blur": dimension(2), "spread": dimension(0), "inset": True}


def write_document(path, document):
    path.write_text(json.dumps(document), encoding="utf-8")
    return path


def write_layer(root, name, tokens):
    return write_document(root / name, nest(tokens, metadata()))


@pytest.fixture
def root(tmp_path):
    shutil.copytree(ROOT / "tokens", tmp_path / "tokens")
    return tmp_path


def test_local_pointers_resolve_whole_values_composite_fields_and_array_items():
    blue = color_from_hex("#336699")
    document = {
        "primitive": {
            "blue": token("color", blue),
            "gap": token("dimension", dimension(8)),
            "curve": token("cubicBezier", [0.2, 0, 0.8, 1]),
        },
        "semantic": {
            "blue": token("color", {"$ref": "#/primitive/blue/$value"}),
            "red-component": token("number", {"$ref": "#/primitive/blue/$value/components/0"}),
            "curve-end": token("number", {"$ref": "#/primitive/curve/$value/3"}),
            "gap": token("dimension", {
                "value": {"$ref": "#/primitive/gap/$value/value"},
                "unit": {"$ref": "#/primitive/gap/$value/unit"}}),
            "tint": token("color", {
                "colorSpace": "srgb", "components": [
                    {"$ref": "#/primitive/blue/$value/components/0"},
                    {"$ref": "#/primitive/blue/$value/components/1"},
                    {"$ref": "#/primitive/blue/$value/components/2"}],
                "alpha": {"$ref": "#/primitive/blue/$value/alpha"}}),
        },
    }
    values = Resolver(flatten(document)).all()
    assert values["semantic.blue"] == blue
    assert values["semantic.red-component"] == blue["components"][0]
    assert values["semantic.curve-end"] == 1
    assert values["semantic.gap"] == dimension(8)
    assert values["semantic.tint"] == blue


def test_property_pointer_cannot_invent_fields_inside_a_raw_alias_string():
    document = {
        "primitive": {"gap": token("dimension", dimension(8)),
                      "copy": token("dimension", "{primitive.gap}")},
        "semantic": {"size": token("number", {"$ref": "#/primitive/copy/$value/value"})},
    }
    with pytest.raises(TokenError):
        Resolver(flatten(document)).all()


@pytest.mark.parametrize(("pointer", "expected"), [
    pytest.param("#/$extensions/com.example.design-tool/slash~1key", 3, id="escaped-slash"),
    pytest.param("#/$extensions/com.example.design-tool/tilde~0key", 4, id="escaped-tilde"),
    pytest.param("#/$extensions/com.example.design-tool/~01", 5, id="decode-tilde-once"),
    pytest.param("#/$extensions/com.example.design-tool/space%20key", 6, id="uri-fragment"),
    pytest.param("#/$extensions/com.example.design-tool/%C3%A5", 7, id="utf8-fragment"),
    pytest.param("#/$extensions/com.example.design-tool/items/1", 8, id="metadata-array"),
    pytest.param("#/$extensions/com.example.design-tool/01", 9, id="numeric-object-key"),
])
def test_pointer_escaping_and_document_metadata_targets(pointer, expected):
    document = {
        "$extensions": {VENDOR: {"slash/key": 3, "tilde~key": 4, "~1": 5,
                                  "space key": 6, "å": 7, "items": [0, 8], "01": 9}},
        "semantic": {"count": token("number", {"$ref": pointer})},
    }
    assert Resolver(flatten(document)).all()["semantic.count"] == expected


@pytest.mark.parametrize("pointer", [
    pytest.param("#/primitive/absent/$value", id="missing-token"),
    pytest.param("#/primitive.count/$value", id="dotted-segment-is-not-a-token-path"),
    pytest.param("#/primitive/blue/$value/absent", id="missing-property"),
    pytest.param("#/primitive/blue/$value/components/3", id="out-of-range-index"),
    pytest.param("#/primitive/blue/$value/components/01", id="leading-zero-index"),
    pytest.param("#/primitive/blue/$value/components/-1", id="negative-index"),
    pytest.param("#/primitive/blue/$value/components/-", id="append-index"),
    pytest.param("#/primitive/blue/$value/alpha/child", id="traversal-through-scalar"),
    pytest.param("#primitive/blue/$value", id="missing-slash"),
    pytest.param("#/primitive/blue/~2", id="invalid-escape"),
    pytest.param("#/primitive/blue/~", id="incomplete-escape"),
    pytest.param("#/primitive/blue/%ZZ", id="invalid-percent-escape"),
    pytest.param("#/primitive/blue/%FF", id="invalid-utf8-escape"),
    pytest.param("other.tokens.json#/primitive/blue/$value", id="external-file"),
    pytest.param("https://example.invalid/tokens.json#/blue/$value", id="external-url"),
    pytest.param(None, id="null-reference"),
    pytest.param(1, id="numeric-reference"),
    pytest.param([], id="array-reference"),
])
def test_invalid_or_unresolvable_pointers_raise_token_errors(pointer):
    document = {
        "primitive": {"blue": token("color", color_from_hex("#336699")), "count": token("number", 4)},
        "semantic": {"count": token("number", {"$ref": pointer})},
    }
    with pytest.raises(TokenError):
        Resolver(flatten(document)).all()


@pytest.mark.parametrize("value", [
    pytest.param("{primitive.count}", id="brace"),
    pytest.param({"$ref": "#/primitive/count/$value"}, id="json-pointer"),
])
def test_explicit_alias_type_must_match_even_when_literal_shapes_overlap(value):
    document = {"primitive": {"count": token("number", 400)},
                "semantic": {"weight": token("fontWeight", value)}}
    with pytest.raises(TokenError):
        Resolver(flatten(document)).all()


@pytest.mark.parametrize("document", [
    pytest.param({"semantic": {
        "first": token("dimension", {"$ref": "#/semantic/second/$value"}),
        "second": token("dimension", "{semantic.first}")}}, id="mixed-alias-cycle"),
    pytest.param({"semantic": {
        "first": token("number", {"$ref": "#/semantic/first/$value"})}}, id="self-pointer"),
    pytest.param({"semantic": {
        "count": token("number", {"$ref": "#/semantic/gap/$value/value"}),
        "gap": token("dimension", {"value": {"$ref": "#/semantic/count/$value"}, "unit": "px"})}},
        id="property-pointer-cycle"),
])
def test_pointer_and_brace_cycles_raise_token_errors(document):
    with pytest.raises(TokenError):
        Resolver(flatten(document)).all()


def test_untyped_alias_chains_derive_types_before_group_defaults():
    document = {
        "primitive": {"gap": token("dimension", dimension(8))},
        "semantic": {"$type": "number",
                     "first": token(None, "{primitive.gap}"),
                     "second": token(None, {"$ref": "#/semantic/first/$value"}),
                     "literal": token(None, 2),
                     "nested": {"$type": "dimension", "gap": token(None, dimension(4)),
                                "count": token("number", 3)}},
        "component": {"gap": token(None, "{semantic.second}")},
    }
    tokens = flatten(document)
    values = Resolver(tokens).all()
    for path in ("semantic.first", "semantic.second", "component.gap", "semantic.nested.gap"):
        assert tokens[path]["$type"] == "dimension"
    assert values["component.gap"] == dimension(8)
    assert tokens["semantic.literal"]["$type"] == "number"
    assert tokens["semantic.nested.count"]["$type"] == "number"


@pytest.mark.parametrize("value", [1, dimension(8), color_from_hex("#336699")])
def test_untyped_literals_are_not_guessed(value):
    with pytest.raises(TokenError):
        Resolver(flatten({"semantic": {"untyped": token(None, value)}})).all()


@pytest.mark.parametrize("reference", [
    pytest.param("{primitive.scale.$root}", id="brace"),
    pytest.param({"$ref": "#/primitive/scale/$root/$value"}, id="json-pointer"),
])
def test_group_root_is_a_distinct_referencable_token(reference):
    document = {
        "primitive": {"scale": {"$type": "number", "$root": token(None, 1), "large": token(None, 2)}},
        "semantic": {"default": token(None, reference)},
    }
    tokens = flatten(document)
    assert set(tokens) == {"primitive.scale.$root", "primitive.scale.large", "semantic.default"}
    assert Resolver(tokens).all()["semantic.default"] == 1
    assert nest(tokens)["primitive"]["scale"]["$root"]["$value"] == 1


def test_brace_alias_to_group_does_not_implicitly_select_root():
    document = {"primitive": {"scale": {"$root": token("number", 1)}},
                "semantic": {"default": token("number", "{primitive.scale}")}}
    with pytest.raises(TokenError):
        Resolver(flatten(document)).all()


@pytest.mark.parametrize(("keyword", "extension"), [
    pytest.param("$extends", "{primitive.template}", id="extends-brace"),
    pytest.param("$extends", "#/primitive/template", id="extends-pointer"),
    pytest.param("$ref", "#/primitive/template", id="group-ref"),
])
def test_extends_merges_nested_groups_and_replaces_complete_tokens(keyword, extension):
    old = token(None, 2, **{"$deprecated": True, "$extensions": {VENDOR: {"old": True}}})
    document = {
        "primitive": {
            "template": {"$type": "number", "$deprecated": "Use the newer scale.",
                         "$extensions": {VENDOR: {"editor": [1, None, True]}},
                         "$root": token(None, 0),
                         "nested": {"keep": token(None, 1), "replace": old}},
            "derived": {keyword: extension, "nested": {
                "replace": token(None, 3, **{"$deprecated": False}), "add": token(None, 4)}},
            "final": {"$extends": "{primitive.derived}", "nested": {"add": token(None, 5)}},
        },
    }
    tokens = flatten(document)
    values = Resolver(tokens).all()
    assert values["primitive.derived.$root"] == 0
    assert values["primitive.derived.nested.keep"] == 1
    assert values["primitive.derived.nested.replace"] == 3
    assert values["primitive.final.nested.add"] == 5
    assert tokens["primitive.derived.nested.keep"]["$type"] == "number"
    assert tokens["primitive.derived.nested.keep"]["$deprecated"] == "Use the newer scale."
    assert tokens["primitive.derived.nested.replace"]["$deprecated"] is False
    assert "$extensions" not in tokens["primitive.derived.nested.replace"]
    inherited = nest(tokens)["primitive"]["derived"]
    assert inherited["$extensions"] == document["primitive"]["template"]["$extensions"]


@pytest.mark.parametrize("extension", [
    pytest.param("{primitive.missing}", id="missing-brace-target"),
    pytest.param("#/primitive/missing", id="missing-pointer-target"),
    pytest.param("{primitive.gap}", id="token-brace-target"),
    pytest.param("#/primitive/gap", id="token-pointer-target"),
    pytest.param("#/primitive/gap/$value", id="dimension-object-target"),
    pytest.param("#/primitive/blue/$value", id="color-object-target"),
    pytest.param("#/$extensions/com.example.design-tool", id="vendor-metadata-target"),
    pytest.param("{primitive.gap.$value}", id="malformed-brace-target"),
    pytest.param("other.tokens.json#/group", id="external-target"),
])
def test_extends_rejects_missing_non_group_and_external_targets(extension):
    document = {
        "$extensions": {VENDOR: {"editor": {}}},
        "primitive": {"gap": token("dimension", dimension(8)),
                      "blue": token("color", color_from_hex("#336699")),
                      "derived": {"$extends": extension}},
    }
    with pytest.raises(TokenError):
        Resolver(flatten(document)).all()


@pytest.mark.parametrize("groups", [
    pytest.param({"first": {"$extends": "{primitive.first}"}}, id="self-extension"),
    pytest.param({"first": {"$extends": "{primitive.second}"},
                  "second": {"$extends": "#/primitive/third"},
                  "third": {"$extends": "{primitive.first}"}}, id="mixed-chain"),
    pytest.param({"first": {"nested": {"$extends": "{primitive.first}"}}}, id="ancestor-extension"),
])
def test_extends_cycles_raise_token_errors(groups):
    with pytest.raises(TokenError):
        Resolver(flatten({"primitive": groups})).all()


def test_overlay_context_supplies_extends_targets_and_alias_types_without_leaking_tokens():
    context = {"primitive": {"template": {"$type": "number", "count": token(None, 2)},
                             "gap": token("dimension", dimension(8))}}
    overlay = {"semantic": {"inherited": {"$extends": "{primitive.template}"},
                             "gap": token(None, "{primitive.gap}")}}
    tokens = flatten(overlay, context=context)
    assert set(tokens) == {"semantic.inherited.count", "semantic.gap"}
    assert tokens["semantic.inherited.count"]["$value"] == 2
    assert tokens["semantic.inherited.count"]["$type"] == "number"
    assert tokens["semantic.gap"]["$type"] == "dimension"


def test_nesting_preserves_root_group_empty_group_and_token_extensions():
    document = {
        "$description": "A portable document.",
        "$extensions": {EXTENSION: dict(metadata(), name="Fjord", asset="manifest.json"),
                        VENDOR: {"root": [None, True, {"nested": "data"}]}},
        "semantic": {"$description": "A semantic group.", "$extensions": {VENDOR: {"group": 1}},
                     "empty": {"$extensions": {VENDOR: {"keep": True}}},
                     "count": token("number", 2, **{"$extensions": {VENDOR: {"token": 3}}})},
    }
    original = deepcopy(document)
    tokens = flatten(document)
    Resolver(tokens).all()
    output = nest(tokens, dict(metadata(), mode="day", colorSpace="srgb"))
    assert document == original
    assert output["$description"] == document["$description"]
    assert output["$extensions"][VENDOR] == document["$extensions"][VENDOR]
    assert output["$extensions"][EXTENSION] == dict(document["$extensions"][EXTENSION], mode="day", colorSpace="srgb")
    assert output["semantic"]["$extensions"] == document["semantic"]["$extensions"]
    assert output["semantic"]["empty"] == document["semantic"]["empty"]
    assert output["semantic"]["count"]["$extensions"] == document["semantic"]["count"]["$extensions"]


@pytest.mark.parametrize("reference", [
    pytest.param("{primitive.single}", id="brace"),
    pytest.param({"$ref": "#/primitive/single/$value"}, id="json-pointer"),
])
def test_shadow_array_element_aliases_resolve_single_objects(reference):
    document = {"primitive": {"single": token("shadow", shadow())},
                "semantic": {"layers": token("shadow", [reference, shadow()])}}
    assert Resolver(flatten(document)).all()["semantic.layers"] == [shadow(), shadow()]


@pytest.mark.parametrize("reference", [
    pytest.param("{primitive.layers}", id="brace"),
    pytest.param({"$ref": "#/primitive/layers/$value"}, id="json-pointer"),
])
def test_shadow_array_element_aliases_cannot_expand_arrays(reference):
    document = {"primitive": {"layers": token("shadow", [shadow(), shadow()])},
                "semantic": {"layers": token("shadow", [reference])}}
    with pytest.raises(TokenError):
        Resolver(flatten(document)).all()


@pytest.mark.parametrize("reference", [
    pytest.param("{primitive.layers}", id="brace"),
    pytest.param({"$ref": "#/primitive/layers/$value"}, id="json-pointer"),
])
def test_whole_shadow_array_aliases_remain_valid(reference):
    document = {"primitive": {"layers": token("shadow", [shadow(), shadow()])},
                "semantic": {"layers": token(None, reference)}}
    tokens = flatten(document)
    assert Resolver(tokens).all()["semantic.layers"] == [shadow(), shadow()]
    assert tokens["semantic.layers"]["$type"] == "shadow"


def test_shadow_literal_nested_arrays_are_not_flattened():
    with pytest.raises(TokenError):
        Resolver(flatten({"semantic": {"layers": token("shadow", [[shadow()]])}})).all()


@pytest.fixture
def decorated_project(root):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    original_metadata = deepcopy(base["$extensions"][EXTENSION])
    base["$extensions"][VENDOR] = {"base": [None, True, {"editor": "portable"}]}
    base["semantic"]["$extensions"] = {VENDOR: {"base-group": True}}
    base["component"]["button"]["primary"]["font-size"]["$extensions"] = {VENDOR: {"base-token": True}}
    write_document(base_path, base)
    for mode in MODES:
        path = root / "tokens/modes" / f"{mode}.tokens.json"
        document = read_json(path)
        document["$extensions"][f"{VENDOR}.mode"] = {"mode": mode}
        document["semantic"]["$extensions"] = {f"{VENDOR}.mode": {"mode-group": mode}}
        document["semantic"]["color"]["$extensions"] = {VENDOR: {"mode-group": mode}}
        document["semantic"]["color"]["ink"]["$extensions"] = {VENDOR: {"mode-token": mode}}
        write_document(path, document)
    theme_document = nest({"semantic.spacing.step-2": token("dimension", dimension(8), **{
        "$extensions": {VENDOR: {"theme-token": True}}})}, metadata())
    theme_document["$extensions"][f"{VENDOR}.theme"] = {"theme": True}
    theme_document["semantic"]["$extensions"] = {f"{VENDOR}.theme": {"theme-group": True}}
    theme_document["semantic"]["spacing"]["$extensions"] = {VENDOR: {"theme-group": True}}
    theme = write_document(root / "theme.json", theme_document)
    override_document = nest({"semantic.spacing.step-4": token("dimension", dimension(16), **{
        "$extensions": {VENDOR: {"override-token": True}}})}, metadata())
    override_document["$extensions"][f"{VENDOR}.override"] = {"override": True}
    override_document["semantic"]["$extensions"] = {f"{VENDOR}.override": {"override-group": True}}
    override_document["component"] = {"button": {"$extensions": {VENDOR: {"override-group": True}}}}
    override = write_document(root / "override.json", override_document)
    return root, theme, override, original_metadata


def assert_portable_metadata(document, mode, original_metadata):
    extensions = document["$extensions"]
    assert extensions[VENDOR] == {"base": [None, True, {"editor": "portable"}]}
    assert extensions[f"{VENDOR}.mode"] == {"mode": mode}
    assert extensions[f"{VENDOR}.theme"] == {"theme": True}
    assert extensions[f"{VENDOR}.override"] == {"override": True}
    assert extensions[EXTENSION] == dict(original_metadata, mode=mode, colorSpace="srgb")
    assert document["semantic"]["$extensions"] == {
        VENDOR: {"base-group": True}, f"{VENDOR}.mode": {"mode-group": mode},
        f"{VENDOR}.theme": {"theme-group": True}, f"{VENDOR}.override": {"override-group": True}}
    assert document["semantic"]["color"]["$extensions"] == {VENDOR: {"mode-group": mode}}
    assert document["semantic"]["spacing"]["$extensions"] == {VENDOR: {"theme-group": True}}
    assert document["component"]["button"]["$extensions"] == {VENDOR: {"override-group": True}}
    assert document["component"]["button"]["primary"]["font-size"]["$extensions"] == {VENDOR: {"base-token": True}}
    assert document["semantic"]["color"]["ink"]["$extensions"] == {VENDOR: {"mode-token": mode}}
    assert document["semantic"]["spacing"]["step-2"]["$extensions"] == {VENDOR: {"theme-token": True}}
    assert document["semantic"]["spacing"]["step-4"]["$extensions"] == {VENDOR: {"override-token": True}}


@pytest.mark.parametrize("mode", MODES)
def test_project_dtcg_export_preserves_layer_metadata_and_existing_design_values(decorated_project, mode):
    root, theme, override, original_metadata = decorated_project
    composition = Project(root).compose(mode, theme, [override])
    assert composition.values == Project(ROOT).compose(mode).values
    document = export(composition, "dtcg")
    assert_portable_metadata(document, mode, original_metadata)
    assert Resolver(flatten(document)).all() == composition.values


def test_cli_build_and_dtcg_export_preserve_all_layer_extensions(decorated_project):
    root, theme, override, original_metadata = decorated_project
    outputs = [root / "build.json", root / "export.json"]
    for command, arguments, output in (("build", [], outputs[0]),
                                       ("export", ["--format", "dtcg"], outputs[1])):
        result = subprocess.run([
            sys.executable, "-m", "fjord", command, "--root", str(root), "--mode", "day",
            "--theme", str(theme), "--override", str(override), "--output", str(output), *arguments,
        ], cwd=root, capture_output=True, text=True)
        assert result.returncode == 0, result.stderr
        document = read_json(output)
        assert_portable_metadata(document, "day", original_metadata)
        assert Resolver(flatten(document)).all() == Project(ROOT).compose("day").values
    assert outputs[0].read_bytes() == outputs[1].read_bytes()


def test_pointer_aliases_rebind_after_each_override_with_types_and_provenance(root):
    reference = {"$ref": "#/semantic/spacing/step-3/$value"}
    theme = write_layer(root, "theme.json", {"semantic.spacing.step-4": token(None, reference)})
    first = write_layer(root, "first.json", {"semantic.spacing.step-3": token("dimension", dimension(14))})
    last = write_layer(root, "last.json", {"semantic.spacing.step-3": token("dimension", dimension(12))})
    composition = Project(root).compose("night", theme, [first, last])
    for path in ("semantic.spacing.step-3", "semantic.spacing.step-4", "component.button.primary.padding-inline"):
        assert composition.values[path] == dimension(12)
        assert composition.tokens[path]["$type"] == "dimension"
    assert composition.provenance["semantic.spacing.step-3"] == "override:2:last.json"
    assert composition.provenance["semantic.spacing.step-4"] == "theme:theme.json"
    assert composition.provenance["component.button.primary.padding-inline"] == "base"
    resolved = export(composition, "resolved")["tokens"]["semantic.spacing.step-4"]
    assert resolved["type"] == "dimension"
    assert resolved["value"] == dimension(12)
    assert resolved["original"] == reference
    assert resolved["source"] == "theme:theme.json"
    assert export(composition, "dtcg")["semantic"]["spacing"]["step-4"]["$value"] == reference


@pytest.mark.parametrize("reference", [
    pytest.param("{semantic.custom-gap.$root}", id="brace"),
    pytest.param({"$ref": "#/semantic/custom-gap/$root/$value"}, id="json-pointer"),
])
def test_project_root_aliases_rebind_and_export_distinct_css_names(root, reference):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    base["semantic"]["custom-gap"] = {
        "$type": "dimension", "$root": token(None, dimension(16)), "root": token(None, dimension(20))}
    write_document(base_path, base)
    theme = write_layer(root, "theme.json", {"semantic.spacing.step-4": token(None, reference)})
    override = write_layer(root, "override.json", {"semantic.custom-gap.$root": token("dimension", dimension(12))})
    composition = Project(root).compose("night", theme, [override])
    assert composition.values["semantic.spacing.step-4"] == dimension(12)
    assert composition.values["component.button.primary.padding-inline"] == dimension(12)
    assert composition.provenance["semantic.custom-gap.$root"] == "override:1:override.json"
    document = export(composition, "dtcg")
    assert document["semantic"]["custom-gap"]["$root"]["$value"] == dimension(12)
    assert Resolver(flatten(document)).all()["semantic.spacing.step-4"] == dimension(12)
    css = export(composition, "css")
    assert r"--fjord-token-semantic-custom-gap-\$root: 12px;" in css
    assert "--fjord-token-semantic-custom-gap-root: 20px;" in css


@pytest.mark.parametrize(("keyword", "extension"), [
    pytest.param("$extends", "{semantic.template}", id="extends-brace"),
    pytest.param("$extends", "#/semantic/template", id="extends-pointer"),
    pytest.param("$ref", "#/semantic/template", id="group-ref"),
])
def test_project_extends_keeps_raw_inheritance_live_after_target_override(root, keyword, extension):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    base["semantic"]["template"] = {"$type": "dimension", "$extensions": {VENDOR: {"inherited": True}},
                                    "step-4": token(None, dimension(16))}
    del base["semantic"]["spacing"]["step-4"]
    base["semantic"]["spacing"][keyword] = extension
    write_document(base_path, base)
    theme = write_document(root / "theme.json", {
        "$extensions": {EXTENSION: metadata()}, "semantic": {"spacing": {keyword: extension}}})
    override = write_layer(root, "override.json", {"semantic.template.step-4": token("dimension", dimension(12))})
    composition = Project(root).compose("night", theme, [override])
    assert composition.values["semantic.spacing.step-4"] == dimension(12)
    assert composition.values["component.button.primary.padding-inline"] == dimension(12)
    assert composition.provenance["semantic.template.step-4"] == "override:1:override.json"
    assert composition.provenance["semantic.spacing.step-4"] == "theme:theme.json"
    document = export(composition, "dtcg")
    assert document["semantic"]["spacing"]["$extensions"] == {VENDOR: {"inherited": True}}
    assert Resolver(flatten(document)).all()["semantic.spacing.step-4"] == dimension(12)


def test_project_required_roles_accept_alias_derived_types_from_base_and_mode(root):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    del base["component"]["button"]["primary"]["background"]["$type"]
    write_document(base_path, base)
    mode_path = root / "tokens/modes/day.tokens.json"
    mode = read_json(mode_path)
    del mode["semantic"]["color"]["ink"]["$type"]
    write_document(mode_path, mode)
    composition = Project(root).compose("day")
    expected = Project(ROOT).compose("day")
    assert composition.values == expected.values
    resolved = export(composition, "resolved")["tokens"]
    assert resolved["semantic.color.ink"]["type"] == "color"
    assert resolved["component.button.primary.background"]["type"] == "color"


def test_inferred_alias_type_cannot_change_a_required_mode_role(root):
    mode_path = root / "tokens/modes/day.tokens.json"
    mode = read_json(mode_path)
    mode["semantic"]["color"]["ink"] = token(None, {"$ref": "#/primitive/spacing/space-4/$value"})
    write_document(mode_path, mode)
    with pytest.raises(TokenError, match="semantic.color.ink"):
        Project(root).compose("day")


def test_group_type_override_cannot_change_effective_required_types(root):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    base["semantic"]["spacing"]["$type"] = "dimension"
    base["semantic"]["spacing"]["step-4"] = token(None, dimension(16))
    write_document(base_path, base)
    override = write_document(root / "override.json", {
        "$extensions": {EXTENSION: metadata()}, "semantic": {"spacing": {"$type": "number"}}})
    with pytest.raises(TokenError, match="semantic.spacing.step-4"):
        Project(root).compose("night", overrides=[override])


@pytest.mark.parametrize("reference", [
    pytest.param("{semantic.color.accent}", id="brace"),
    pytest.param({"$ref": "#/semantic/color/accent/$value"}, id="json-pointer"),
])
def test_component_whole_value_aliases_to_semantic_roles_are_equivalent(root, reference):
    override = write_layer(root, "override.json", {"component.button.primary.background": token(None, reference)})
    composition = Project(root).compose("day", overrides=[override])
    assert composition.values["component.button.primary.background"] == composition.values["semantic.color.accent"]
    assert export(composition, "resolved")["tokens"]["component.button.primary.background"]["type"] == "color"


@pytest.mark.parametrize(("path", "kind", "reference"), [
    pytest.param("component.button.primary.background", "color", "{primitive.color.day.accent}", id="component-brace"),
    pytest.param("component.button.primary.background", "color", {"$ref": "#/primitive/color/day/accent/$value"},
                 id="component-pointer"),
    pytest.param("semantic.spacing.step-4", "dimension", "{component.button.primary.padding-block}", id="semantic-brace"),
    pytest.param("semantic.spacing.step-4", "dimension", {"$ref": "#/component/button/primary/padding-block/$value"},
                 id="semantic-pointer"),
])
def test_pointer_aliases_cannot_bypass_project_hierarchy(root, path, kind, reference):
    override = write_layer(root, "override.json", {path: token(kind, reference)})
    with pytest.raises(TokenError, match="semantic role|hierarchy"):
        Project(root).compose("day", overrides=[override])


@pytest.mark.parametrize("reference", [
    pytest.param("{semantic.spacing.step-4}", id="brace"),
    pytest.param({"$ref": "#/semantic/spacing/step-4/$value"}, id="json-pointer"),
])
def test_primitive_pointer_aliases_cannot_reference_semantic_tokens(root, reference):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    base["primitive"]["probe"] = token("dimension", reference)
    write_document(base_path, base)
    with pytest.raises(TokenError, match="hierarchy"):
        Project(root).compose("night")


def test_pointer_deprecation_warning_keeps_source_role_and_replacement(root):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    base["semantic"]["legacy-gap"] = token("dimension", "{semantic.spacing.step-4}", **{
        "$deprecated": "Use {semantic.spacing.step-4}."})
    write_document(base_path, base)
    override = write_layer(root, "legacy-use.json", {"component.button.primary.padding-inline": token(None, {
        "$ref": "#/semantic/legacy-gap/$value"})})
    composition = Project(root).compose("night", overrides=[override])
    assert composition.values["component.button.primary.padding-inline"] == dimension(16)
    assert any(all(detail in warning for detail in (
        str(override), "component.button.primary.padding-inline", "semantic.legacy-gap", "semantic.spacing.step-4"))
        for warning in composition.deprecated)


@pytest.mark.parametrize("reference", [
    pytest.param("{semantic.shadow-source}", id="brace"),
    pytest.param({"$ref": "#/semantic/shadow-source/$value"}, id="json-pointer"),
])
def test_project_shadow_element_object_aliases_export_as_single_layers(root, reference):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    base["semantic"]["shadow-source"] = token("shadow", shadow())
    write_document(base_path, base)
    override = write_layer(root, "override.json", {"semantic.shadow.shadow-sm": token("shadow", [reference])})
    composition = Project(root).compose("night", overrides=[override])
    assert composition.values["semantic.shadow.shadow-sm"] == [shadow()]
    assert "--fjord-token-semantic-shadow-shadow-sm: inset 0px 1px 2px 0px #00000080;" in export(composition, "css")


@pytest.mark.parametrize("reference", [
    pytest.param("{semantic.shadow-source}", id="brace"),
    pytest.param({"$ref": "#/semantic/shadow-source/$value"}, id="json-pointer"),
])
def test_project_rejects_array_valued_shadow_element_aliases_but_accepts_whole_aliases(root, reference):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    base["semantic"]["shadow-source"] = token("shadow", [shadow(), shadow()])
    write_document(base_path, base)
    whole = write_layer(root, "whole.json", {"semantic.shadow.shadow-sm": token(None, reference)})
    composition = Project(root).compose("night", overrides=[whole])
    assert composition.values["semantic.shadow.shadow-sm"] == [shadow(), shadow()]
    assert export(composition, "resolved")["tokens"]["semantic.shadow.shadow-sm"]["type"] == "shadow"
    element = write_layer(root, "element.json", {"semantic.shadow.shadow-sm": token("shadow", [reference])})
    with pytest.raises(TokenError):
        Project(root).compose("night", overrides=[element])


def test_single_shadow_object_exports_through_project_and_cli_css(root):
    override = write_layer(root, "single-shadow.json", {"semantic.shadow.shadow-sm": token("shadow", shadow())})
    composition = Project(root).compose("night", overrides=[override])
    assert composition.values["semantic.shadow.shadow-sm"] == shadow()
    declaration = "--fjord-token-semantic-shadow-shadow-sm: inset 0px 1px 2px 0px #00000080;"
    assert declaration in export(composition, "css")
    output = root / "single-shadow.css"
    result = subprocess.run([
        sys.executable, "-m", "fjord", "export", "--root", str(root), "--format", "css",
        "--override", str(override), "--output", str(output),
    ], cwd=root, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    assert declaration in output.read_text(encoding="utf-8")


def test_project_property_pointers_resolve_dimension_fields_and_color_array_entries(root):
    base_path = root / "tokens/base.tokens.json"
    base = read_json(base_path)
    base["semantic"]["spacing"]["step-3"] = token("dimension", dimension(12))
    write_document(base_path, base)
    gap = {"value": {"$ref": "#/semantic/spacing/step-3/$value/value"},
           "unit": {"$ref": "#/semantic/spacing/step-3/$value/unit"}}
    ink = {"colorSpace": "srgb", "components": [
        {"$ref": f"#/primitive/color/day/ink/$value/components/{index}"} for index in range(3)],
        "alpha": {"$ref": "#/primitive/color/day/ink/$value/alpha"}}
    theme = write_layer(root, "properties.json", {"semantic.spacing.step-4": token("dimension", gap),
                                                "semantic.color.ink": token("color", ink)})
    override = write_layer(root, "target.json", {"semantic.spacing.step-3": token("dimension", dimension(14))})
    composition = Project(root).compose("day", theme, [override])
    assert composition.values["semantic.spacing.step-4"] == dimension(14)
    assert composition.values["component.button.primary.padding-inline"] == dimension(14)
    assert composition.values["semantic.color.ink"] == Project(ROOT).compose("day").values["semantic.color.ink"]
    document = export(composition, "dtcg")
    assert document["semantic"]["spacing"]["step-4"]["$value"] == gap
    assert document["semantic"]["color"]["ink"]["$value"] == ink
    assert Resolver(flatten(document)).all()["semantic.spacing.step-4"] == dimension(14)


@pytest.mark.parametrize("value", [
    pytest.param({"value": {"$ref": "#/primitive/blue/$value/components"}, "unit": "px"}, id="array-not-number"),
    pytest.param({"value": 8, "unit": {"$ref": "#/primitive/blue/$value/alpha"}}, id="number-not-unit"),
])
def test_property_pointer_targets_must_match_the_containing_field_type(value):
    document = {"primitive": {"blue": token("color", color_from_hex("#336699"))},
                "semantic": {"gap": token("dimension", value)}}
    with pytest.raises(TokenError):
        Resolver(flatten(document)).all()
