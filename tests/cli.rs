//! CLI integration tests (CLI1, CLI2, O4, O5, P9 and A1 through `fjord`).
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs;
use std::path::Path;

use common::{Fixture, fjord, layer};
use rstest::rstest;
use serde_json::{Value, json};
use tempfile::TempDir;

/// Exit code and captured streams of one `fjord` run.
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    /// Lines of `stream` starting with `prefix`.
    fn lines<'a>(stream: &'a str, prefix: &str) -> Vec<&'a str> {
        stream.lines().filter(|l| l.starts_with(prefix)).collect()
    }

    fn errors(&self, rule: &str) -> Vec<&str> {
        Self::lines(&self.stderr, &format!("ERROR [{rule}]"))
    }

    fn warnings(&self, rule: &str) -> Vec<&str> {
        Self::lines(&self.stderr, &format!("WARN [{rule}]"))
    }

    #[track_caller]
    fn assert_exit(&self, code: i32) -> &Self {
        assert_eq!(
            self.code, code,
            "exit code\nstdout:\n{}\nstderr:\n{}",
            self.stdout, self.stderr
        );
        self
    }
}

/// Run `fjord args…` with `cwd` as the working directory.
fn run_in(cwd: &Path, args: &[&str]) -> Run {
    let output = fjord().args(args).current_dir(cwd).output().unwrap();
    Run {
        code: output.status.code().expect("fjord exited through a signal"),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

/// Run `fjord <command> --root <fixture> args…` from the fixture root.
fn run(f: &Fixture, command: &[&str], args: &[&str]) -> Run {
    let root = f.root().to_str().unwrap();
    let mut all: Vec<&str> = command.to_vec();
    all.extend(["--root", root]);
    all.extend(args);
    run_in(f.root(), &all)
}

fn tokens() -> Fixture {
    Fixture::project(&["tokens"])
}

fn full() -> Fixture {
    Fixture::project(&["tokens", "assets"])
}

/// A valid override changing one semantic radius.
fn valid_override(root: &Path, name: &str) -> String {
    let path = layer(
        root,
        name,
        &[(
            "semantic.radius.control",
            json!({"$type": "dimension", "$value": {"value": 6, "unit": "px"}}),
        )],
    );
    path.to_str().unwrap().to_owned()
}

/// An override assigning a path that does not exist (C4).
fn invalid_override(root: &Path) -> String {
    let path = layer(
        root,
        "broken.tokens.json",
        &[(
            "semantic.radius.no-such-radius",
            json!({"$type": "dimension", "$value": {"value": 6, "unit": "px"}}),
        )],
    );
    path.to_str().unwrap().to_owned()
}

fn edit_json(path: &Path, edit: impl FnOnce(&mut Value)) {
    let mut value: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

// ---------------------------------------------------------------- CLI2

#[rstest]
#[case::top_level(&["--help"])]
#[case::build(&["build", "--help"])]
#[case::export(&["export", "--help"])]
#[case::validate(&["validate", "--help"])]
fn cli2_help_exits_0_on_stdout(#[case] args: &[&str]) {
    let dir = TempDir::new().unwrap();
    let r = run_in(dir.path(), args);
    r.assert_exit(0);
    assert_ne!(r.stdout, "");
    assert!(r.stderr.is_empty(), "{}", r.stderr);
}

#[rstest]
#[case::unknown_mode(&["build", "--mode", "wrong"])]
#[case::abbreviated_flag(&["build", "--mod", "night"])]
#[case::abbreviated_flag_export(&["export", "--format", "css", "--mod", "night"])]
#[case::abbreviated_root(&["validate", "--roo", "."])]
#[case::abbreviated_subcommand(&["valid"])]
#[case::missing_format(&["export", "--mode", "night"])]
#[case::unknown_format(&["export", "--format", "yaml"])]
fn cli2_usage_errors_exit_2(#[case] args: &[&str]) {
    let f = tokens();
    let r = run_in(f.root(), args);
    r.assert_exit(2);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert_ne!(r.stderr, "");
    assert!(!f.path("dist").exists());
}

#[test]
fn cli2_validation_failure_exits_1_on_stderr() {
    let f = tokens();
    let broken = invalid_override(f.root());
    let r = run(&f, &["build"], &["--override", &broken]);
    r.assert_exit(1);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert_eq!(r.errors("override-path").len(), 1, "{}", r.stderr);
    assert!(r.stderr.lines().last().unwrap().starts_with("ERROR "));
    assert!(!f.path("dist").exists());
}

// --------------------------------------------------------- root discovery

#[test]
fn root_is_discovered_from_tokens_dir() {
    let f = full();
    let r = run_in(&f.path("tokens"), &["validate"]);
    r.assert_exit(0);
    assert!(r.stdout.starts_with("PASS:"), "{}", r.stdout);

    let r = run_in(&f.path("tokens/modes"), &["build", "--mode", "day"]);
    r.assert_exit(0);
    assert!(f.path("dist/fjord.day.tokens.json").is_file());
    let wrote = Run::lines(&r.stdout, "Wrote day: ");
    assert_eq!(wrote.len(), 1, "{}", r.stdout);
    assert!(
        wrote[0].ends_with("dist/fjord.day.tokens.json"),
        "{}",
        wrote[0]
    );
}

#[rstest]
#[case::validate(&["validate"])]
#[case::build(&["build"])]
#[case::export(&["export", "--format", "css"])]
fn root_discovery_fails_outside_a_project(#[case] args: &[&str]) {
    let dir = TempDir::new().unwrap();
    let cwd = fs::canonicalize(dir.path()).unwrap();
    let r = run_in(&cwd, args);
    r.assert_exit(1);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    let errors = r.errors("io");
    assert_eq!(errors.len(), 1, "{}", r.stderr);
    assert!(errors[0].contains(cwd.to_str().unwrap()), "{}", errors[0]);
    assert_eq!(fs::read_dir(&cwd).unwrap().count(), 0);
}

// -------------------------------------------------------------------- O5

#[rstest]
#[case::build_night(&["build"], "night", "dist/fjord.night.tokens.json")]
#[case::build_day(&["build", "--mode", "day"], "day", "dist/fjord.day.tokens.json")]
#[case::build_aurora(&["build", "--mode", "aurora"], "aurora", "dist/fjord.aurora.tokens.json")]
#[case::export_dtcg(&["export", "--format", "dtcg"], "night", "dist/fjord.night.dtcg.json")]
#[case::export_resolved(&["export", "--format", "resolved", "--mode", "day"], "day", "dist/fjord.day.resolved.json")]
#[case::export_css(&["export", "--format", "css", "--mode", "aurora"], "aurora", "dist/fjord.aurora.tokens.css")]
fn o5_default_output_names(#[case] args: &[&str], #[case] mode: &str, #[case] rel: &str) {
    let f = tokens();
    let r = run(&f, args, &[]);
    r.assert_exit(0);
    assert_eq!(r.stdout, format!("Wrote {mode}: {rel}\n"));
    assert!(r.stderr.is_empty(), "{}", r.stderr);
    assert!(f.path(rel).is_file());
    assert_eq!(fs::read_dir(f.path("dist")).unwrap().count(), 1);
}

#[test]
fn o5_default_output_is_relative_to_root_not_cwd() {
    let f = tokens();
    let elsewhere = TempDir::new().unwrap();
    let root = f.root().to_str().unwrap();
    let r = run_in(elsewhere.path(), &["build", "--root", root]);
    r.assert_exit(0);
    assert!(f.path("dist/fjord.night.tokens.json").is_file());
    assert_eq!(fs::read_dir(elsewhere.path()).unwrap().count(), 0);
}

#[test]
fn o5_explicit_output_is_shown_as_given() {
    let f = tokens();
    let r = run(
        &f,
        &["export", "--format", "resolved"],
        &["--output", "dist/custom.json"],
    );
    r.assert_exit(0);
    assert_eq!(r.stdout, "Wrote night: dist/custom.json\n");
    assert!(f.path("dist/custom.json").is_file());
}

#[rstest]
#[case::night("night")]
#[case::day("day")]
#[case::aurora("aurora")]
fn o5_build_equals_export_dtcg(#[case] mode: &str) {
    let f = tokens();
    let theme = f.path("tokens/themes/fern.tokens.json");
    let compact = valid_override(f.root(), "compact.tokens.json");
    let request = [
        "--mode",
        mode,
        "--theme",
        theme.to_str().unwrap(),
        "--override",
        &compact,
    ];
    run(&f, &["build"], &request).assert_exit(0);
    run(&f, &["export", "--format", "dtcg"], &request).assert_exit(0);
    let built = fs::read(f.path(&format!("dist/fjord.{mode}.tokens.json"))).unwrap();
    let exported = fs::read(f.path(&format!("dist/fjord.{mode}.dtcg.json"))).unwrap();
    assert_ne!(built.len(), 0);
    assert!(
        built == exported,
        "build and export --format dtcg differ for {mode}"
    );
}

// -------------------------------------------------------------------- O4

#[test]
fn o4_refuses_layer_source_under_tokens() {
    let f = tokens();
    let before = fs::read(f.path("tokens/base.tokens.json")).unwrap();
    let r = run(&f, &["build"], &["--output", "tokens/base.tokens.json"]);
    r.assert_exit(1);
    assert_eq!(r.errors("output-protected").len(), 1, "{}", r.stderr);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert_eq!(fs::read(f.path("tokens/base.tokens.json")).unwrap(), before);
}

#[test]
fn o4_refuses_root_file_outside_dist() {
    let f = tokens();
    let out = f.path("out.json");
    let r = run(
        &f,
        &["export", "--format", "resolved"],
        &["--output", out.to_str().unwrap()],
    );
    r.assert_exit(1);
    assert_eq!(r.errors("output-protected").len(), 1, "{}", r.stderr);
    assert!(!out.exists());
}

#[cfg(unix)]
#[test]
fn o4_refuses_symlink_in_dist_into_assets() {
    let f = full();
    let target = f.path("assets/icons/check.svg");
    let before = fs::read(&target).unwrap();
    fs::create_dir_all(f.path("dist")).unwrap();
    std::os::unix::fs::symlink(&target, f.path("dist/check.svg")).unwrap();
    let r = run(
        &f,
        &["export", "--format", "css"],
        &["--output", "dist/check.svg"],
    );
    r.assert_exit(1);
    assert_eq!(r.errors("output-protected").len(), 1, "{}", r.stderr);
    assert_eq!(fs::read(&target).unwrap(), before);
    assert!(f.path("dist/check.svg").is_symlink());
}

#[test]
fn o4_refuses_supplied_layer_file_under_dist() {
    let f = tokens();
    let compact = valid_override(f.root(), "dist/compact.tokens.json");
    let before = fs::read(&compact).unwrap();
    let r = run(
        &f,
        &["build"],
        &["--override", &compact, "--output", &compact],
    );
    r.assert_exit(1);
    assert_eq!(r.errors("output-protected").len(), 1, "{}", r.stderr);
    assert_eq!(fs::read(&compact).unwrap(), before);
}

#[test]
fn o4_refuses_supplied_theme_outside_root() {
    let f = tokens();
    let outside = TempDir::new().unwrap();
    let theme = outside.path().join("fern.tokens.json");
    fs::copy(f.path("tokens/themes/fern.tokens.json"), &theme).unwrap();
    let before = fs::read(&theme).unwrap();
    let theme = theme.to_str().unwrap();
    let r = run(&f, &["build"], &["--theme", theme, "--output", theme]);
    r.assert_exit(1);
    assert_eq!(r.errors("output-protected").len(), 1, "{}", r.stderr);
    assert_eq!(fs::read(theme).unwrap(), before);
}

#[test]
fn o4_failed_build_keeps_the_old_output() {
    let f = tokens();
    let out = f.path("dist/fjord.night.tokens.json");
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, b"previous build\n").unwrap();
    let broken = invalid_override(f.root());
    let r = run(&f, &["build"], &["--override", &broken]);
    r.assert_exit(1);
    assert!(!r.errors("override-path").is_empty(), "{}", r.stderr);
    assert_eq!(fs::read(&out).unwrap(), b"previous build\n");
    assert_eq!(fs::read_dir(f.path("dist")).unwrap().count(), 1);
}

// ------------------------------------------------------------------- CLI1

#[rstest]
#[case::build(&["build"])]
#[case::export_dtcg(&["export", "--format", "dtcg"])]
#[case::export_resolved(&["export", "--format", "resolved"])]
#[case::export_css(&["export", "--format", "css"])]
fn cli1_build_and_export_need_no_assets(#[case] args: &[&str]) {
    let f = full();
    fs::remove_dir_all(f.path("assets")).unwrap();
    let r = run(&f, args, &["--mode", "day"]);
    r.assert_exit(0);
    assert!(r.stdout.starts_with("Wrote day: "), "{}", r.stdout);
}

#[test]
fn cli1_validate_passes_on_a_full_copy() {
    let f = full();
    let r = run(&f, &["validate"], &[]);
    r.assert_exit(0);
    assert!(r.stdout.starts_with("PASS:"), "{}", r.stdout);
    assert_eq!(r.stdout.lines().count(), 1);
    assert!(r.stderr.is_empty(), "{}", r.stderr);
    assert!(!f.path("dist").exists());
}

// -------------------------------------------------------------------- A1

#[test]
fn a1_validate_fails_after_tampering_a_font_byte() {
    let f = full();
    let font = f.path("assets/fonts/raleway/Raleway-latin.woff2");
    let mut bytes = fs::read(&font).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    fs::write(&font, bytes).unwrap();
    let r = run(&f, &["validate"], &[]);
    r.assert_exit(1);
    assert!(!r.errors("font-asset").is_empty(), "{}", r.stderr);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
}

#[rstest]
#[case::fonts_array("assets/fonts/manifest.json", "[]", "font-asset")]
#[case::fonts_empty_object("assets/fonts/manifest.json", "{}", "font-asset")]
#[case::fonts_null("assets/fonts/manifest.json", "null", "font-asset")]
#[case::fonts_families_object(
    "assets/fonts/manifest.json",
    r#"{"schemaVersion": 1, "fontDisplay": "swap", "families": {}}"#,
    "font-asset"
)]
#[case::fonts_not_json("assets/fonts/manifest.json", "[", "json")]
#[case::inspection_array("assets/fonts/inspection.json", "[]", "font-asset")]
#[case::icons_array("assets/icons/manifest.json", "[]", "icon-asset")]
#[case::icons_empty_object("assets/icons/manifest.json", "{}", "icon-asset")]
#[case::icons_string("assets/icons/manifest.json", r#""icons""#, "icon-asset")]
#[case::icons_not_json("assets/icons/manifest.json", "{", "json")]
fn a1_validate_fails_for_malformed_manifest(
    #[case] file: &str,
    #[case] content: &str,
    #[case] rule: &str,
) {
    let f = full();
    fs::write(f.path(file), content).unwrap();
    let r = run(&f, &["validate"], &[]);
    r.assert_exit(1);
    let errors = r.errors(rule);
    assert!(!errors.is_empty(), "{}", r.stderr);
    assert!(errors.iter().any(|e| e.contains(file)), "{}", r.stderr);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
}

// -------------------------------------------------------------------- P9

/// Deprecate `primitive.border-width.border-hairline`, which base uses once.
fn deprecate_base_use(f: &Fixture) {
    edit_json(&f.path("tokens/base.tokens.json"), |base| {
        base["primitive"]["border-width"]["border-hairline"]
            .as_object_mut()
            .unwrap()
            .insert(
                "$deprecated".into(),
                json!("Use semantic.border-width.border-hairline."),
            );
    });
}

#[test]
fn p9_validate_warns_once_for_a_base_use() {
    let f = full();
    deprecate_base_use(&f);
    let r = run(&f, &["validate"], &[]);
    r.assert_exit(0);
    assert_eq!(r.warnings("deprecated").len(), 1, "{}", r.stderr);
    assert!(r.stdout.starts_with("PASS:"), "{}", r.stdout);
}

#[rstest]
#[case::build_night(&["build"])]
#[case::build_day(&["build", "--mode", "day"])]
#[case::export_css(&["export", "--format", "css", "--mode", "aurora"])]
fn p9_build_warns_once_despite_preflight(#[case] args: &[&str]) {
    let f = tokens();
    deprecate_base_use(&f);
    let r = run(&f, args, &[]);
    r.assert_exit(0);
    let warnings = r.warnings("deprecated");
    assert_eq!(warnings.len(), 1, "{}", r.stderr);
    assert!(
        warnings[0].contains("tokens/base.tokens.json"),
        "{}",
        warnings[0]
    );
    assert!(r.stdout.starts_with("Wrote "), "{}", r.stdout);
}

// ------------------------------------------------------------------- IN3

#[rstest]
#[case::day_mode_validate("tokens/modes/day.tokens.json", &["validate"])]
#[case::day_mode_build("tokens/modes/day.tokens.json", &["build"])]
#[case::usage_validate("tokens/usage.json", &["validate"])]
#[case::usage_export("tokens/usage.json", &["export", "--format", "css"])]
fn in3_missing_source_is_reported_by_path(#[case] file: &str, #[case] command: &[&str]) {
    let f = full();
    fs::remove_file(f.path(file)).unwrap();
    let r = run(&f, command, &[]);
    r.assert_exit(1);
    let errors = r.errors("io");
    assert!(!errors.is_empty(), "{}", r.stderr);
    assert!(errors.iter().all(|e| e.contains(file)), "{}", r.stderr);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert!(!f.path("dist").exists());
}
