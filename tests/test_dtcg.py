import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from fjord_tokens.dtcg import EXTENSION, Resolver, TokenError, color_from_hex, flatten, hex_color, nest, read_json
from fjord_tokens.project import Project, check_contrast, metadata, validate_project


ROOT = Path(__file__).resolve().parents[1]


def token(kind, value, description):
    return {"$type": kind, "$value": value, "$description": description}


class DtcgTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(prefix="fjord-test-")
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        shutil.copytree(ROOT / "tokens", self.root / "tokens")

    def patch(self, file, path, definition):
        document = read_json(self.root / file)
        tokens = flatten(document)
        if definition is None:
            del tokens[path]
        else:
            tokens[path] = definition
        (self.root / file).write_text(json.dumps(nest(tokens, document["$extensions"][EXTENSION])), encoding="utf-8")

    def layer(self, name, path, kind, value):
        file = self.root / name
        file.write_text(json.dumps(nest({path: token(kind, value, "Test override.")}, metadata())), encoding="utf-8")
        return file

    def cli(self, *arguments):
        return subprocess.run([sys.executable, "-m", "fjord_tokens", *map(str, arguments)],
                              cwd=self.root, capture_output=True, text=True)

    def test_all_modes_pass_without_documentation_or_implementations(self):
        report = validate_project(self.root)
        self.assertEqual(report.contrast_checks, 430)
        self.assertGreater(report.tokens, 140)

    def test_shadow_has_typed_fields_and_inset(self):
        values = Project(self.root).compose("night").values
        layers = values["semantic.shadow.shadow-sm"]
        self.assertTrue(layers[0]["inset"])
        self.assertFalse(layers[1]["inset"])
        self.assertEqual(layers[1]["offsetY"], {"value": 1, "unit": "px"})
        self.assertEqual(layers[1]["blur"], {"value": 2, "unit": "px"})
        self.assertAlmostEqual(layers[1]["color"]["alpha"], 89 / 255)

    def test_typography_and_font_metadata(self):
        composed = Project(self.root).compose("night")
        display = composed.values["semantic.typography.type-display"]
        self.assertEqual(display["fontSize"], {"value": 40, "unit": "px"})
        self.assertEqual(display["letterSpacing"], {"value": -0.4, "unit": "px"})
        self.assertEqual(display["lineHeight"], 1.2)
        self.assertEqual(display["fontFamily"][0], "Raleway")
        italic = composed.tokens["semantic.typography.type-code-italic"]["$extensions"][EXTENSION]
        self.assertEqual(italic["fontStyle"], "italic")
        self.assertEqual(len([p for p in composed.tokens if p.startswith("semantic.typography.")]), 15)

    def test_order_is_base_mode_theme_then_overrides(self):
        path = "semantic.spacing.space-4"
        theme = self.layer("theme.json", path, "dimension", {"value": 12, "unit": "px"})
        document = read_json(theme)
        document["semantic"]["density"] = {"gap-compact": {
            "$type": "dimension", "$value": "{semantic.spacing.space-2}",
            "$description": "Keep compact density within the overridden comfortable gap."}}
        theme.write_text(json.dumps(document))
        first = self.layer("first.json", path, "dimension", {"value": 10, "unit": "px"})
        last = self.layer("last.json", path, "dimension", {"value": 8, "unit": "px"})
        project = Project(self.root)
        self.assertEqual(project.compose("night").values[path]["value"], 16)
        self.assertEqual(project.compose("night", theme).values[path]["value"], 12)
        composed = project.compose("night", theme, [first, last])
        self.assertEqual(composed.values[path]["value"], 8)
        self.assertEqual(composed.values["component.button.primary.padding-inline"]["value"], 8)
        self.assertIn("override:2:last.json", composed.provenance[path])

    def test_mode_values_never_inherit_night(self):
        self.patch("tokens/modes/day.tokens.json", "semantic.color.ink", None)
        with self.assertRaisesRegex(TokenError, "missing mode roles.*semantic.color.ink"):
            Project(self.root).compose("day")

    def test_base_cannot_supply_mode_fallbacks(self):
        self.patch("tokens/base.tokens.json", "semantic.color.ink", token("color", color_from_hex("#ffffff"), "Invalid base role."))
        with self.assertRaisesRegex(TokenError, "base must not define mode roles"):
            Project(self.root).compose("night")

    def test_unknown_mode_fails(self):
        with self.assertRaisesRegex(TokenError, "unknown mode"):
            Project(self.root).compose("dya")

    def test_unknown_override_fails(self):
        layer = self.layer("override.json", "semantic.color.unknown", "color", color_from_hex("#000000"))
        with self.assertRaisesRegex(TokenError, "unknown override path"):
            Project(self.root).compose("night", overrides=[layer])

    def test_primitive_override_fails(self):
        layer = self.layer("override.json", "primitive.spacing.space-4", "dimension", {"value": 12, "unit": "px"})
        with self.assertRaisesRegex(TokenError, "primitive overrides are forbidden"):
            Project(self.root).compose("night", overrides=[layer])

    def test_type_change_fails(self):
        layer = self.layer("override.json", "semantic.color.ink", "number", 42)
        with self.assertRaisesRegex(TokenError, "type change is forbidden"):
            Project(self.root).compose("night", overrides=[layer])

    def test_null_cannot_delete_a_token(self):
        layer = self.layer("override.json", "semantic.color.ink", "color", None)
        with self.assertRaisesRegex(TokenError, "color requires fields"):
            Project(self.root).compose("night", overrides=[layer])

    def test_composites_replace_atomically(self):
        path = "semantic.shadow.shadow-sm"
        original = Project(self.root).compose("night").values[path]
        layer = self.layer("override.json", path, "shadow", [original[1]])
        composed = Project(self.root).compose("night", overrides=[layer])
        self.assertEqual(composed.values[path], [original[1]])

    def test_partial_composite_cannot_deep_merge(self):
        layer = self.layer("override.json", "semantic.shadow.shadow-sm", "shadow", {"blur": {"value": 4, "unit": "px"}})
        with self.assertRaisesRegex(TokenError, "shadow requires fields"):
            Project(self.root).compose("night", overrides=[layer])

    def test_invalid_earlier_layer_cannot_be_hidden(self):
        first = self.layer("first.json", "semantic.spacing.space-4", "dimension", None)
        last = self.layer("last.json", "semantic.spacing.space-4", "dimension", {"value": 12, "unit": "px"})
        with self.assertRaisesRegex(TokenError, "dimension requires fields"):
            Project(self.root).compose("night", overrides=[first, last])

    def test_invalid_semantic_dimensions_cannot_be_hidden(self):
        first = self.layer("first.json", "semantic.spacing.space-4", "dimension", {"value": -1, "unit": "px"})
        last = self.layer("last.json", "semantic.spacing.space-4", "dimension", {"value": 12, "unit": "px"})
        with self.assertRaisesRegex(TokenError, "must not be negative"):
            Project(self.root).compose("night", overrides=[first, last])

    def test_semantic_font_size_and_stacking_order(self):
        for path, kind, value, message in (
            ("semantic.font-size.text-md", "dimension", {"value": 0, "unit": "px"}, "font size must be positive"),
            ("semantic.z-index.z-modal", "number", 1.5, "stacking order must be an integer"),
        ):
            with self.subTest(path=path):
                layer = self.layer("override.json", path, kind, value)
                with self.assertRaisesRegex(TokenError, message):
                    Project(self.root).compose("night", overrides=[layer])

    def test_alias_rebinds_after_override(self):
        layer = self.layer("override.json", "semantic.color.accent", "color", "{semantic.color.accent-hover}")
        composed = Project(self.root).compose("day", overrides=[layer])
        self.assertEqual(composed.values["semantic.color.focus-ring"], composed.values["semantic.color.accent-hover"])
        self.assertEqual(hex_color(composed.values["component.button.primary.background"]), "#00527e")

    def test_missing_and_cyclic_references_fail(self):
        for value, message in (("{semantic.color.absent}", "missing alias target"),
                               ("{semantic.color.focus-ring}", "alias cycle")):
            with self.subTest(value=value):
                layer = self.layer("override.json", "semantic.color.accent", "color", value)
                with self.assertRaisesRegex(TokenError, message):
                    Project(self.root).compose("night", overrides=[layer])

    def test_wrong_alias_type_fails(self):
        layer = self.layer("override.json", "semantic.color.accent", "color", "{semantic.spacing.space-4}")
        with self.assertRaisesRegex(TokenError, "expected color"):
            Project(self.root).compose("night", overrides=[layer])

    def test_component_must_reference_semantic_role(self):
        for value in (color_from_hex("#ffffff"), "{primitive.color.night.accent}"):
            with self.subTest(value=value):
                layer = self.layer("override.json", "component.button.primary.background", "color", value)
                with self.assertRaisesRegex(TokenError, "must alias a semantic role"):
                    Project(self.root).compose("night", overrides=[layer])

    def test_component_contrast_is_checked_after_override(self):
        layer = self.layer("override.json", "component.button.primary.background", "color", "{semantic.color.on-accent}")
        with self.assertRaisesRegex(TokenError, "component.button.primary.foreground.*below 4.5"):
            check_contrast(Project(self.root).compose("night", overrides=[layer]))

    def test_fractional_srgb_contrast_is_not_quantized(self):
        color = {"colorSpace": "srgb", "components": [0.45, 0.45, 0.45], "alpha": 1}
        layer = self.layer("override.json", "semantic.color.ansi-bright-black", "color", color)
        with self.assertRaisesRegex(TokenError, "below 4.5"):
            check_contrast(Project(self.root).compose("night", overrides=[layer]))

    def test_alpha_in_contrast_is_rejected(self):
        color = color_from_hex("#e1e9ef80")
        layer = self.layer("override.json", "semantic.color.ink", "color", color)
        with self.assertRaisesRegex(TokenError, "must be opaque"):
            check_contrast(Project(self.root).compose("night", overrides=[layer]))


    def test_invalid_typed_values_are_rejected(self):
        cases = [("color", "#ffffff"), ("color", {"colorSpace": "srgb", "components": [2, 0, 0]}),
                 ("color", {"colorSpace": "srgb", "components": [0, 0, 0], "alpha": True}),
                 ("dimension", {"value": True, "unit": "px"}), ("dimension", {"value": 1, "unit": "em"}),
                 ("duration", {"value": -1, "unit": "ms"}), ("fontWeight", 1001),
                 ("fontFamily", []), ("cubicBezier", [2, 0, 0, 1]), ("number", False)]
        for kind, value in cases:
            with self.subTest(kind=kind, value=value):
                with self.assertRaises(TokenError):
                    Resolver({"primitive.test": token(kind, value, "Invalid value.")}).all()

    def test_literal_nested_shadow_arrays_fail(self):
        layers = Project(self.root).compose("night").values["semantic.shadow.shadow-sm"]
        layer = self.layer("override.json", "semantic.shadow.shadow-sm", "shadow", [layers])
        with self.assertRaisesRegex(TokenError, "literal nested arrays"):
            Project(self.root).compose("night", overrides=[layer])

    def test_example_layers_build_in_every_mode(self):
        project = Project(self.root)
        for mode in ("night", "day", "aurora"):
            with self.subTest(mode=mode):
                composition = project.compose(mode, ROOT / "examples/themes/soft-windows.tokens.json",
                                              [ROOT / "examples/overrides/compact-spacing.tokens.json"])
                self.assertGreater(check_contrast(composition), 100)

    def test_invalid_structures_and_names_are_rejected(self):
        for document in ({"bad.name": token("number", 1, "Bad name.")},
                         {"x": {"$type": []}}, {"x": {"$ref": "#/other"}},
                         {"x": dict(token("number", 1, "Bad node."), child={})}):
            with self.subTest(document=document):
                with self.assertRaises(TokenError):
                    flatten(document)

    def test_duplicate_json_keys_fail(self):
        path = self.root / "duplicate.json"
        path.write_text('{"color": {}, "color": {}}')
        with self.assertRaisesRegex(TokenError, "duplicate JSON key"):
            read_json(path)

    def test_nonfinite_and_malformed_json_fail(self):
        path = self.root / "invalid.json"
        for content in ('{"value": NaN}', '{"value": Infinity}', '{"value": -Infinity}', '{"value":'):
            with self.subTest(content=content):
                path.write_text(content, encoding="utf-8")
                with self.assertRaises(TokenError):
                    read_json(path)

    def test_design_metadata_describes_current_assets(self):
        info = read_json(self.root / "tokens/base.tokens.json")["$extensions"][EXTENSION]
        self.assertEqual(info["name"], "Fjord")
        self.assertEqual(set(info["designSeeds"]), {"primary", "accents"})
        self.assertEqual(info["fontAssets"], "assets/fonts/manifest.json")

    def test_group_types_and_deprecation_are_inherited(self):
        tokens = flatten({"primitive": {"$type": "number", "$deprecated": "Use the new role.",
                                       "gap": {"$value": 1, "$description": "Group spacing."}}})
        self.assertEqual(tokens["primitive.gap"]["$type"], "number")
        self.assertEqual(tokens["primitive.gap"]["$deprecated"], "Use the new role.")

    def test_deprecation_alias_stays_usable_and_reports_warning(self):
        self.patch("tokens/base.tokens.json", "semantic.spacing.deprecated-gap", {
            "$type": "dimension", "$value": "{semantic.spacing.space-4}",
            "$description": "Deprecated gap.", "$deprecated": "Use {semantic.spacing.space-4}."})
        report = validate_project(self.root)
        self.assertEqual(len(report.deprecated), 3)
        self.assertEqual(Project(self.root).compose("day").values["semantic.spacing.deprecated-gap"]["value"], 16)

    def test_unknown_schema_or_contract_version_fails(self):
        path = self.root / "tokens/contract.json"
        original = read_json(path)
        for field, value in (("schemaVersion", 3), ("version", "1.0.0")):
            with self.subTest(field=field):
                changed = dict(original, **{field: value})
                path.write_text(json.dumps(changed))
                with self.assertRaises(TokenError):
                    Project(self.root)

    def test_layers_require_compatible_metadata(self):
        layer = self.layer("override.json", "semantic.spacing.space-4", "dimension", {"value": 8, "unit": "px"})
        document = read_json(layer)
        document["$extensions"][EXTENSION]["contractVersion"] = "0.4.0"
        layer.write_text(json.dumps(document))
        with self.assertRaisesRegex(TokenError, "unsupported contract version"):
            Project(self.root).compose("night", overrides=[layer])

    def test_contract_cannot_remove_required_roles(self):
        path = self.root / "tokens/contract.json"
        data = read_json(path)
        del data["required"]["semantic.color.ink"]
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(TokenError, "required role semantic.color.ink"):
            Project(self.root)

    def test_cli_build_is_valid_dtcg_and_repeatable(self):
        output = self.root / "dist/day.tokens.json"
        result = self.cli("build", "--root", self.root, "--mode", "day")
        self.assertEqual(result.returncode, 0, result.stderr)
        before = output.read_bytes()
        document = read_json(output)
        values = Resolver(flatten(document)).all()
        self.assertEqual(hex_color(values["semantic.color.ink"]), "#16232f")
        self.assertEqual(self.cli("build", "--root", self.root, "--mode", "day").returncode, 0)
        self.assertEqual(output.read_bytes(), before)

    def test_build_will_not_overwrite_sources(self):
        output = self.root / "tokens/base.tokens.json"
        before = output.read_bytes()
        result = self.cli("build", "--root", self.root, "--output", output)
        self.assertEqual(result.returncode, 1)
        self.assertIn("must not replace token sources", result.stderr)
        self.assertEqual(output.read_bytes(), before)

    def test_build_failure_leaves_existing_output_intact(self):
        output = self.root / "existing.json"
        output.write_text("keep")
        layer = self.layer("bad.json", "semantic.color.ink", "color", color_from_hex("#111920"))
        result = self.cli("build", "--root", self.root, "--override", layer, "--output", output)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(output.read_text(), "keep")
        self.assertNotIn("Traceback", result.stderr)

    def test_cli_help_and_invalid_arguments(self):
        self.assertEqual(self.cli("--help").returncode, 0)
        self.assertEqual(self.cli("build", "--mode", "wrong").returncode, 2)
        self.assertEqual(self.cli("build", "--mod", "night").returncode, 2)

    def test_cli_default_paths_work_outside_project(self):
        result = self.cli("validate")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("PASS:", result.stdout)

    def test_cli_reports_missing_sources_without_tracebacks(self):
        for relative in ("tokens/modes/day.tokens.json", "tokens/usage.json"):
            with self.subTest(file=relative):
                path = self.root / relative
                saved = path.read_bytes()
                path.unlink()
                try:
                    result = self.cli("validate", "--root", self.root)
                    self.assertEqual(result.returncode, 1)
                    self.assertIn(relative, result.stderr)
                    self.assertNotIn("Traceback", result.stderr)
                finally:
                    path.write_bytes(saved)


if __name__ == "__main__":
    unittest.main()
