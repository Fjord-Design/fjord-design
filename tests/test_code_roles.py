import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from fjord_tokens.code_roles import MAIN_SYNTAX, ROLES, check_syntax_distances, oklab, syntax_distances, validate_bindings
from fjord_tokens.contract import ANSI, THEMES
from fjord_tokens.dtcg import TokenError, color_from_hex, nest
from fjord_tokens.preview import check_samples, load_samples, render_preview, render_samples
from fjord_tokens.project import Project, check_contrast, metadata


ROOT = Path(__file__).resolve().parents[1]


class CodeRoleTests(unittest.TestCase):
    def test_every_binding_resolves_in_every_mode(self):
        project = Project(ROOT)
        for mode in THEMES:
            with self.subTest(mode=mode):
                composed = project.compose(mode)
                validate_bindings(composed)
                self.assertGreater(check_contrast(composed), 100)

    def test_binding_without_a_colour_source_fails(self):
        composed = Project(ROOT).compose("night")
        del composed.tokens["semantic.color.git-added"]
        with self.assertRaisesRegex(TokenError, "expected colour token semantic.color.git-added"):
            validate_bindings(composed)

    def test_status_and_diff_roles_have_distinct_text_markers(self):
        for prefix, count in (("git-", 7), ("diagnostic-", 4)):
            markers = [role.marker for name, role in ROLES.items() if name.startswith(prefix)]
            self.assertEqual(len(markers), count)
            self.assertEqual(len(set(markers)), count)
            self.assertTrue(all(marker.strip() for marker in markers))
        self.assertEqual([ROLES[f"diff-{state}"].marker[0] for state in ("added", "changed", "deleted")], ["+", "~", "-"])

    def test_all_ansi_slots_are_bound_in_order(self):
        self.assertEqual([ROLES[name].token for name in ANSI], [f"semantic.color.{name}" for name in ANSI])
        self.assertEqual(ANSI[0], "ansi-black")
        self.assertEqual(ANSI[8], "ansi-bright-black")
        self.assertEqual(ANSI[15], "ansi-bright-white")

    def test_oklab_reference_red(self):
        expected = (0.6279553606, 0.2248630611, 0.1258462985)
        for actual, reference in zip(oklab(color_from_hex("#ff0000")), expected):
            self.assertAlmostEqual(actual, reference, places=8)

    def test_syntax_distance_thresholds_pass_every_mode(self):
        for mode in THEMES:
            composed = Project(ROOT).compose(mode)
            self.assertEqual(len(list(syntax_distances(composed))), 36)
            check_syntax_distances(composed)

    def test_readable_but_indistinguishable_syntax_fails(self):
        composed = Project(ROOT).compose("night")
        composed.values["semantic.color.syn-type"] = composed.values["semantic.color.syn-keyword"]
        with self.assertRaisesRegex(TokenError, "OKLab distance .* is below 7"):
            check_contrast(composed)

    def test_all_example_spans_pass_on_their_actual_surfaces(self):
        samples = load_samples(ROOT)
        seen = {role for sample in samples for line in sample["lines"] for role, _ in line["spans"]}
        self.assertTrue({f"syntax-{name}" for name in MAIN_SYNTAX} <= seen)
        for mode in THEMES:
            self.assertGreater(check_samples(samples, Project(ROOT).compose(mode)), 100)

    def test_low_contrast_text_on_a_diff_surface_fails(self):
        samples = load_samples(ROOT)
        diff = next(sample for sample in samples if sample["id"] == "diff")
        diff["lines"][1]["spans"][0][0] = "terminal-hint"
        with self.assertRaisesRegex(TokenError, "diff:2 terminal-hint.*below 4.5"):
            check_samples(samples, Project(ROOT).compose("night"))

    def test_json_and_shell_examples_are_valid_source(self):
        examples = {sample["id"]: "\n".join("".join(text for _, text in line["spans"]) for line in sample["lines"])
                    for sample in load_samples(ROOT)}
        self.assertEqual(json.loads(examples["json"])["mode"], "night")
        result = subprocess.run(["sh", "-n"], input=examples["shell"], text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_specimen_content_is_escaped_as_text(self):
        samples = load_samples(ROOT)
        samples[0]["title"] = '<script>alert("title")</script>'
        samples[0]["lines"][0]["spans"][0][1] = '<img src=x onerror="alert(1)"> {{styles}}'
        html = render_samples(samples)
        self.assertNotIn("<script>", html)
        self.assertNotIn("<img", html)
        self.assertIn("&lt;script&gt;", html)
        self.assertIn("&lt;img", html)


class PreviewTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="fjord-preview-test-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for directory in ("tokens", "docs", "examples", "assets/fonts"):
            shutil.copytree(ROOT / directory, self.root / directory)

    def cli(self, *arguments):
        return subprocess.run([sys.executable, "-m", "fjord_tokens", "preview", "--root", str(self.root), *map(str, arguments)],
                              capture_output=True, text=True)

    def test_preview_is_repeatable_and_includes_every_mode(self):
        html = render_preview(self.root)
        self.assertEqual(html, render_preview(self.root))
        for mode in THEMES:
            self.assertIn(f'html[data-theme="{mode}"]', html)
            self.assertIn(f'data-mode="{mode}"', html)
        for sample in load_samples(self.root):
            self.assertIn(f'heading-{sample["id"]}', html)
        self.assertNotIn("{{styles}}", html)
        self.assertNotIn("{{samples}}", html)

    def test_unknown_or_missing_example_is_rejected(self):
        path = self.root / "examples/code/samples.json"
        samples = load_samples(self.root)
        samples[0]["lines"][0]["spans"][0][0] = "syntax-typo"
        path.write_text(json.dumps(samples))
        with self.assertRaisesRegex(TokenError, "unknown or non-text role"):
            render_preview(self.root)
        path.write_text(json.dumps(samples[1:]))
        with self.assertRaisesRegex(TokenError, "supply one example"):
            load_samples(self.root)

    def test_invalid_examples_do_not_replace_output(self):
        output = self.root / "existing.html"
        output.write_text("keep")
        (self.root / "examples/code/samples.json").write_text("[]")
        result = self.cli("--output", output)
        self.assertEqual(result.returncode, 1)
        self.assertNotIn("Traceback", result.stderr)
        self.assertEqual(output.read_text(), "keep")

    def test_preview_cannot_replace_examples(self):
        path = self.root / "examples/code/samples.json"
        before = path.read_bytes()
        result = self.cli("--output", path)
        self.assertEqual(result.returncode, 1)
        self.assertIn("must not replace token sources", result.stderr)
        self.assertEqual(path.read_bytes(), before)

    def test_cli_writes_offline_html(self):
        result = self.cli()
        self.assertEqual(result.returncode, 0, result.stderr)
        html = (self.root / "dist/code-and-terminal.html").read_text()
        self.assertTrue(html.startswith("<!doctype html>"))
        self.assertNotIn("<script src=", html)
        self.assertIn('<link rel="stylesheet" href="fonts/typography.css">', html)
        self.assertTrue((self.root / "dist/fonts/typography.css").is_file())
        self.assertEqual((self.root / "dist/fonts/cascadia-code/CascadiaCode.woff2").read_bytes(),
                         (self.root / "assets/fonts/cascadia-code/CascadiaCode.woff2").read_bytes())
        self.assertNotIn("https://", html)

    def test_preview_applies_overrides_in_every_mode(self):
        layer = self.root / "accent.tokens.json"
        layer.write_text(json.dumps(nest({"semantic.color.accent": {
            "$type": "color", "$value": "{semantic.color.info}", "$description": "Use the info accent."
        }}, metadata())))
        result = self.cli("--override", layer)
        self.assertEqual(result.returncode, 0, result.stderr)
        html = (self.root / "dist/code-and-terminal.html").read_text()
        for color in ("#92c1e3", "#215e92", "#93c9ec"):
            self.assertIn(f"--accent: {color};", html)

    def test_preview_rejects_an_override_that_fails_in_one_mode(self):
        layer = self.root / "accent.tokens.json"
        layer.write_text(json.dumps(nest({"semantic.color.accent": {
            "$type": "color", "$value": color_from_hex("#90c6e3"), "$description": "A colour for a dark surface."
        }}, metadata())))
        output = self.root / "preview.html"
        output.write_text("keep")
        result = self.cli("--override", layer, "--output", output)
        self.assertEqual(result.returncode, 1)
        self.assertIn("[day]", result.stderr)
        self.assertEqual(output.read_text(), "keep")


if __name__ == "__main__":
    unittest.main()
