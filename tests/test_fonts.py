import hashlib
import json
from pathlib import Path
import shutil

import pytest

from fjord.dtcg import EXTENSION, TokenError
from fjord.fonts import ROLES, asset_path, load_fonts


ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture
def font_root(tmp_path):
    shutil.copytree(ROOT / "tokens", tmp_path / "tokens")
    shutil.copytree(ROOT / "assets/fonts", tmp_path / "assets/fonts")
    return tmp_path


def edit_json(root, name, change):
    path = root / name
    data = json.loads(path.read_text())
    change(data)
    path.write_text(json.dumps(data))


def test_pinned_assets_cover_roles_and_keep_original_licences(font_root):
    manifest = load_fonts(font_root)
    assert sum(len(family["faces"]) for family in manifest["families"]) == 9
    assert len(manifest["families"]) == 5
    assert len(ROLES) == 15
    assert all(family["license"] == "OFL-1.1" for family in manifest["families"])


def test_typography_must_use_a_shared_semantic_font_stack(font_root):
    edit_json(font_root, "tokens/base.tokens.json", lambda data: data["semantic"]["typography"]["code"]["$value"].update(
        fontFamily=["Cascadia Code", "Unexpected fallback"]))
    with pytest.raises(TokenError, match="shared semantic font stacks"):
        load_fonts(font_root)


def test_corrupt_font_is_rejected(font_root):
    path = font_root / "assets/fonts/cascadia-code/CascadiaCode.woff2"
    path.write_bytes(path.read_bytes() + b"changed")
    with pytest.raises(TokenError, match="hash changed"):
        load_fonts(font_root)


def test_missing_licence_is_rejected(font_root):
    (font_root / "assets/fonts/fira-code/OFL.txt").unlink()
    with pytest.raises(FileNotFoundError):
        load_fonts(font_root)


def test_stale_or_incomplete_inspection_is_rejected(font_root):
    edit_json(font_root, "assets/fonts/inspection.json", lambda data: data["faces"][0].update(missingProbeCodepoints=["U+00E6"]))
    with pytest.raises(TokenError, match="inspection is missing or stale"):
        load_fonts(font_root)


def test_unsupported_asset_weight_is_rejected(font_root):
    edit_json(font_root, "assets/fonts/manifest.json", lambda data: data["families"][0]["faces"][0].update(weight=[100, 1000]))
    with pytest.raises(TokenError, match="inspection is missing or stale"):
        load_fonts(font_root)


def test_typography_cannot_request_an_unavailable_primary_weight(font_root):
    edit_json(font_root, "tokens/base.tokens.json", lambda data: data["semantic"]["typography"]["code"]["$value"].update(fontWeight=800))
    with pytest.raises(TokenError, match="no bundled primary face"):
        load_fonts(font_root)


def test_asset_paths_cannot_escape_the_font_directory(font_root):
    with pytest.raises(TokenError, match="must stay inside"):
        asset_path(font_root / "assets/fonts", "../../tokens/base.tokens.json")


def test_symlinks_cannot_escape_the_font_directory(font_root):
    link = font_root / "assets/fonts/escape"
    link.symlink_to(font_root / "tokens", target_is_directory=True)
    with pytest.raises(TokenError, match="must stay inside"):
        asset_path(font_root / "assets/fonts", "escape/base.tokens.json")


def test_invalid_woff_header_is_rejected_even_with_updated_hash(font_root):
    path = font_root / "assets/fonts/cascadia-code/CascadiaCode.woff2"
    path.write_bytes(b"bad!" + path.read_bytes()[4:])
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    edit_json(font_root, "assets/fonts/manifest.json", lambda data: data["families"][3]["faces"][0].update(sha256=digest))
    with pytest.raises(TokenError, match="invalid WOFF2 signature"):
        load_fonts(font_root)


def test_inspection_must_retain_danish_and_confusable_probe(font_root):
    edit_json(font_root, "assets/fonts/inspection.json", lambda data: data.update(probe="abc"))
    with pytest.raises(TokenError, match="glyph probe"):
        load_fonts(font_root)


def test_manifest_must_retain_styles_and_families(font_root):
    edit_json(font_root, "assets/fonts/manifest.json", lambda data: data["families"][3]["faces"].pop())
    with pytest.raises(TokenError, match="expected styles"):
        load_fonts(font_root)


def test_font_pointer_follows_contract_base(font_root):
    (font_root / "tokens/base.tokens.json").rename(font_root / "tokens/source.tokens.json")
    edit_json(font_root, "tokens/contract.json", lambda data: data.update(base="source.tokens.json"))
    load_fonts(font_root)
    edit_json(font_root, "tokens/source.tokens.json", lambda data: data["$extensions"][EXTENSION].update(fontAssets="elsewhere.json"))
    with pytest.raises(TokenError, match="fontAssets"):
        load_fonts(font_root)
