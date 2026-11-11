import json
from pathlib import Path
import shutil
import subprocess
import sys

import pytest

from fjord.dtcg import EXTENSION, Resolver, TokenError, color_from_hex, flatten, hex_color, nest, read_json
from fjord.project import Project, check_contrast, metadata, validate_project


ROOT = Path(__file__).resolve().parents[1]


def token(kind, value, description):
    return {"$type": kind, "$value": value, "$description": description}


@pytest.fixture
def root(tmp_path):
    shutil.copytree(ROOT / "tokens", tmp_path / "tokens")
    return tmp_path


def patch_tokens(root, file, path, definition):
    document = read_json(root / file)
    tokens = flatten(document)
    if definition is None:
        del tokens[path]
    else:
        tokens[path] = definition
    (root / file).write_text(json.dumps(nest(tokens, document["$extensions"][EXTENSION])), encoding="utf-8")


def add_deprecated_role(root, path="semantic.test-gap", kind="dimension", replacement="semantic.spacing.step-4"):
    definition = token(kind, "{" + replacement + "}", "Synthetic deprecated role.")
    definition["$deprecated"] = f"Use {{{replacement}}}."
    patch_tokens(root, "tokens/base.tokens.json", path, definition)


def write_layer(root, name, path, kind, value):
    file = root / name
    file.write_text(json.dumps(nest({path: token(kind, value, "Test override.")}, metadata())), encoding="utf-8")
    return file


def run_cli(root, *arguments):
    return subprocess.run([sys.executable, "-m", "fjord", *map(str, arguments)],
                          cwd=root, capture_output=True, text=True)


def test_all_modes_pass_without_documentation_or_implementations(root):
    report = validate_project(root)
    assert report.deprecated == []


def test_shadow_has_typed_fields_and_inset(root):
    values = Project(root).compose("night").values
    layers = values["semantic.shadow.shadow-sm"]
    assert layers[0]["inset"]
    assert not layers[1]["inset"]
    assert layers[1]["offsetY"] == {"value": 1, "unit": "px"}
    assert layers[1]["blur"] == {"value": 2, "unit": "px"}
    assert layers[1]["color"]["alpha"] == pytest.approx(89 / 255, rel=0, abs=5e-8)


def test_typography_and_font_metadata(root):
    composed = Project(root).compose("night")
    display = composed.values["semantic.typography.display"]
    assert display["fontSize"] == {"value": 40, "unit": "px"}
    assert display["letterSpacing"] == {"value": -0.4, "unit": "px"}
    assert display["lineHeight"] == 1.2
    assert display["fontFamily"][0] == "Raleway"
    italic = composed.tokens["semantic.typography.code-italic"]["$extensions"][EXTENSION]
    assert italic["fontStyle"] == "italic"


def test_order_is_base_mode_theme_then_overrides(root):
    path = "semantic.spacing.step-4"
    theme = write_layer(root, "theme.json", path, "dimension", {"value": 12, "unit": "px"})
    document = read_json(theme)
    document["semantic"]["density"] = {"gap-compact": {
        "$type": "dimension", "$value": "{semantic.spacing.step-2}",
        "$description": "Keep compact density within the overridden comfortable gap."}}
    theme.write_text(json.dumps(document))
    first = write_layer(root, "first.json", path, "dimension", {"value": 10, "unit": "px"})
    last = write_layer(root, "last.json", path, "dimension", {"value": 8, "unit": "px"})
    project = Project(root)
    assert project.compose("night").values[path]["value"] == 16
    assert project.compose("night", theme).values[path]["value"] == 12
    composed = project.compose("night", theme, [first, last])
    assert composed.values[path]["value"] == 8
    assert composed.values["component.button.primary.padding-inline"]["value"] == 8
    assert "override:2:last.json" in composed.provenance[path]
    assert composed.deprecated == []


def test_mode_values_never_inherit_night(root):
    patch_tokens(root, "tokens/modes/day.tokens.json", "semantic.color.ink", None)
    with pytest.raises(TokenError, match="missing mode roles.*semantic.color.ink"):
        Project(root).compose("day")


def test_base_cannot_supply_mode_fallbacks(root):
    patch_tokens(root, "tokens/base.tokens.json", "semantic.color.ink", token("color", color_from_hex("#ffffff"), "Invalid base role."))
    with pytest.raises(TokenError, match="base must not define mode roles"):
        Project(root).compose("night")


def test_unknown_mode_fails(root):
    with pytest.raises(TokenError, match="unknown mode"):
        Project(root).compose("dya")


def test_unknown_override_fails(root):
    layer = write_layer(root, "override.json", "semantic.color.unknown", "color", color_from_hex("#000000"))
    with pytest.raises(TokenError, match="unknown override path"):
        Project(root).compose("night", overrides=[layer])


def test_primitive_override_fails(root):
    layer = write_layer(root, "override.json", "primitive.spacing.space-4", "dimension", {"value": 12, "unit": "px"})
    with pytest.raises(TokenError, match="primitive overrides are forbidden"):
        Project(root).compose("night", overrides=[layer])


def test_type_change_fails(root):
    layer = write_layer(root, "override.json", "semantic.color.ink", "number", 42)
    with pytest.raises(TokenError, match="type change is forbidden"):
        Project(root).compose("night", overrides=[layer])


def test_null_cannot_delete_a_token(root):
    layer = write_layer(root, "override.json", "semantic.color.ink", "color", None)
    with pytest.raises(TokenError, match="color requires fields"):
        Project(root).compose("night", overrides=[layer])


def test_composites_replace_atomically(root):
    path = "semantic.shadow.shadow-sm"
    original = Project(root).compose("night").values[path]
    layer = write_layer(root, "override.json", path, "shadow", [original[1]])
    composed = Project(root).compose("night", overrides=[layer])
    assert composed.values[path] == [original[1]]


def test_partial_composite_cannot_deep_merge(root):
    layer = write_layer(root, "override.json", "semantic.shadow.shadow-sm", "shadow", {"blur": {"value": 4, "unit": "px"}})
    with pytest.raises(TokenError, match="shadow requires fields"):
        Project(root).compose("night", overrides=[layer])


def test_invalid_earlier_layer_cannot_be_hidden(root):
    first = write_layer(root, "first.json", "semantic.spacing.step-4", "dimension", None)
    last = write_layer(root, "last.json", "semantic.spacing.step-4", "dimension", {"value": 12, "unit": "px"})
    with pytest.raises(TokenError, match="dimension requires fields"):
        Project(root).compose("night", overrides=[first, last])


def test_invalid_semantic_dimensions_cannot_be_hidden(root):
    first = write_layer(root, "first.json", "semantic.spacing.step-4", "dimension", {"value": -1, "unit": "px"})
    last = write_layer(root, "last.json", "semantic.spacing.step-4", "dimension", {"value": 12, "unit": "px"})
    with pytest.raises(TokenError, match="must not be negative"):
        Project(root).compose("night", overrides=[first, last])


@pytest.mark.parametrize(("path", "kind", "value", "message"), [
    pytest.param("semantic.font-size.text-md", "dimension", {"value": 0, "unit": "px"},
                 "font size must be positive", id="zero-font-size"),
    pytest.param("semantic.z-index.z-modal", "number", 1.5,
                 "stacking order must be an integer", id="fractional-stacking-order"),
])
def test_semantic_font_size_and_stacking_order(root, path, kind, value, message):
    layer = write_layer(root, "override.json", path, kind, value)
    with pytest.raises(TokenError, match=message):
        Project(root).compose("night", overrides=[layer])


def test_alias_rebinds_after_override(root):
    layer = write_layer(root, "override.json", "semantic.color.accent", "color", "{semantic.color.accent-hover}")
    composed = Project(root).compose("day", overrides=[layer])
    assert composed.values["semantic.color.focus-ring"] == composed.values["semantic.color.accent-hover"]
    assert hex_color(composed.values["component.button.primary.background"]) == "#00527e"


@pytest.mark.parametrize(("value", "message"), [
    pytest.param("{semantic.color.absent}", "missing alias target", id="missing-target"),
    pytest.param("{semantic.color.focus-ring}", "alias cycle", id="cycle"),
])
def test_missing_and_cyclic_references_fail(root, value, message):
    layer = write_layer(root, "override.json", "semantic.color.accent", "color", value)
    with pytest.raises(TokenError, match=message):
        Project(root).compose("night", overrides=[layer])


def test_wrong_alias_type_fails(root):
    layer = write_layer(root, "override.json", "semantic.color.accent", "color", "{semantic.spacing.step-4}")
    with pytest.raises(TokenError, match="expected color"):
        Project(root).compose("night", overrides=[layer])


@pytest.mark.parametrize("value", [
    pytest.param(color_from_hex("#ffffff"), id="literal"),
    pytest.param("{primitive.color.night.accent}", id="primitive-alias"),
])
def test_component_must_reference_semantic_role(root, value):
    layer = write_layer(root, "override.json", "component.button.primary.background", "color", value)
    with pytest.raises(TokenError, match="must alias a semantic role"):
        Project(root).compose("night", overrides=[layer])


def test_component_contrast_is_checked_after_override(root):
    layer = write_layer(root, "override.json", "component.button.primary.background", "color", "{semantic.color.on-accent}")
    with pytest.raises(TokenError, match="component.button.primary.foreground.*below 4.5"):
        check_contrast(Project(root).compose("night", overrides=[layer]))


def test_fractional_srgb_contrast_is_not_quantized(root):
    color = {"colorSpace": "srgb", "components": [0.45, 0.45, 0.45], "alpha": 1}
    layer = write_layer(root, "override.json", "semantic.color.ansi-bright-black", "color", color)
    with pytest.raises(TokenError, match="below 4.5"):
        check_contrast(Project(root).compose("night", overrides=[layer]))


def test_alpha_in_contrast_is_rejected(root):
    color = color_from_hex("#e1e9ef80")
    layer = write_layer(root, "override.json", "semantic.color.ink", "color", color)
    with pytest.raises(TokenError, match="must be opaque"):
        check_contrast(Project(root).compose("night", overrides=[layer]))


@pytest.mark.parametrize(("kind", "value"), [
    pytest.param("color", "#ffffff", id="color-string"),
    pytest.param("color", {"colorSpace": "srgb", "components": [2, 0, 0]}, id="color-out-of-range"),
    pytest.param("color", {"colorSpace": "srgb", "components": [0, 0, 0], "alpha": True}, id="boolean-alpha"),
    pytest.param("dimension", {"value": True, "unit": "px"}, id="boolean-dimension"),
    pytest.param("dimension", {"value": 1, "unit": "em"}, id="unsupported-dimension-unit"),
    pytest.param("duration", {"value": -1, "unit": "ms"}, id="negative-duration"),
    pytest.param("fontWeight", 1001, id="font-weight-out-of-range"),
    pytest.param("fontFamily", [], id="empty-font-family"),
    pytest.param("cubicBezier", [2, 0, 0, 1], id="bezier-out-of-range"),
    pytest.param("number", False, id="boolean-number"),
])
def test_invalid_typed_values_are_rejected(kind, value):
    with pytest.raises(TokenError):
        Resolver({"primitive.test": token(kind, value, "Invalid value.")}).all()


def test_literal_nested_shadow_arrays_fail(root):
    layers = Project(root).compose("night").values["semantic.shadow.shadow-sm"]
    layer = write_layer(root, "override.json", "semantic.shadow.shadow-sm", "shadow", [layers])
    with pytest.raises(TokenError, match="literal nested arrays"):
        Project(root).compose("night", overrides=[layer])


@pytest.mark.parametrize("mode", ["night", "day", "aurora"])
def test_example_layers_build_in_every_mode(root, mode):
    composition = Project(root).compose(mode, ROOT / "examples/themes/soft-windows.tokens.json",
                                        [ROOT / "examples/overrides/compact-spacing.tokens.json"])
    check_contrast(composition)
    assert composition.deprecated == []


@pytest.mark.parametrize("document", [
    pytest.param({"bad.name": token("number", 1, "Bad name.")}, id="dotted-name"),
    pytest.param({"x": {"$type": []}}, id="invalid-type"),
    pytest.param({"x": {"$ref": "#/other"}}, id="unsupported-reference"),
    pytest.param({"x": dict(token("number", 1, "Bad node."), child={})}, id="token-with-child"),
])
def test_invalid_structures_and_names_are_rejected(document):
    with pytest.raises(TokenError):
        flatten(document)


def test_duplicate_json_keys_fail(tmp_path):
    path = tmp_path / "duplicate.json"
    path.write_text('{"color": {}, "color": {}}')
    with pytest.raises(TokenError, match="duplicate JSON key"):
        read_json(path)


@pytest.mark.parametrize("content", [
    pytest.param('{"value": NaN}', id="nan"),
    pytest.param('{"value": Infinity}', id="positive-infinity"),
    pytest.param('{"value": -Infinity}', id="negative-infinity"),
    pytest.param('{"value":', id="malformed-json"),
])
def test_nonfinite_and_malformed_json_fail(tmp_path, content):
    path = tmp_path / "invalid.json"
    path.write_text(content, encoding="utf-8")
    with pytest.raises(TokenError):
        read_json(path)


def test_design_metadata_describes_current_assets(root):
    info = read_json(root / "tokens/base.tokens.json")["$extensions"][EXTENSION]
    assert info["name"] == "Fjord"
    assert set(info["designSeeds"]) == {"primary", "accents"}
    assert info["fontAssets"] == "assets/fonts/manifest.json"


def test_group_types_and_deprecation_are_inherited():
    tokens = flatten({"primitive": {"$type": "number", "$deprecated": "Use the new role.",
                                   "gap": {"$value": 1, "$description": "Group spacing."}}})
    assert tokens["primitive.gap"]["$type"] == "number"
    assert tokens["primitive.gap"]["$deprecated"] == "Use the new role."


def test_inherited_deprecated_declaration_is_quiet_until_used(root):
    path = root / "tokens/base.tokens.json"
    document = read_json(path)
    document["semantic"]["legacy-spacing"] = {
        "$type": "dimension", "$deprecated": "Use {semantic.spacing.step-4}.",
        "gap": {"$value": "{semantic.spacing.step-4}", "$description": "Deprecated gap."}}
    path.write_text(json.dumps(document))
    assert validate_project(root).deprecated == []
    project = Project(root)
    assert project.compose("day").values["semantic.legacy-spacing.gap"]["value"] == 16
    layer = write_layer(root, "legacy-use.json", "component.button.primary.padding-inline", "dimension",
                        "{semantic.legacy-spacing.gap}")
    composed = project.compose("day", overrides=[layer])
    assert len(composed.deprecated) == 1
    warning = composed.deprecated[0]
    for detail in (str(layer), "component.button.primary.padding-inline",
                   "semantic.legacy-spacing.gap", "semantic.spacing.step-4"):
        assert detail in warning


def test_source_aliases_warn_once_per_actual_use_across_modes(root):
    add_deprecated_role(root)
    add_deprecated_role(root, "semantic.test-color", "color", "semantic.color.bg-pressed")
    patch_tokens(root, "tokens/base.tokens.json", "component.button.primary.padding-inline",
                 token("dimension", "{semantic.test-gap}", "Synthetic deprecated spacing use."))
    patch_tokens(root, "tokens/modes/night.tokens.json", "semantic.color.bg-selected",
                 token("color", "{semantic.test-color}", "Synthetic deprecated background use."))
    warnings = validate_project(root).deprecated
    assert len(warnings) == 2
    for source, role, deprecated, replacement in (
        ("tokens/base.tokens.json", "component.button.primary.padding-inline",
         "semantic.test-gap", "semantic.spacing.step-4"),
        ("tokens/modes/night.tokens.json", "semantic.color.bg-selected",
         "semantic.test-color", "semantic.color.bg-pressed"),
    ):
        matching = [warning for warning in warnings if str(root / source) in warning]
        assert len(matching) == 1
        for detail in (role, deprecated, replacement):
            assert detail in matching[0]


def test_deprecated_theme_assignment_uses_prior_metadata(root):
    add_deprecated_role(root)
    layer = write_layer(root, "deprecated-theme.json", "semantic.test-gap", "dimension", {"value": 12, "unit": "px"})
    composed = Project(root).compose("night", theme=layer)
    assert "$deprecated" not in composed.tokens["semantic.test-gap"]
    assert composed.values["semantic.test-gap"]["value"] == 12
    assert composed.values["semantic.spacing.step-4"]["value"] == 16
    assert len(composed.deprecated) == 1
    for detail in (str(layer), "semantic.test-gap", "semantic.spacing.step-4"):
        assert detail in composed.deprecated[0]


def test_deprecation_metadata_cannot_be_cleared_for_later_layers(root):
    add_deprecated_role(root)
    first = write_layer(root, "replace-deprecated.json", "semantic.test-gap", "dimension", {"value": 12, "unit": "px"})
    document = read_json(first)
    document["semantic"]["test-gap"]["$deprecated"] = False
    first.write_text(json.dumps(document))
    last = write_layer(root, "reference-deprecated.json", "component.button.primary.padding-inline", "dimension",
                       "{semantic.test-gap}")
    composed = Project(root).compose("night", overrides=[first, last])
    assert composed.tokens["semantic.test-gap"]["$deprecated"] is False
    assert composed.values["component.button.primary.padding-inline"]["value"] == 12
    assert len(composed.deprecated) == 2
    for source in (first, last):
        matching = [warning for warning in composed.deprecated if str(source) in warning]
        assert len(matching) == 1
        assert "semantic.test-gap" in matching[0]
        assert "semantic.spacing.step-4" in matching[0]


def test_earlier_deprecated_reference_is_not_erased_by_later_override(root):
    add_deprecated_role(root)
    role = "component.button.primary.padding-inline"
    first = write_layer(root, "deprecated.json", role, "dimension", "{semantic.test-gap}")
    last = write_layer(root, "modern.json", role, "dimension", "{semantic.spacing.step-4}")
    composed = Project(root).compose("night", overrides=[first, last])
    assert composed.tokens[role]["$value"] == "{semantic.spacing.step-4}"
    assert len(composed.deprecated) == 1
    for detail in (str(first), role, "semantic.test-gap", "semantic.spacing.step-4"):
        assert detail in composed.deprecated[0]


def test_nested_composite_references_warn_once_per_deprecated_target(root):
    add_deprecated_role(root, replacement="semantic.spacing.step-1")
    role = "semantic.shadow.shadow-sm"
    shadows = Project(root).compose("night").values[role]
    shadows[0]["offsetX"] = "{semantic.test-gap}"
    shadows[0]["offsetY"] = "{semantic.test-gap}"
    layer = write_layer(root, "deprecated-shadow.json", role, "shadow", shadows)
    composed = Project(root).compose("night", overrides=[layer])
    assert composed.values[role][0]["offsetX"] == {"value": 4, "unit": "px"}
    assert len(composed.deprecated) == 1
    for detail in (str(layer), role, "semantic.test-gap", "semantic.spacing.step-1"):
        assert detail in composed.deprecated[0]


def test_removed_role_cannot_be_overridden(root):
    layer = write_layer(root, "removed.json", "semantic.spacing.space-4", "dimension", {"value": 12, "unit": "px"})
    with pytest.raises(TokenError, match="semantic.spacing.space-4: unknown override path"):
        Project(root).compose("night", overrides=[layer])


@pytest.mark.parametrize(("path", "kind", "target"), [
    pytest.param("component.button.primary.padding-inline", "dimension", "semantic.spacing.space-4",
                 id="removed-spacing"),
    pytest.param("semantic.color.bg-pressed", "color", "primitive.color.night.bg-active",
                 id="removed-color"),
])
def test_removed_roles_cannot_be_alias_targets(root, path, kind, target):
    layer = write_layer(root, "removed-reference.json", path, kind, "{" + target + "}")
    with pytest.raises(TokenError, match="missing alias target " + target):
        Project(root).compose("night", overrides=[layer])


def test_pressed_and_selected_have_independent_roles(root):
    layer = write_layer(root, "selected.json", "semantic.color.bg-selected", "color", "{semantic.color.bg-surface}")
    composed = Project(root).compose("night", overrides=[layer])
    assert composed.values["semantic.color.bg-selected"] == composed.values["semantic.color.bg-surface"]
    assert composed.values["semantic.color.bg-selected"] != composed.values["semantic.color.bg-pressed"]
    check_contrast(composed)
    layer = write_layer(root, "pressed.json", "semantic.color.accent-pressed", "color", "{semantic.color.accent}")
    composed = Project(root).compose("night", overrides=[layer])
    assert composed.values["component.button.primary.pressed-background"] == composed.values["semantic.color.accent"]
    check_contrast(composed)


@pytest.mark.parametrize(("field", "value"), [
    pytest.param("schemaVersion", 3, id="unsupported-schema"),
    pytest.param("version", "1.0.0", id="unsupported-major-contract"),
    pytest.param("version", "0.4.0", id="old-contract"),
])
def test_unknown_schema_or_contract_version_fails(root, field, value):
    path = root / "tokens/contract.json"
    changed = dict(read_json(path), **{field: value})
    path.write_text(json.dumps(changed))
    with pytest.raises(TokenError):
        Project(root)


def test_layers_require_compatible_metadata(root):
    layer = write_layer(root, "override.json", "semantic.spacing.step-4", "dimension", {"value": 20, "unit": "px"})
    document = read_json(layer)
    document["$extensions"][EXTENSION]["contractVersion"] = "0.5.1"
    layer.write_text(json.dumps(document))
    assert Project(root).compose("night", overrides=[layer]).values["semantic.spacing.step-4"]["value"] == 20
    document["$extensions"][EXTENSION]["contractVersion"] = "0.4.0"
    layer.write_text(json.dumps(document))
    with pytest.raises(TokenError, match=r"unsupported contract version.*expected a 0\.5\.x contract"):
        Project(root).compose("night", overrides=[layer])


def test_contract_cannot_remove_required_roles(root):
    path = root / "tokens/contract.json"
    data = read_json(path)
    del data["required"]["semantic.color.ink"]
    path.write_text(json.dumps(data))
    with pytest.raises(TokenError, match="required role semantic.color.ink"):
        Project(root)


def test_cli_build_is_valid_dtcg_and_repeatable(root):
    output = root / "dist/day.tokens.json"
    result = run_cli(root, "build", "--root", root, "--mode", "day")
    assert result.returncode == 0, result.stderr
    before = output.read_bytes()
    document = read_json(output)
    values = Resolver(flatten(document)).all()
    assert hex_color(values["semantic.color.ink"]) == "#16232f"
    assert run_cli(root, "build", "--root", root, "--mode", "day").returncode == 0
    assert output.read_bytes() == before


def test_build_will_not_overwrite_sources(root):
    output = root / "tokens/base.tokens.json"
    before = output.read_bytes()
    result = run_cli(root, "build", "--root", root, "--output", output)
    assert result.returncode == 1
    assert "must not replace token sources" in result.stderr
    assert output.read_bytes() == before


def test_build_failure_leaves_existing_output_intact(root):
    output = root / "existing.json"
    output.write_text("keep")
    layer = write_layer(root, "bad.json", "semantic.color.ink", "color", color_from_hex("#111920"))
    result = run_cli(root, "build", "--root", root, "--override", layer, "--output", output)
    assert result.returncode == 1
    assert output.read_text() == "keep"
    assert "Traceback" not in result.stderr


@pytest.mark.parametrize(("arguments", "returncode"), [
    pytest.param(["--help"], 0, id="help"),
    pytest.param(["build", "--mode", "wrong"], 2, id="unknown-mode"),
    pytest.param(["build", "--mod", "night"], 2, id="abbreviated-option"),
])
def test_cli_help_and_invalid_arguments(root, arguments, returncode):
    assert run_cli(root, *arguments).returncode == returncode


def test_cli_default_paths_work_outside_project(root):
    result = run_cli(root, "validate")
    assert result.returncode == 0, result.stderr
    assert "PASS:" in result.stdout


@pytest.mark.parametrize("relative", ["tokens/modes/day.tokens.json", "tokens/usage.json"])
def test_cli_reports_missing_sources_without_tracebacks(root, relative):
    (root / relative).unlink()
    result = run_cli(root, "validate", "--root", root)
    assert result.returncode == 1
    assert relative in result.stderr
    assert "Traceback" not in result.stderr
