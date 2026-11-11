import json
from pathlib import Path
import shutil

import pytest

from fjord.color import design_seed_values, in_srgb_gamut, map_oklch_to_srgb, oklch_to_srgb, parse_oklch, srgb_to_oklab, srgb_to_oklch
from fjord.dtcg import EXTENSION, Resolver, TokenError, color_from_hex, flatten, hex_color
from fjord.project import Project, check_contrast, contrast_value, validate_project


ROOT = Path(__file__).resolve().parents[1]
SEEDS = ["oklch(32.9478% 0.034021 247.264)", "oklch(20.8843% 0.018341 245.245)",
         "oklch(34.9975% 0.034292 250.45)", "oklch(76.0906% 0.065221 71.267)",
         "oklch(86.9933% 0.028271 46.055)", "oklch(75.9325% 0.065969 10.27)",
         "oklch(86.9339% 0.02788 129.841)"]


def test_published_red_oklab_vector():
    expected = (0.6279553606, 0.2248630611, 0.1258462985)
    assert srgb_to_oklab(color_from_hex("#ff0000")) == pytest.approx(expected, rel=0, abs=5e-10)


@pytest.mark.parametrize("mode", ("night", "day", "aurora"))
def test_srgb_roundtrip_preserves_approved_palette(mode):
    composition = Project(ROOT).compose(mode)
    for name, value in composition.values.items():
        if composition.tokens[name]["$type"] != "color":
            continue
        converted = map_oklch_to_srgb(srgb_to_oklch(value))
        assert hex_color(converted.color)[:7] == hex_color(value)[:7], name


def test_out_of_gamut_reduces_chroma_and_keeps_lightness_hue():
    value = (0.7, 0.4, 30)
    assert not in_srgb_gamut(oklch_to_srgb(value))
    result = map_oklch_to_srgb(value)
    assert result.mapped
    assert result.chroma < value[1]
    assert all(0 <= v <= 1 for v in result.color["components"])
    lightness, _, hue = srgb_to_oklch(result.color)
    assert lightness == pytest.approx(0.7, rel=0, abs=5e-7)
    assert hue == pytest.approx(30, rel=0, abs=5e-5)
    assert result == map_oklch_to_srgb(value)


@pytest.mark.parametrize("lightness, expected", ((0, "#000000"), (1, "#ffffff")))
def test_endpoints(lightness, expected):
    assert hex_color(map_oklch_to_srgb((lightness, 0.4, 120)).color) == expected


def test_hue_wrap():
    assert map_oklch_to_srgb((0.7, 0.1, -30)) == map_oklch_to_srgb((0.7, 0.1, 330))


@pytest.mark.parametrize("value", (
    (-0.1, 0.1, 30), (1.1, 0.1, 30), (0.5, -0.1, 30), (0.5, 1.1, 30),
    (True, 0.1, 30), (0.5, float("nan"), 30), (0.5, 0.1, float("inf")),
))
def test_invalid_or_nonfinite_coordinates_are_rejected(value):
    with pytest.raises(TokenError):
        map_oklch_to_srgb(value)


@pytest.mark.parametrize("text", (
    "oklch(50% .1 20)-", "oklch(101% 0.1 20)", "oklch(50% NaN 20)", "oklch(50% 0.1 none)",
))
def test_invalid_oklch_text_is_rejected(text):
    with pytest.raises(TokenError):
        parse_oklch(text)


def test_hex_rounds_half_up_and_rejects_inconsistent_hint():
    color = {"colorSpace": "srgb", "components": [0.5, 0.5 / 255, 1], "alpha": 1}
    assert hex_color(color) == "#8001ff"
    color["hex"] = "#ffffff"
    with pytest.raises(TokenError, match="must match the rounded"):
        Resolver(flatten({"primitive": {"color": {"test": {"$type": "color", "$value": color,
                                                           "$description": "Rounding test."}}}})).all()
    color["hex"] = "#8001FF"
    Resolver(flatten({"primitive": {"color": {"test": {"$type": "color", "$value": color,
                                                       "$description": "Matching fallback."}}}})).all()


def test_hex_rounding_cannot_hide_a_contrast_failure():
    composition = Project(ROOT).compose("day")
    foreground = {"colorSpace": "srgb", "components": [0.465] * 3, "alpha": 1}
    background = color_from_hex("#ffffff")
    assert contrast_value(foreground, background) > 4.5
    assert contrast_value(color_from_hex(hex_color(foreground)), background) < 4.5
    composition.values["semantic.color.ink"] = foreground
    composition.values["semantic.color.bg-sunken"] = background
    with pytest.raises(TokenError, match="HEX fallback.*below 4.5"):
        check_contrast(composition)


def test_original_seed_metadata_is_retained_and_validated(tmp_path):
    shutil.copytree(ROOT / "tokens", tmp_path / "tokens")
    path = tmp_path / "tokens/base.tokens.json"
    document = json.loads(path.read_text())
    seeds = document["$extensions"][EXTENSION]["designSeeds"]
    assert seeds["primary"] + seeds["accents"] == SEEDS
    assert len(design_seed_values(seeds)) == 7
    seeds["primary"][0] += "-"
    path.write_text(json.dumps(document))
    with pytest.raises(TokenError, match="invalid Fjord OKLCH seed"):
        validate_project(tmp_path)
