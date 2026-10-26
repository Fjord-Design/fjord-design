import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from fjord_tokens.dtcg import TokenError, nest
from fjord_tokens.project import Project, check_contrast, metadata, validate_project
from fjord_tokens.semantics import semantic_bundle, semantic_report


ROOT = Path(__file__).resolve().parents[1]


class SemanticTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="fjord-semantics-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for name in ("tokens", "examples/themes", "reference/svelte/src/lib"):
            shutil.copytree(ROOT / name, self.root / name)
        (self.root / "docs").mkdir()
        shutil.copyfile(ROOT / "docs/platforms.md", self.root / "docs/platforms.md")

    def usage(self, change):
        path = self.root / "tokens/usage.json"
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data))

    def test_all_identities_modes_states_and_component_aliases_are_retained(self):
        report = semantic_report(self.root)
        self.assertEqual(len(report["usage"]["roles"]), 34)
        self.assertEqual(len(report["usage"]["states"]), 8)
        self.assertEqual(set(report["identities"]), {"fjord", "fern"})
        for identity, modes in report["identities"].items():
            self.assertEqual(set(modes), {"night", "day", "aurora"})
            for mode, data in modes.items():
                self.assertEqual(len(data["colors"]), 81)
                for pair in data["pairs"]:
                    self.assertGreaterEqual(pair["sourceRatio"], pair["minimum"])
                    self.assertGreaterEqual(pair["hexRatio"], pair["minimum"])
                if identity == "fern":
                    self.assertNotEqual(data["colors"]["accent"]["hex"], report["identities"]["fjord"][mode]["colors"]["accent"]["hex"])

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

    def test_semantic_bundle_is_deterministic_and_local(self):
        first = semantic_bundle(self.root)
        self.assertEqual(first, semantic_bundle(self.root))
        self.assertNotIn(b"{{", first["semantics.html"])
        self.assertNotIn(b"https://", first["semantics.html"])
        self.assertIn(b"prefers-color-scheme:light", first["semantics.css"])
        self.assertEqual(validate_project(self.root).contrast_checks, 430)

    def test_check_writes_nothing_and_output_cannot_replace_source(self):
        command = [sys.executable, "-m", "fjord_tokens", "semantics", "--root", str(self.root)]
        checked = subprocess.run([*command, "--check"], capture_output=True, text=True)
        self.assertEqual(checked.returncode, 0, checked.stderr)
        self.assertFalse((self.root / "dist").exists())
        target = self.root / "tokens/usage.json"
        original = target.read_bytes()
        (self.root / "dist").mkdir()
        (self.root / "dist/semantics.css").symlink_to(target)
        built = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(built.returncode, 1, built.stderr)
        self.assertNotIn("Traceback", built.stderr)
        self.assertEqual(target.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
