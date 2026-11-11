import hashlib
import json
from pathlib import Path
import shutil

import pytest

from fjord.dtcg import TokenError, nest
from fjord.icons import load_icons
from fjord.project import Project, metadata


ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture
def layout_root(tmp_path):
    for name in ("tokens", "assets/icons"):
        shutil.copytree(ROOT / name, tmp_path / name)
    return tmp_path


def layer(root, path, kind, value, name="override.json"):
    output = root / name
    output.write_text(json.dumps(nest({path: {"$type": kind, "$value": value, "$description": "Layout regression fixture."}}, metadata())))
    return output


@pytest.mark.parametrize("path, kind, value, message", (
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
))
def test_invalid_interaction_and_layout_values_are_rejected(layout_root, path, kind, value, message):
    override = layer(layout_root, path, kind, value)
    with pytest.raises(TokenError, match=message):
        Project(layout_root).compose("night", overrides=[override])


def test_invalid_intermediate_target_cannot_be_hidden(layout_root):
    first = layer(layout_root, "semantic.size.target-touch", "dimension", {"value": 30, "unit": "px"}, "first.json")
    last = layer(layout_root, "semantic.size.target-touch", "dimension", {"value": 44, "unit": "px"}, "last.json")
    with pytest.raises(TokenError, match="at least 44"):
        Project(layout_root).compose("day", overrides=[first, last])


def test_equivalent_rem_and_seconds_are_supported(layout_root):
    target = layer(layout_root, "semantic.size.target-touch", "dimension", {"value": 2.75, "unit": "rem"})
    duration = layer(layout_root, "semantic.duration.duration-slow", "duration", {"value": .26, "unit": "s"}, "time.json")
    Project(layout_root).compose("aurora", overrides=[target, duration])


def test_pinned_icon_bytes_and_original_licence_are_required(layout_root):
    manifest = load_icons(layout_root)
    assert len(manifest["icons"]) == 16
    assert manifest["version"] == "2.1.1"
    path = layout_root / "assets/icons/info.svg"
    path.write_bytes(path.read_bytes() + b"changed")
    with pytest.raises(TokenError, match="SVG hash differs"):
        load_icons(layout_root)


def test_original_icon_licence_cannot_drift(layout_root):
    (layout_root / "assets/icons/LICENSE").write_bytes(b"MIT")
    with pytest.raises(TokenError, match="original MIT licence"):
        load_icons(layout_root)


def test_icon_pointer_follows_the_contract_base(layout_root):
    (layout_root / "tokens/base.tokens.json").rename(layout_root / "tokens/source.tokens.json")
    contract = layout_root / "tokens/contract.json"
    data = json.loads(contract.read_text())
    data["base"] = "source.tokens.json"
    contract.write_text(json.dumps(data))
    load_icons(layout_root)
    source = layout_root / "tokens/source.tokens.json"
    data = json.loads(source.read_text())
    data["$extensions"]["org.xerrion.fjord"]["iconAssets"] = "elsewhere.json"
    source.write_text(json.dumps(data))
    with pytest.raises(TokenError, match="iconAssets"):
        load_icons(layout_root)


def test_active_svg_content_is_rejected_even_if_hash_is_updated(layout_root):
    path = layout_root / "assets/icons/info.svg"
    data = path.read_text().replace("</svg>", '<script>alert(1)</script></svg>').encode()
    path.write_bytes(data)
    manifest_path = layout_root / "assets/icons/manifest.json"
    manifest = json.loads(manifest_path.read_text())
    next(item for item in manifest["icons"] if item["name"] == "info")["sha256"] = hashlib.sha256(data).hexdigest()
    manifest_path.write_text(json.dumps(manifest))
    with pytest.raises(TokenError, match="only static filled paths"):
        load_icons(layout_root)
