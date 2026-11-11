from pathlib import Path

import pytest

from fjord.dtcg import TokenError, color_from_hex
from fjord.project import Project, check_contrast, contrast_value


ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("foreground, background, expected", (
    ("#000000", "#ffffff", 21), ("#112233", "#112233", 1),
    ("#767676", "#ffffff", 4.5422249596), ("#777777", "#ffffff", 4.4780894536),
))
def test_known_srgb_ratios(foreground, background, expected):
    ratio = contrast_value(color_from_hex(foreground), color_from_hex(background))
    assert ratio == pytest.approx(expected, rel=0, abs=5e-8)


@pytest.mark.parametrize("mode, color, ratio", (
    ("night", "#738290", "4.498225"), ("day", "#687683", "4.096792"),
))
def test_dim_terminal_text_below_threshold_fails(mode, color, ratio):
    composed = Project(ROOT).compose(mode)
    composed.values["semantic.color.ansi-bright-black"] = color_from_hex(color)
    with pytest.raises(TokenError, match=ratio + ":1 is below 4.5:1"):
        check_contrast(composed)


@pytest.mark.parametrize("role, background, minimum", (
    ("focus-ring", "bg-base", "3"), ("danger", "danger-soft", "4.5"),
))
def test_focus_and_status_contrast_are_checked(role, background, minimum):
    composed = Project(ROOT).compose("day")
    composed.values[f"semantic.color.{role}"] = composed.values[f"semantic.color.{background}"]
    with pytest.raises(TokenError, match=f"{role}.*below {minimum}:1"):
        check_contrast(composed)


def test_comments_require_the_stricter_night_ratio():
    composed = Project(ROOT).compose("night")
    composed.values["semantic.color.syn-comment"] = color_from_hex("#849a99")
    with pytest.raises(TokenError, match="syn-comment.*below 7.5:1"):
        check_contrast(composed)


def test_explicit_full_opacity_is_supported():
    assert contrast_value(color_from_hex("#000000ff"), color_from_hex("#ffffffff")) == 21
