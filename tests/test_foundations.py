import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from fjord_tokens.dtcg import TokenError, nest
from fjord_tokens.foundations import foundation_bundle, foundation_report
from fjord_tokens.icons import load_icons
from fjord_tokens.project import Project, metadata


ROOT = Path(__file__).resolve().parents[1]


class FoundationTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="fjord-foundations-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for name in ("tokens", "assets/icons", "reference/svelte/src/lib"):
            shutil.copytree(ROOT / name, self.root / name)
        (self.root / "docs").mkdir()
        shutil.copyfile(ROOT / "docs/platforms.md", self.root / "docs/platforms.md")

    def layer(self, path, kind, value, name="override.json"):
        output = self.root / name
        output.write_text(json.dumps(nest({path: {"$type": kind, "$value": value, "$description": "Layout regression fixture."}}, metadata())))
        return output

    def test_invalid_interaction_and_layout_values_are_rejected(self):
        for path, kind, value, message in (
            ("semantic.size.target-minimum", "dimension", {"value": 23, "unit": "px"}, "at least 24"),
            ("semantic.size.target-touch", "dimension", {"value": 43, "unit": "px"}, "at least 44"),
            ("semantic.size.target-minimum", "dimension", {"value": 40, "unit": "px"}, "declared target-minimum"),
            ("semantic.size.icon-control", "dimension", {"value": 0, "unit": "px"}, "must be positive"),
            ("semantic.breakpoint.breakpoint-md", "dimension", {"value": 600, "unit": "px"}, "increasing"),
            ("semantic.density.gap-compact", "dimension", {"value": 20, "unit": "px"}, "must not exceed"),
            ("semantic.layout.page-gutter", "dimension", {"value": -1, "unit": "px"}, "must not be negative"),
            ("semantic.border-width.border-focus", "dimension", {"value": 1, "unit": "px"}, "at least 2"),
            ("component.button.primary.focus-width", "dimension", "{semantic.border-width.border-hairline}", "at least 2"),
            ("semantic.z-index.z-popover", "number", 61, "base < sticky"),
            ("semantic.duration.duration-slow", "duration", {"value": 301, "unit": "ms"}, "slow <= 300ms"),
            ("semantic.duration.duration-none", "duration", {"value": 1, "unit": "ms"}, "none = 0"),
        ):
            with self.subTest(path=path):
                layer = self.layer(path, kind, value)
                with self.assertRaisesRegex(TokenError, message):
                    Project(self.root).compose("night", overrides=[layer])

    def test_invalid_intermediate_target_cannot_be_hidden(self):
        first = self.layer("semantic.size.target-touch", "dimension", {"value": 30, "unit": "px"}, "first.json")
        last = self.layer("semantic.size.target-touch", "dimension", {"value": 44, "unit": "px"}, "last.json")
        with self.assertRaisesRegex(TokenError, "at least 44"):
            Project(self.root).compose("day", overrides=[first, last])

    def test_equivalent_rem_and_seconds_are_supported(self):
        target = self.layer("semantic.size.target-touch", "dimension", {"value": 2.75, "unit": "rem"})
        duration = self.layer("semantic.duration.duration-slow", "duration", {"value": .26, "unit": "s"}, "time.json")
        Project(self.root).compose("aurora", overrides=[target, duration])

    def test_pinned_icon_bytes_and_original_licence_are_required(self):
        manifest, bodies = load_icons(self.root)
        self.assertEqual(len(bodies), 16)
        self.assertEqual(manifest["version"], "2.1.1")
        path = self.root / "assets/icons/info.svg"
        path.write_bytes(path.read_bytes() + b"changed")
        with self.assertRaisesRegex(TokenError, "SVG hash differs"):
            load_icons(self.root)

    def test_active_svg_content_is_rejected_even_if_hash_is_updated(self):
        path = self.root / "assets/icons/info.svg"
        data = path.read_text().replace("</svg>", '<script>alert(1)</script></svg>').encode()
        path.write_bytes(data)
        manifest_path = self.root / "assets/icons/manifest.json"
        manifest = json.loads(manifest_path.read_text())
        next(item for item in manifest["icons"] if item["name"] == "info")["sha256"] = hashlib.sha256(data).hexdigest()
        manifest_path.write_text(json.dumps(manifest))
        with self.assertRaisesRegex(TokenError, "only static filled paths"):
            load_icons(self.root)

    def test_licence_and_generated_icon_data_cannot_drift(self):
        path = self.root / "assets/icons/LICENSE"
        original = path.read_bytes()
        path.write_bytes(b"MIT")
        with self.assertRaisesRegex(TokenError, "original MIT licence"):
            load_icons(self.root)
        path.write_bytes(original)
        (self.root / "reference/svelte/src/lib/icons.generated.ts").write_text("changed")
        with self.assertRaisesRegex(TokenError, "Icon data differs"):
            foundation_report(self.root)

    def test_bundle_is_repeatable_and_preserves_both_icon_accessibility_modes(self):
        first = foundation_bundle(self.root)
        self.assertEqual(first, foundation_bundle(self.root))
        html = first["foundations.html"].decode()
        self.assertNotIn("{{", html)
        self.assertIn('aria-label="Information"', html)
        self.assertIn('aria-label="Dismiss sample notification"', html)
        self.assertIn('aria-hidden="true"', html)
        self.assertEqual(len([name for name in first if name.endswith(".svg")]), 16)

    def test_check_writes_nothing_and_build_protects_source_symlinks(self):
        command = [sys.executable, "-m", "fjord_tokens", "foundations", "--root", str(self.root)]
        result = subprocess.run([*command, "--check"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse((self.root / "dist").exists())
        target = self.root / "tokens/base.tokens.json"
        before = target.read_bytes()
        (self.root / "dist").mkdir()
        (self.root / "dist/foundations.css").symlink_to(target)
        result = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(result.returncode, 1)
        self.assertNotIn("Traceback", result.stderr)
        self.assertEqual(target.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
