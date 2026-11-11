import json
from pathlib import Path
import shutil

import pytest

from fjord.dtcg import TokenError, nest
from fjord.project import Project, check_contrast, metadata, validate_project


ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture
def usage_root(tmp_path):
    for name in ("tokens", "examples/themes", "examples/overrides"):
        shutil.copytree(ROOT / name, tmp_path / name)
    return tmp_path


def edit_usage(root, change):
    path = root / "tokens/usage.json"
    data = json.loads(path.read_text())
    change(data)
    path.write_text(json.dumps(data))


def test_usage_retains_required_roles_and_states(usage_root):
    project = Project(usage_root)
    assert len(project.usage["roles"]) == 35
    assert len(project.usage["states"]) == 8
    assert validate_project(usage_root).deprecated == []


@pytest.mark.parametrize("mode", ("night", "day", "aurora"))
@pytest.mark.parametrize("theme_path", (
    None, *sorted(path.relative_to(ROOT)
                  for directory in ("tokens/themes", "examples/themes")
                  for path in (ROOT / directory).glob("*.tokens.json")
                  if path != ROOT / "tokens/themes/fjord.tokens.json"),
), ids=lambda path: path.name if path is not None else "base")
@pytest.mark.parametrize("with_overrides", (False, True), ids=("no-overrides", "all-overrides"))
def test_themes_and_overrides_preserve_shared_design_contract(usage_root, mode, theme_path, with_overrides):
    project = Project(usage_root)
    original = project.compose(mode)
    bindings = {path: token for path, token in original.tokens.items() if path.startswith("component.")}
    theme = usage_root / theme_path if theme_path is not None else None
    overrides = sorted((usage_root / "examples/overrides").glob("*.tokens.json")) if with_overrides else []
    composition = project.compose(mode, theme, overrides)
    assert original.tokens.keys() == composition.tokens.keys()
    assert bindings == {path: token for path, token in composition.tokens.items() if path.startswith("component.")}
    check_contrast(composition)
    assert composition.deprecated == []


def test_missing_state_or_role_is_rejected(usage_root):
    edit_usage(usage_root, lambda data: data["states"].pop("loading"))
    with pytest.raises(TokenError, match="retain all"):
        Project(usage_root)


def test_help_cannot_use_disabled_text(usage_root):
    edit_usage(usage_root, lambda data: data["roles"]["text-help"].update(token="semantic.color.ink-disabled"))
    with pytest.raises(TokenError, match="disabled text must not be used"):
        Project(usage_root)


def test_help_cannot_be_marked_decorative_to_bypass_contrast(usage_root):
    edit_usage(usage_root, lambda data: data["roles"]["text-help"].update(kind="decorative", minimumContrast=None))
    with pytest.raises(TokenError, match="preserve its text role"):
        Project(usage_root)


def test_contrast_threshold_cannot_be_weakened(usage_root):
    edit_usage(usage_root, lambda data: data["states"]["loading"].update(minimumContrast=3))
    with pytest.raises(TokenError, match="4.5:1"):
        Project(usage_root)


def test_unknown_or_primitive_usage_reference_is_rejected(usage_root):
    edit_usage(usage_root, lambda data: data["roles"]["canvas"].update(token="primitive.color.night.bg-base"))
    with pytest.raises(TokenError, match="existing semantic or component"):
        Project(usage_root).compose("night")


@pytest.mark.parametrize("value", [
    pytest.param("{semantic.color.ink-disabled}", id="brace"),
    pytest.param({"$ref": "#/semantic/color/ink-disabled/$value"}, id="json-pointer"),
])
def test_help_alias_cannot_rebind_to_disabled_text(usage_root, value):
    path = usage_root / "override.json"
    path.write_text(json.dumps(nest({"semantic.color.ink-muted": {
        "$type": "color", "$value": value, "$description": "Invalid help rebinding."}}, metadata())))
    with pytest.raises(TokenError, match="must not alias disabled text"):
        Project(usage_root).compose("night", overrides=[path])


def test_later_override_cannot_hide_disabled_text_pointer_use(usage_root):
    first = usage_root / "disabled.json"
    last = usage_root / "repaired.json"
    for path, value in ((first, {"$ref": "#/semantic/color/ink-disabled/$value"}),
                        (last, "{semantic.color.ink-subtle}")):
        path.write_text(json.dumps(nest({"semantic.color.ink-muted": {
            "$type": "color", "$value": value, "$description": "Text usage layer."}}, metadata())))
    with pytest.raises(TokenError, match="must not alias disabled text"):
        Project(usage_root).compose("night", overrides=[first, last])


def test_opaque_surface_policy_rejects_transparency(usage_root):
    composition = Project(usage_root).compose("night")
    composition.values["semantic.color.bg-surface"]["alpha"] = .5
    with pytest.raises(TokenError, match="alpha cannot be ignored"):
        check_contrast(composition)


def test_incomplete_mode_has_no_dark_fallback(usage_root):
    path = usage_root / "tokens/modes/day.tokens.json"
    data = json.loads(path.read_text())
    del data["semantic"]["color"]["accent"]
    path.write_text(json.dumps(data))
    with pytest.raises(TokenError, match="missing mode roles"):
        Project(usage_root).compose("day")
