import json
from pathlib import Path
import subprocess
import sys

import pytest

from fjord.contract import MODES
from fjord.dtcg import TokenError, hex_color, nest
from fjord.export import css_document, export
from fjord.project import Project, metadata

ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("theme", (None, ROOT / "examples/themes/fern.tokens.json"), ids=("base", "fern"))
@pytest.mark.parametrize("mode", MODES)
def test_formats_use_one_composition_and_are_deterministic(theme, mode):
    composed = Project(ROOT).compose(mode, theme)
    for kind in ("dtcg", "resolved", "css"):
        assert export(composed, kind) == export(composed, kind), kind
    assert export(composed, "dtcg") == nest(composed.tokens, dict(metadata(), mode=mode, colorSpace="srgb"))
    resolved = export(composed, "resolved")
    assert resolved["metadata"]["mode"] == mode
    assert resolved["metadata"]["colorSpace"] == "srgb"
    for name, value in composed.values.items():
        assert resolved["tokens"][name]["value"] == value, name
        assert resolved["tokens"][name]["original"] == composed.tokens[name]["$value"], name
    css = export(composed, "css")
    assert "--fjord-token-semantic-color-accent: " + hex_color(composed.values["semantic.color.accent"]) + ";" in css
    assert "--fjord-token-semantic-typography-body-font-size: 15.0px;" in css


def test_aliases_are_rejected_before_export(tmp_path):
    path = tmp_path / "bad.json"
    path.write_text(json.dumps({"$extensions": {"org.xerrion.fjord": metadata()},
                                "semantic": {"color": {"accent": {"$type": "color", "$value": "{semantic.color.not-real}", "$description": "Bad alias"}}}}))
    with pytest.raises(TokenError):
        Project(ROOT).compose("night", overrides=[path])


def test_missing_roles_are_rejected_before_export():
    composed = Project(ROOT).compose("night")
    del composed.tokens["semantic.color.syn-comment"]
    with pytest.raises(TokenError):
        export(composed, "css")


def test_name_collisions_are_rejected():
    composed = Project(ROOT).compose("night")
    composed.values["semantic.color.a-b"] = composed.values["semantic.color.ink"]
    composed.tokens["semantic.color.a-b"] = composed.tokens["semantic.color.ink"]
    composed.values["semantic.color.a.b"] = composed.values["semantic.color.ink"]
    composed.tokens["semantic.color.a.b"] = composed.tokens["semantic.color.ink"]
    with pytest.raises(TokenError, match="name collision"):
        css_document(composed)


def test_cli_cannot_replace_sources_and_is_repeatable(tmp_path):
    output = tmp_path / "night.tokens.css"
    command = [sys.executable, "-m", "fjord", "export", "--format", "css", "--output", str(output)]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    first = output.read_bytes()
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    assert output.read_bytes() == first
    target = ROOT / "tokens" / "base.tokens.json"
    before = target.read_bytes()
    result = subprocess.run(command[:-1] + [str(target)], cwd=ROOT, capture_output=True, text=True)
    assert result.returncode != 0
    assert target.read_bytes() == before
