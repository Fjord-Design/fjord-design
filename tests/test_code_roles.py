from pathlib import Path

import pytest

from fjord.code_roles import ROLES, check_syntax_distances, oklab, syntax_distances, validate_bindings
from fjord.contract import ANSI, MODES
from fjord.dtcg import TokenError, color_from_hex
from fjord.project import Project, check_contrast


ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("mode", MODES)
def test_every_binding_resolves_in_every_mode(mode):
    composed = Project(ROOT).compose(mode)
    validate_bindings(composed)
    assert check_contrast(composed) > 100


def test_binding_without_a_colour_source_fails():
    composed = Project(ROOT).compose("night")
    del composed.tokens["semantic.color.git-added"]
    with pytest.raises(TokenError, match="expected colour token semantic.color.git-added"):
        validate_bindings(composed)


@pytest.mark.parametrize("prefix, count", (("git-", 7), ("diagnostic-", 4)))
def test_status_roles_have_distinct_text_markers(prefix, count):
    markers = [role.marker for name, role in ROLES.items() if name.startswith(prefix)]
    assert len(markers) == count
    assert len(set(markers)) == count
    assert all(marker.strip() for marker in markers)


def test_diff_roles_have_distinct_text_markers():
    assert [ROLES[f"diff-{state}"].marker[0] for state in ("added", "changed", "deleted")] == ["+", "~", "-"]


def test_all_ansi_slots_are_bound_in_order():
    assert [ROLES[name].token for name in ANSI] == [f"semantic.color.{name}" for name in ANSI]
    assert ANSI[0] == "ansi-black"
    assert ANSI[8] == "ansi-bright-black"
    assert ANSI[15] == "ansi-bright-white"


def test_oklab_reference_red():
    expected = (0.6279553606, 0.2248630611, 0.1258462985)
    assert oklab(color_from_hex("#ff0000")) == pytest.approx(expected, rel=0, abs=5e-9)


@pytest.mark.parametrize("mode", MODES)
def test_syntax_distance_thresholds_pass_every_mode(mode):
    composed = Project(ROOT).compose(mode)
    assert len(list(syntax_distances(composed))) == 36
    check_syntax_distances(composed)


def test_readable_but_indistinguishable_syntax_fails():
    composed = Project(ROOT).compose("night")
    composed.values["semantic.color.syn-type"] = composed.values["semantic.color.syn-keyword"]
    with pytest.raises(TokenError, match="OKLab distance .* is below 7"):
        check_contrast(composed)
