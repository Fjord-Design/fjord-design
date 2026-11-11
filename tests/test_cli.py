import json
from pathlib import Path
import shutil
import subprocess
import sys

import pytest

from fjord.dtcg import nest
from fjord.project import metadata


ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture
def root(tmp_path):
    for directory in ("tokens", "assets/fonts", "assets/icons"):
        shutil.copytree(ROOT / directory, tmp_path / directory)
    return tmp_path


def run_cli(root, command, *arguments):
    return subprocess.run([sys.executable, "-m", "fjord", command,
                           "--root", str(root), *map(str, arguments)],
                          cwd=root, capture_output=True, text=True)


def write_layer(root, name, path, value):
    file = root / name
    file.write_text(json.dumps(nest({path: {"$type": "dimension", "$value": value,
                                           "$description": "Test layer."}}, metadata())))
    return file


def add_deprecated_gap(root):
    source = root / "tokens/base.tokens.json"
    document = json.loads(source.read_text())
    document["semantic"]["test-gap"] = {
        "$type": "dimension", "$value": "{semantic.spacing.step-4}",
        "$description": "Synthetic deprecated gap.", "$deprecated": "Use {semantic.spacing.step-4}."}
    source.write_text(json.dumps(document))


def test_validate_checks_assets_without_documentation_or_implementations(root):
    result = run_cli(root, "validate")
    assert result.returncode == 0, result.stderr
    assert "PASS:" in result.stdout
    assert result.stderr == ""
    assert "9 font faces and 16 icons with pinned licences" in result.stdout
    assert not (root / "dist").exists()


@pytest.mark.parametrize("relative", [
    "assets/fonts/cascadia-code/CascadiaCode.woff2",
    "assets/fonts/fira-code/OFL.txt",
    "assets/icons/info.svg",
    "assets/icons/LICENSE",
])
def test_validate_reports_corrupt_assets_and_licences_without_tracebacks(root, relative):
    path = root / relative
    path.write_bytes(path.read_bytes() + b"changed")
    result = run_cli(root, "validate")
    assert result.returncode == 1
    assert "ERROR" in result.stderr
    assert "Traceback" not in result.stderr


@pytest.mark.parametrize("relative", [
    "assets/fonts/manifest.json",
    "assets/fonts/inspection.json",
    "assets/icons/manifest.json",
])
def test_validate_rejects_malformed_asset_manifest_shapes(root, relative):
    (root / relative).write_text(json.dumps([]))
    result = run_cli(root, "validate")
    assert result.returncode == 1
    assert "Traceback" not in result.stderr


@pytest.mark.parametrize(("command", "arguments"), [
    pytest.param("build", [], id="build"),
    pytest.param("export", ["--format", "resolved"], id="export-resolved"),
    pytest.param("export", ["--format", "css"], id="export-css"),
    pytest.param("export", ["--format", "dtcg"], id="export-dtcg"),
])
def test_token_build_and_export_do_not_require_asset_files(root, command, arguments):
    shutil.rmtree(root / "assets")
    result = run_cli(root, command, *arguments)
    assert result.returncode == 0, result.stderr
    assert result.stderr == ""
    assert run_cli(root, "validate").returncode == 1


def test_build_and_dtcg_export_match(root):
    build = root / "build.json"
    exported = root / "export.json"
    for command, args in (("build", ["--output", build]),
                          ("export", ["--format", "dtcg", "--output", exported])):
        result = run_cli(root, command, *args)
        assert result.returncode == 0, result.stderr
        assert "Wrote night:" in result.stdout
    assert build.read_bytes() == exported.read_bytes()


@pytest.mark.parametrize(("command", "arguments"), [
    pytest.param("build", [], id="build"),
    pytest.param("export", ["--format", "resolved"], id="export-resolved"),
])
def test_build_and_export_report_requested_theme_and_override_uses(root, command, arguments):
    add_deprecated_gap(root)
    theme = write_layer(root, "deprecated-theme.json", "semantic.test-gap", {"value": 12, "unit": "px"})
    override = write_layer(root, "deprecated-override.json", "component.button.primary.padding-inline", "{semantic.test-gap}")
    result = run_cli(root, command, *arguments, "--theme", theme, "--override", override)
    assert result.returncode == 0, result.stderr
    warnings = result.stderr.splitlines()
    assert len(warnings) == 2
    for source in (theme, override):
        matching = [warning for warning in warnings if str(source) in warning]
        assert len(matching) == 1
        for detail in ("WARN deprecated", "semantic.test-gap", "semantic.spacing.step-4"):
            assert detail in matching[0]


@pytest.mark.parametrize(("command", "arguments"), [
    pytest.param("validate", [], id="validate"),
    pytest.param("build", [], id="build"),
    pytest.param("export", ["--format", "css"], id="export-css"),
])
def test_shared_source_warning_is_not_duplicated_by_default_mode_preflight(root, command, arguments):
    add_deprecated_gap(root)
    source = root / "tokens/base.tokens.json"
    document = json.loads(source.read_text())
    document["component"]["button"]["primary"]["padding-inline"]["$value"] = "{semantic.test-gap}"
    source.write_text(json.dumps(document))
    result = run_cli(root, command, *arguments)
    assert result.returncode == 0, result.stderr
    warnings = result.stderr.splitlines()
    assert len(warnings) == 1
    for detail in ("WARN deprecated", str(source), "component.button.primary.padding-inline",
                   "semantic.test-gap", "semantic.spacing.step-4"):
        assert detail in warnings[0]


def test_replaced_default_theme_does_not_warn_for_an_unapplied_use(root):
    add_deprecated_gap(root)
    default = root / "tokens/themes/fjord.tokens.json"
    deprecated = write_layer(root, "deprecated.json", "semantic.test-gap", {"value": 12, "unit": "px"})
    default.write_text(deprecated.read_text())
    theme = write_layer(root, "modern.json", "semantic.spacing.step-4", {"value": 16, "unit": "px"})
    result = run_cli(root, "build", "--theme", theme)
    assert result.returncode == 0, result.stderr
    assert result.stderr == ""


@pytest.mark.parametrize(("command", "arguments"), [
    pytest.param("build", [], id="build"),
    pytest.param("export", ["--format", "resolved"], id="export-resolved"),
])
def test_build_and_export_cannot_replace_source_assets_through_symlinks(root, command, arguments):
    target = root / "assets/icons/info.svg"
    original = target.read_bytes()
    output = root / "output.json"
    output.symlink_to(target)
    result = run_cli(root, command, *arguments, "--output", output)
    assert result.returncode == 1
    assert "must not replace token sources" in result.stderr
    assert target.read_bytes() == original
