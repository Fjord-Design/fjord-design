import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from fjord_tokens.dtcg import EXTENSION, TokenError
from fjord_tokens.fonts import ROLES, asset_path, load_fonts


ROOT = Path(__file__).resolve().parents[1]


class FontTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(prefix="fjord-fonts-")
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        shutil.copytree(ROOT / "tokens", self.root / "tokens")
        shutil.copytree(ROOT / "assets/fonts", self.root / "assets/fonts")

    def edit(self, name, change):
        path = self.root / name
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data))


    def test_pinned_assets_cover_roles_and_keep_original_licences(self):
        manifest = load_fonts(self.root)
        self.assertEqual(sum(len(family["faces"]) for family in manifest["families"]), 9)
        self.assertEqual(len(manifest["families"]), 5)
        self.assertEqual(len(ROLES), 15)
        self.assertTrue(all(family["license"] == "OFL-1.1" for family in manifest["families"]))

    def test_typography_must_use_a_shared_semantic_font_stack(self):
        self.edit("tokens/base.tokens.json", lambda data: data["semantic"]["typography"]["type-code"]["$value"].update(
            fontFamily=["Cascadia Code", "Unexpected fallback"]))
        with self.assertRaisesRegex(TokenError, "shared semantic font stacks"):
            load_fonts(self.root)

    def test_corrupt_font_is_rejected(self):
        path = self.root / "assets/fonts/cascadia-code/CascadiaCode.woff2"
        path.write_bytes(path.read_bytes() + b"changed")
        with self.assertRaisesRegex(TokenError, "hash changed"):
            load_fonts(self.root)

    def test_missing_licence_is_rejected(self):
        (self.root / "assets/fonts/fira-code/OFL.txt").unlink()
        with self.assertRaises(FileNotFoundError):
            load_fonts(self.root)

    def test_stale_or_incomplete_inspection_is_rejected(self):
        self.edit("assets/fonts/inspection.json", lambda data: data["faces"][0].update(missingProbeCodepoints=["U+00E6"]))
        with self.assertRaisesRegex(TokenError, "inspection is missing or stale"):
            load_fonts(self.root)

    def test_unsupported_asset_weight_is_rejected(self):
        self.edit("assets/fonts/manifest.json", lambda data: data["families"][0]["faces"][0].update(weight=[100, 1000]))
        with self.assertRaisesRegex(TokenError, "inspection is missing or stale"):
            load_fonts(self.root)

    def test_typography_cannot_request_an_unavailable_primary_weight(self):
        self.edit("tokens/base.tokens.json", lambda data: data["semantic"]["typography"]["type-code"]["$value"].update(fontWeight=800))
        with self.assertRaisesRegex(TokenError, "no bundled primary face"):
            load_fonts(self.root)

    def test_asset_paths_cannot_escape_the_font_directory(self):
        with self.assertRaisesRegex(TokenError, "must stay inside"):
            asset_path(self.root / "assets/fonts", "../../tokens/base.tokens.json")
        link = self.root / "assets/fonts/escape"
        link.symlink_to(self.root / "tokens", target_is_directory=True)
        with self.assertRaisesRegex(TokenError, "must stay inside"):
            asset_path(self.root / "assets/fonts", "escape/base.tokens.json")

    def test_invalid_woff_header_is_rejected_even_with_updated_hash(self):
        path = self.root / "assets/fonts/cascadia-code/CascadiaCode.woff2"
        path.write_bytes(b"bad!" + path.read_bytes()[4:])
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        self.edit("assets/fonts/manifest.json", lambda data: data["families"][3]["faces"][0].update(sha256=digest))
        with self.assertRaisesRegex(TokenError, "invalid WOFF2 signature"):
            load_fonts(self.root)

    def test_inspection_must_retain_danish_and_confusable_probe(self):
        self.edit("assets/fonts/inspection.json", lambda data: data.update(probe="abc"))
        with self.assertRaisesRegex(TokenError, "glyph probe"):
            load_fonts(self.root)

    def test_manifest_must_retain_styles_and_families(self):
        self.edit("assets/fonts/manifest.json", lambda data: data["families"][3]["faces"].pop())
        with self.assertRaisesRegex(TokenError, "expected styles"):
            load_fonts(self.root)

    def test_font_pointer_follows_contract_base(self):
        (self.root / "tokens/base.tokens.json").rename(self.root / "tokens/source.tokens.json")
        self.edit("tokens/contract.json", lambda data: data.update(base="source.tokens.json"))
        load_fonts(self.root)
        self.edit("tokens/source.tokens.json", lambda data: data["$extensions"][EXTENSION].update(fontAssets="elsewhere.json"))
        with self.assertRaisesRegex(TokenError, "fontAssets"):
            load_fonts(self.root)


if __name__ == "__main__":
    unittest.main()
