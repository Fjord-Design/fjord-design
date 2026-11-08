import json
from pathlib import Path
import shutil
import tempfile
import unittest

from fjord_tokens.dtcg import TokenError, nest
from fjord_tokens.project import Project, check_contrast, metadata, validate_project


ROOT = Path(__file__).resolve().parents[1]


class UsageTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="fjord-semantics-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for name in ("tokens", "examples/themes", "examples/overrides"):
            shutil.copytree(ROOT / name, self.root / name)

    def usage(self, change):
        path = self.root / "tokens/usage.json"
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data))


    def test_example_themes_and_overrides_preserve_shared_design_contract(self):
        project = Project(self.root)
        self.assertEqual(len(project.usage["roles"]), 34)
        self.assertEqual(len(project.usage["states"]), 8)
        self.assertEqual(validate_project(self.root).contrast_checks, 430)
        themes = [None, *sorted((self.root / "examples/themes").glob("*.tokens.json"))]
        overrides = sorted((self.root / "examples/overrides").glob("*.tokens.json"))
        for mode in ("night", "day", "aurora"):
            original = project.compose(mode)
            bindings = {path: token for path, token in original.tokens.items() if path.startswith("component.")}
            for theme in themes:
                for layers in ([], overrides):
                    with self.subTest(mode=mode, theme=theme, overrides=layers):
                        composition = project.compose(mode, theme, layers)
                        self.assertEqual(original.tokens.keys(), composition.tokens.keys())
                        self.assertEqual(bindings, {path: token for path, token in composition.tokens.items()
                                                    if path.startswith("component.")})
                        self.assertGreater(check_contrast(composition), 100)

    def test_missing_state_or_role_is_rejected(self):
        self.usage(lambda data: data["states"].pop("loading"))
        with self.assertRaisesRegex(TokenError, "retain all"):
            Project(self.root)

    def test_help_cannot_use_disabled_text(self):
        self.usage(lambda data: data["roles"]["text-help"].update(token="semantic.color.ink-disabled"))
        with self.assertRaisesRegex(TokenError, "disabled text must not be used"):
            Project(self.root)

    def test_help_cannot_be_marked_decorative_to_bypass_contrast(self):
        self.usage(lambda data: data["roles"]["text-help"].update(kind="decorative", minimumContrast=None))
        with self.assertRaisesRegex(TokenError, "preserve its text role"):
            Project(self.root)

    def test_contrast_threshold_cannot_be_weakened(self):
        self.usage(lambda data: data["states"]["loading"].update(minimumContrast=3))
        with self.assertRaisesRegex(TokenError, "4.5:1"):
            Project(self.root)

    def test_unknown_or_primitive_usage_reference_is_rejected(self):
        self.usage(lambda data: data["roles"]["canvas"].update(token="primitive.color.night.bg-base"))
        with self.assertRaisesRegex(TokenError, "existing semantic or component"):
            Project(self.root).compose("night")

    def test_help_alias_cannot_rebind_to_disabled_text(self):
        path = self.root / "override.json"
        path.write_text(json.dumps(nest({"semantic.color.ink-muted": {
            "$type": "color", "$value": "{semantic.color.ink-disabled}", "$description": "Invalid help rebinding."}}, metadata())))
        with self.assertRaisesRegex(TokenError, "must not alias disabled text"):
            Project(self.root).compose("night", overrides=[path])

    def test_opaque_surface_policy_rejects_transparency(self):
        composition = Project(self.root).compose("night")
        composition.values["semantic.color.bg-surface"]["alpha"] = .5
        with self.assertRaisesRegex(TokenError, "alpha cannot be ignored"):
            check_contrast(composition)

    def test_incomplete_mode_has_no_dark_fallback(self):
        path = self.root / "tokens/modes/day.tokens.json"
        data = json.loads(path.read_text())
        del data["semantic"]["color"]["accent"]
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(TokenError, "missing mode roles"):
            Project(self.root).compose("day")


if __name__ == "__main__":
    unittest.main()
