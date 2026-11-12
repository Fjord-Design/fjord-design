//! Project policy through `Project`: usage (P5), contrast (P6), syntax
//! separation (P7) and design seeds (P8).
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs;
use std::path::PathBuf;

use fjord::diagnostic::{Failure, Rule};
use fjord::dtcg::value::TypedValue;
use fjord::policy::contract::Mode;
use fjord::project::{Composition, Project, Selection};
use rstest::rstest;
use serde_json::{Value, json};

use common::{Fixture, layer, layer_document, rules};

const USAGE: &str = "tokens/usage.json";
const DISABLED: &str = "semantic.color.ink-disabled";

// ---------------------------------------------------------------- helpers

fn fixture() -> Fixture {
    Fixture::project(&["tokens"])
}

fn open(f: &Fixture) -> Project {
    match Project::open(f.root()) {
        Ok(p) => p,
        Err(failure) => panic!("{failure}"),
    }
}

fn compose(project: &Project, mode: Mode, overrides: &[PathBuf]) -> Result<Composition, Failure> {
    project.compose(&Selection {
        mode,
        theme: None,
        overrides: overrides.to_vec(),
    })
}

fn composed(project: &Project, mode: Mode, overrides: &[PathBuf]) -> Composition {
    match compose(project, mode, overrides) {
        Ok(c) => c,
        Err(failure) => panic!("{failure}"),
    }
}

/// Rewrites `tokens/usage.json` of the fixture.
fn edit_usage(f: &Fixture, edit: impl FnOnce(&mut Value)) {
    edit_json(f, USAGE, edit);
}

fn edit_json(f: &Fixture, rel: &str, edit: impl FnOnce(&mut Value)) {
    let path = f.path(rel);
    let mut value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

/// Asserts that every diagnostic of `failure` is `rule` (and there is one).
fn assert_only(failure: &Failure, rule: Rule) {
    let found = rules(failure);
    assert!(
        !found.is_empty() && found.iter().all(|r| *r == rule),
        "expected only {rule:?}:\n{failure}"
    );
}

/// Asserts a `rule` diagnostic located at token `path` (`<file>: <path>`).
fn assert_at(failure: &Failure, rule: Rule, path: &str) {
    let suffix = format!(": {path}");
    assert!(
        failure
            .0
            .iter()
            .any(|d| d.rule == rule && d.location.ends_with(&suffix)),
        "expected {rule:?} at {path}:\n{failure}"
    );
}

fn colour(components: [f64; 3]) -> Value {
    json!({"$type": "color", "$value": {"colorSpace": "srgb", "components": components}})
}

fn translucent(components: [f64; 3], alpha: f64) -> Value {
    json!({"$type": "color", "$value": {"colorSpace": "srgb", "components": components, "alpha": alpha}})
}

fn alias(path: &str) -> Value {
    json!({"$type": "color", "$value": format!("{{{path}}}")})
}

/// A whole-token JSON pointer alias to `path`.
fn pointer(path: &str) -> Value {
    json!({"$type": "color", "$value": {"$ref": format!("#/{}/$value", path.replace('.', "/"))}})
}

fn rgb(c: &Composition, path: &str) -> [f64; 3] {
    match &c.get(path).unwrap().value {
        TypedValue::Color(color) => color.components,
        other => panic!("{path} is not a colour: {other:?}"),
    }
}

/// Independent WCAG 2.2 oracle used to pick test colours.
fn luminance(rgb: [f64; 3]) -> f64 {
    let [r, g, b] = rgb.map(|c| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn hex_rounded(rgb: [f64; 3]) -> [f64; 3] {
    rgb.map(|c| (c * 255.0).round() / 255.0)
}

/// 8-bit greys, so unrounded and HEX contrast agree.
fn greys() -> impl Iterator<Item = [f64; 3]> {
    (0..=255u8).map(|k| [f64::from(k) / 255.0; 3])
}

/// The grey with the highest contrast below `min` against `other`.
fn closest_failing_grey(other: [f64; 3], min: f64) -> [f64; 3] {
    let grey = greys()
        .filter(|g| contrast(*g, other) < min)
        .max_by(|a, b| contrast(*a, other).total_cmp(&contrast(*b, other)))
        .unwrap();
    assert!(
        contrast(grey, other) > min - 0.25,
        "no grey close to {min}:1 against {other:?}"
    );
    grey
}

// ---------------------------------------------------------------- P5 static

/// Usage violations are found when the project opens, located in the usage file.
fn assert_usage_rejected(edit: fn(&mut Value)) {
    let f = fixture();
    edit_usage(&f, edit);
    let failure = Project::open(f.root()).unwrap_err();
    assert_only(&failure, Rule::Usage);
    assert!(
        failure.0.iter().all(|d| d.location.starts_with(USAGE)),
        "{failure}"
    );
}

#[rstest]
#[case::string(|v: &mut Value| v["schemaVersion"] = json!("1"))]
#[case::fraction(|v: &mut Value| v["schemaVersion"] = json!(1.5))]
#[case::zero(|v: &mut Value| v["schemaVersion"] = json!(0))]
#[case::missing(|v: &mut Value| { v.as_object_mut().unwrap().remove("schemaVersion"); })]
fn p5_schema_version_is_1(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

#[rstest]
#[case::missing_role(|v: &mut Value| { v["roles"].as_object_mut().unwrap().remove("focus"); })]
#[case::extra_role(|v: &mut Value| { let link = v["roles"]["link"].clone(); v["roles"]["brand-link"] = link; })]
#[case::border_as_decorative(|v: &mut Value| v["roles"]["border-control"]["kind"] = json!("decorative"))]
#[case::missing_kind(|v: &mut Value| { v["roles"]["canvas"].as_object_mut().unwrap().remove("kind"); })]
#[case::kind_case(|v: &mut Value| v["roles"]["text"]["kind"] = json!("Text"))]
#[case::unknown_kind(|v: &mut Value| v["roles"]["surface"]["kind"] = json!("surface"))]
fn p5_role_names_and_kinds_are_exact(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

#[rstest]
#[case::missing_state(|v: &mut Value| { v["states"].as_object_mut().unwrap().remove("selected"); })]
#[case::extra_state(|v: &mut Value| { let hover = v["states"]["hover"].clone(); v["states"]["visited"] = hover; })]
#[case::states_not_object(|v: &mut Value| v["states"] = json!([]))]
fn p5_states_are_exact(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

#[rstest]
#[case::role_description_blank(|v: &mut Value| v["roles"]["link"]["description"] = json!("   "))]
#[case::role_description_missing(|v: &mut Value| { v["roles"]["canvas"].as_object_mut().unwrap().remove("description"); })]
#[case::role_description_not_text(|v: &mut Value| v["roles"]["focus"]["description"] = json!(1))]
#[case::applies_to_empty(|v: &mut Value| v["states"]["pressed"]["appliesTo"] = json!(""))]
#[case::applies_to_missing(|v: &mut Value| { v["states"]["loading"].as_object_mut().unwrap().remove("appliesTo"); })]
#[case::behaviour_unicode_space(|v: &mut Value| v["states"]["invalid"]["behaviour"] = json!("\u{a0}\t\u{2003}"))]
#[case::behaviour_missing(|v: &mut Value| { v["states"]["disabled"].as_object_mut().unwrap().remove("behaviour"); })]
fn p5_descriptions_are_non_blank(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

#[rstest]
#[case::below_floor(|v: &mut Value| v["roles"]["text-help"]["minimumContrast"] = json!(4.4))]
#[case::missing_minimum(|v: &mut Value| { v["roles"]["link"].as_object_mut().unwrap().remove("minimumContrast"); })]
#[case::null_minimum(|v: &mut Value| v["roles"]["info"]["minimumContrast"] = Value::Null)]
#[case::string_minimum(|v: &mut Value| v["roles"]["text-secondary"]["minimumContrast"] = json!("4.5"))]
#[case::no_background(|v: &mut Value| v["roles"]["overlay-text"]["backgrounds"] = json!([]))]
#[case::backgrounds_not_array(|v: &mut Value| v["roles"]["success"]["backgrounds"] = json!("semantic.color.bg-base"))]
fn p5_text_roles_need_4_5_and_a_background(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

#[rstest]
#[case::border_below_3(|v: &mut Value| v["roles"]["border-control"]["minimumContrast"] = json!(2.9))]
#[case::border_null(|v: &mut Value| v["roles"]["border-control"]["minimumContrast"] = Value::Null)]
#[case::border_no_background(|v: &mut Value| v["roles"]["border-control"]["backgrounds"] = json!([]))]
#[case::focus_below_3(|v: &mut Value| v["roles"]["focus"]["minimumContrast"] = json!(2.99))]
#[case::focus_no_background(|v: &mut Value| v["roles"]["focus"]["backgrounds"] = json!([]))]
fn p5_border_and_focus_roles_need_3_and_a_background(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

#[test]
fn p5_role_may_raise_its_minimum() {
    let f = fixture();
    edit_usage(&f, |v| {
        v["roles"]["border-control"]["minimumContrast"] = json!(3.5);
    });
    let project = open(&f);
    let role = project
        .usage()
        .roles()
        .iter()
        .find(|r| r.name == "border-control")
        .unwrap();
    assert_eq!(role.minimum, Some(3.5));
}

#[rstest]
#[case::fill(|v: &mut Value| v["roles"]["surface-selected"]["minimumContrast"] = json!(0))]
#[case::decorative(|v: &mut Value| v["roles"]["border-divider"]["minimumContrast"] = json!(3))]
#[case::overlay(|v: &mut Value| v["roles"]["overlay-scrim"]["minimumContrast"] = json!(1))]
#[case::disabled(|v: &mut Value| v["roles"]["text-disabled"]["minimumContrast"] = json!(4.5))]
#[case::missing_key(|v: &mut Value| { v["roles"]["accent-fill"].as_object_mut().unwrap().remove("minimumContrast"); })]
fn p5_exempt_roles_have_null_minimum(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

#[rstest]
#[case::disabled_role_as_text(|v: &mut Value| v["roles"]["text-disabled"]["kind"] = json!("text"))]
#[case::other_role_as_disabled(|v: &mut Value| v["roles"]["overlay-text"]["kind"] = json!("disabled"))]
#[case::disabled_role_rebound(|v: &mut Value| v["roles"]["text-disabled"]["token"] = json!("semantic.color.ink-muted"))]
#[case::link_binds_ink_disabled(|v: &mut Value| v["roles"]["link"]["token"] = json!(DISABLED))]
#[case::fill_binds_ink_disabled(|v: &mut Value| v["roles"]["canvas"]["token"] = json!(DISABLED))]
fn p5_only_text_disabled_is_disabled_and_binds_ink_disabled(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

#[rstest]
#[case::disabled_foreground_rebound(|v: &mut Value| v["states"]["disabled"]["foreground"] = json!("semantic.color.ink-muted"))]
#[case::disabled_minimum(|v: &mut Value| v["states"]["disabled"]["minimumContrast"] = json!(4.5))]
#[case::hover_uses_ink_disabled(|v: &mut Value| v["states"]["hover"]["foreground"] = json!(DISABLED))]
#[case::loading_below_4_5(|v: &mut Value| v["states"]["loading"]["minimumContrast"] = json!(3))]
#[case::invalid_null(|v: &mut Value| v["states"]["invalid"]["minimumContrast"] = Value::Null)]
fn p5_disabled_state_alone_uses_ink_disabled(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

#[rstest]
#[case::text_help(|v: &mut Value| v["roles"]["text-help"]["backgrounds"] = json!(["semantic.color.bg-surface", "semantic.color.bg-raised", "semantic.color.bg-surface"]))]
#[case::focus(|v: &mut Value| v["roles"]["focus"]["backgrounds"].as_array_mut().unwrap().push(json!("semantic.color.bg-sunken")))]
fn p5_backgrounds_are_unique(#[case] edit: fn(&mut Value)) {
    assert_usage_rejected(edit);
}

// ---------------------------------------------------------------- P5 dynamic

#[rstest]
#[case::missing_role_token(|v: &mut Value| v["roles"]["canvas"]["token"] = json!("semantic.color.absent"))]
#[case::missing_background(|v: &mut Value| v["roles"]["text"]["backgrounds"][0] = json!("semantic.color.nope"))]
#[case::primitive_colour(|v: &mut Value| v["roles"]["surface"]["token"] = json!("primitive.color.night.bg-surface"))]
#[case::semantic_non_colour(|v: &mut Value| v["roles"]["link"]["token"] = json!("semantic.font.sans"))]
#[case::state_non_colour(|v: &mut Value| v["states"]["invalid"]["foreground"] = json!("semantic.spacing.step-4"))]
#[case::component_non_colour(|v: &mut Value| v["states"]["default"]["background"] = json!("component.button.primary.radius"))]
fn p5_referenced_paths_are_existing_colour_tokens(
    #[case] edit: fn(&mut Value),
    #[values(Mode::Night, Mode::Day, Mode::Aurora)] mode: Mode,
) {
    let f = fixture();
    edit_usage(&f, edit);
    let project = open(&f);
    let failure = compose(&project, mode, &[]).unwrap_err();
    assert_only(&failure, Rule::Usage);
    assert!(
        failure.0.iter().all(|d| d.location.starts_with(USAGE)),
        "{failure}"
    );
}

#[test]
fn p5_fill_role_bound_to_a_translucent_colour_fails() {
    let f = fixture();
    edit_usage(&f, |v| {
        v["roles"]["canvas"]["token"] = json!("semantic.color.scrim");
    });
    let project = open(&f);
    for mode in Mode::ALL {
        let failure = compose(&project, mode, &[]).unwrap_err();
        assert_only(&failure, Rule::Usage);
        assert_at(&failure, Rule::Usage, "semantic.color.scrim");
    }
}

#[rstest]
fn p5_fill_roles_resolve_opaque(#[values(Mode::Night, Mode::Day, Mode::Aurora)] mode: Mode) {
    const RAISED: &str = "semantic.color.bg-raised";
    let f = fixture();
    let project = open(&f);
    let raised = rgb(&composed(&project, mode, &[]), RAISED);

    // Explicit alpha 1 is opaque.
    let opaque = layer(
        f.root(),
        "opaque.tokens.json",
        &[(RAISED, translucent(raised, 1.0))],
    );
    composed(&project, mode, &[opaque]);

    // `surface-floating` and `overlay-surface` fill with bg-raised.
    let glass = layer(
        f.root(),
        "glass.tokens.json",
        &[(RAISED, translucent(raised, 0.9))],
    );
    let failure = compose(&project, mode, &[glass]).unwrap_err();
    assert_only(&failure, Rule::Usage);
    assert!(
        failure
            .0
            .iter()
            .all(|d| d.location == format!("glass.tokens.json: {RAISED}")),
        "{failure}"
    );
}

/// An active role or state reaching `ink-disabled` through an alias chain
/// split across two overrides, mixing brace and pointer syntax. The first
/// override alone is valid; the second rebinds active text into the chain.
#[rstest]
#[case::role_brace_then_pointer(
    pointer(DISABLED),
    "semantic.color.ink-subtle",
    alias("semantic.color.chart-grid")
)]
#[case::role_pointer_then_brace(
    alias(DISABLED),
    "semantic.color.ink-muted",
    pointer("semantic.color.chart-grid")
)]
#[case::state_pointer_then_brace(
    alias(DISABLED),
    "component.button.primary.foreground",
    pointer("semantic.color.chart-grid")
)]
#[case::role_and_state_brace_then_pointer(
    pointer(DISABLED),
    "semantic.color.danger",
    alias("semantic.color.chart-grid")
)]
fn p5_disabled_text_cannot_be_rebound_through_alias_chains(
    #[case] first: Value,
    #[case] rebound: &str,
    #[case] second: Value,
    #[values(Mode::Night, Mode::Day, Mode::Aurora)] mode: Mode,
) {
    let f = fixture();
    let project = open(&f);
    let first = layer(
        f.root(),
        "first.tokens.json",
        &[("semantic.color.chart-grid", first)],
    );
    composed(&project, mode, std::slice::from_ref(&first));

    let second = layer(f.root(), "second.tokens.json", &[(rebound, second)]);
    let failure = compose(&project, mode, &[first, second]).unwrap_err();
    assert_only(&failure, Rule::Usage);
    assert!(
        failure
            .0
            .iter()
            .any(|d| d.location == format!("second.tokens.json: {rebound}")),
        "{failure}"
    );
}

// ---------------------------------------------------------------- P6

/// Which side of a contrast pair an override replaces.
#[derive(Clone, Copy, Debug)]
enum Side {
    Fg,
    Bg,
}

/// One failing override per row of the step 5 contrast table: the token on
/// the `set` side becomes the 8-bit grey just below the row minimum against
/// the other side of the pair, in every mode.
#[rstest]
#[case::ink_on_s7("ink-muted", "bg-pressed", Side::Fg, 4.5)]
#[case::ink_subtle_on_surfaces("ink-subtle", "bg-raised", Side::Fg, 4.5)]
#[case::text_safe_accents_on_s7("fern", "bg-hover", Side::Fg, 4.5)]
#[case::focus_ring_on_s7("focus-ring", "bg-selected", Side::Fg, 3.0)]
#[case::line_strong("line-strong", "bg-surface", Side::Fg, 3.0)]
#[case::ink_inverse("ink-inverse", "bg-inverse", Side::Bg, 4.5)]
#[case::on_accent("on-accent", "accent-pressed", Side::Bg, 4.5)]
#[case::accent_ink("accent-ink", "accent-soft", Side::Fg, 4.5)]
#[case::on_danger("on-danger", "danger", Side::Bg, 4.5)]
#[case::info_soft("info", "info-soft", Side::Bg, 4.5)]
#[case::success_soft("ink", "success-soft", Side::Bg, 4.5)]
#[case::warning_soft("warning", "warning-soft", Side::Fg, 4.5)]
#[case::danger_soft("danger", "danger-soft", Side::Fg, 4.5)]
#[case::hearth_soft("hearth", "hearth-soft", Side::Fg, 4.5)]
#[case::lilac_soft("ink", "lilac-soft", Side::Bg, 4.5)]
#[case::ink_on_highlights("ink", "search-match-current", Side::Bg, 4.5)]
#[case::syntax("syn-string", "bg-sunken", Side::Fg, 4.5)]
#[case::ansi_slots("ansi-cyan", "bg-base", Side::Fg, 4.5)]
#[case::prompt_and_trees("git-ignored", "bg-base", Side::Fg, 4.5)]
#[case::diagnostics("info", "bg-surface", Side::Fg, 4.5)]
#[case::diff_added("git-added", "diff-add-bg", Side::Bg, 4.5)]
#[case::diff_modified("git-modified", "warning-soft", Side::Fg, 4.5)]
#[case::diff_removed("git-removed", "diff-del-bg", Side::Fg, 4.5)]
fn p6_table_row_fails_below_its_minimum(
    #[case] fg: &str,
    #[case] bg: &str,
    #[case] set: Side,
    #[case] min: f64,
    #[values(Mode::Night, Mode::Day, Mode::Aurora)] mode: Mode,
) {
    let (fg, bg) = (
        format!("semantic.color.{fg}"),
        format!("semantic.color.{bg}"),
    );
    let f = fixture();
    let project = open(&f);
    let default = composed(&project, mode, &[]);
    let (target, other) = match set {
        Side::Fg => (&fg, &bg),
        Side::Bg => (&bg, &fg),
    };
    let grey = closest_failing_grey(rgb(&default, other), min);
    let file = layer(
        f.root(),
        "row.tokens.json",
        &[(target.as_str(), colour(grey))],
    );
    let failure = compose(&project, mode, &[file]).unwrap_err();
    assert_at(&failure, Rule::Contrast, &fg);
}

#[test]
fn p6_hex_only_failure_in_day() {
    const FG: &str = "semantic.color.ink-inverse";
    const BG: &str = "semantic.color.bg-inverse";
    let near = [0.465; 3];
    let white = [1.0; 3];
    assert!(contrast(near, white) >= 4.5);
    assert!(contrast(hex_rounded(near), white) < 4.5);

    let f = fixture();
    let project = open(&f);
    let hex_only = layer(
        f.root(),
        "hex-only.tokens.json",
        &[(BG, colour(white)), (FG, colour(near))],
    );
    let failure = compose(&project, Mode::Day, &[hex_only]).unwrap_err();
    assert_eq!(rules(&failure), [Rule::Contrast], "{failure}");
    assert_at(&failure, Rule::Contrast, FG);

    // #767676 passes on both bases.
    let passing = layer(
        f.root(),
        "passing.tokens.json",
        &[(BG, colour(white)), (FG, colour([118.0 / 255.0; 3]))],
    );
    composed(&project, Mode::Day, &[passing]);
}

#[rstest]
#[case::foreground("semantic.color.ink-inverse")]
#[case::background("semantic.color.bg-inverse")]
fn p6_translucent_colour_in_a_pair_fails(
    #[case] path: &str,
    #[values(Mode::Night, Mode::Day, Mode::Aurora)] mode: Mode,
) {
    let f = fixture();
    let project = open(&f);
    let components = rgb(&composed(&project, mode, &[]), path);

    let opaque = layer(
        f.root(),
        "opaque.tokens.json",
        &[(path, translucent(components, 1.0))],
    );
    composed(&project, mode, &[opaque]);

    let glass = layer(
        f.root(),
        "glass.tokens.json",
        &[(path, translucent(components, 0.99))],
    );
    let failure = compose(&project, mode, &[glass]).unwrap_err();
    assert_eq!(rules(&failure), [Rule::Contrast], "{failure}");
    assert_at(&failure, Rule::Contrast, "semantic.color.ink-inverse");
}

/// `syn-comment` on `bg-base` needs 5:1, raised to 7.5:1 in Night. A grey
/// between 5 and 7.5 that passes its 4.5 pairs fails only in Night.
#[rstest]
#[case::night(Mode::Night, false)]
#[case::day(Mode::Day, true)]
#[case::aurora(Mode::Aurora, true)]
fn p6_night_comment_needs_7_5(#[case] mode: Mode, #[case] passes: bool) {
    const COMMENT: &str = "semantic.color.syn-comment";
    let f = fixture();
    let project = open(&f);
    let default = composed(&project, mode, &[]);
    let [base, sunken, surface] = ["bg-base", "bg-sunken", "bg-surface"]
        .map(|bg| rgb(&default, &format!("semantic.color.{bg}")));
    let grey = greys()
        .find(|g| {
            (5.5..7.2).contains(&contrast(*g, base))
                && contrast(*g, sunken) >= 4.6
                && contrast(*g, surface) >= 4.6
        })
        .unwrap();
    let file = layer(f.root(), "comment.tokens.json", &[(COMMENT, colour(grey))]);
    match compose(&project, mode, &[file]) {
        Ok(_) => assert!(passes, "{mode}: a comment below 7.5:1 passed"),
        Err(failure) => {
            assert!(!passes, "{mode}: {failure}");
            assert_eq!(rules(&failure), [Rule::Contrast], "{failure}");
            assert_at(&failure, Rule::Contrast, COMMENT);
        }
    }
}

#[rstest]
#[case::day(Mode::Day, false)]
#[case::night(Mode::Night, true)]
#[case::aurora(Mode::Aurora, true)]
fn p6_ansi_black_is_checked_in_day_only(#[case] mode: Mode, #[case] passes: bool) {
    const BLACK: &str = "semantic.color.ansi-black";
    let f = fixture();
    let project = open(&f);
    let base = rgb(&composed(&project, mode, &[]), "semantic.color.bg-base");
    let grey = closest_failing_grey(base, 4.5);
    let file = layer(f.root(), "black.tokens.json", &[(BLACK, colour(grey))]);
    match compose(&project, mode, &[file]) {
        Ok(_) => assert!(passes, "{mode}: ansi-black below 4.5:1 passed"),
        Err(failure) => {
            assert!(!passes, "{mode}: {failure}");
            assert_eq!(rules(&failure), [Rule::Contrast], "{failure}");
            assert_at(&failure, Rule::Contrast, BLACK);
        }
    }
}

/// Primary button pairs are checked after overrides, through the component
/// bindings (brace or pointer).
#[rstest]
#[case::background_binds_on_accent(
    "component.button.primary.background",
    alias("semantic.color.on-accent"),
    "component.button.primary.foreground"
)]
#[case::background_points_at_on_accent(
    "component.button.primary.background",
    pointer("semantic.color.on-accent"),
    "component.button.primary.foreground"
)]
#[case::hover_binds_on_accent(
    "component.button.primary.hover-background",
    alias("semantic.color.on-accent"),
    "component.button.primary.foreground"
)]
#[case::pressed_binds_on_accent(
    "component.button.primary.pressed-background",
    alias("semantic.color.on-accent"),
    "component.button.primary.foreground"
)]
#[case::foreground_binds_accent(
    "component.button.primary.foreground",
    alias("semantic.color.accent"),
    "component.button.primary.foreground"
)]
#[case::focus_on_surface(
    "component.button.primary.focus-color",
    alias("semantic.color.bg-raised"),
    "component.button.primary.focus-color"
)]
fn p6_button_pairs_fail(
    #[case] path: &str,
    #[case] token: Value,
    #[case] foreground: &str,
    #[values(Mode::Night, Mode::Day, Mode::Aurora)] mode: Mode,
) {
    let f = fixture();
    let project = open(&f);
    let file = layer(f.root(), "button.tokens.json", &[(path, token)]);
    let failure = compose(&project, mode, &[file]).unwrap_err();
    assert_only(&failure, Rule::Contrast);
    assert_at(&failure, Rule::Contrast, foreground);
}

/// U pairs join the table; a duplicate pair keeps the highest minimum.
#[test]
fn p6_usage_pairs_keep_the_highest_minimum() {
    let f = fixture();
    edit_usage(&f, |v| {
        v["roles"]["text-placeholder"]["minimumContrast"] = json!(21);
    });
    let project = open(&f);
    for mode in Mode::ALL {
        let failure = compose(&project, mode, &[]).unwrap_err();
        assert_only(&failure, Rule::Contrast);
        assert_at(&failure, Rule::Contrast, "semantic.color.ink-subtle");
    }
}

// ---------------------------------------------------------------- P7

#[rstest]
#[case::type_is_keyword("semantic.color.syn-type", "semantic.color.syn-keyword")]
#[case::regex_is_operator("semantic.color.syn-regex", "semantic.color.syn-operator")]
fn p7_identical_syntax_colours_fail(
    #[case] path: &str,
    #[case] target: &str,
    #[values(Mode::Night, Mode::Day, Mode::Aurora)] mode: Mode,
) {
    let f = fixture();
    let project = open(&f);
    let file = layer(f.root(), "syntax.tokens.json", &[(path, alias(target))]);
    let failure = compose(&project, mode, &[file]).unwrap_err();
    assert_eq!(rules(&failure), [Rule::SyntaxDistance], "{failure}");
    assert_at(&failure, Rule::SyntaxDistance, path);
}

// ---------------------------------------------------------------- P8

#[test]
fn p8_malformed_base_seed_fails() {
    let f = fixture();
    edit_json(&f, "tokens/base.tokens.json", |v| {
        v["$extensions"]["org.xerrion.fjord"]["designSeeds"]["primary"][1] =
            json!("oklch(50 0.1 30)");
    });
    let project = open(&f);
    for mode in Mode::ALL {
        let failure = compose(&project, mode, &[]).unwrap_err();
        assert_eq!(rules(&failure), [Rule::DesignSeed], "{failure}");
        assert_eq!(
            failure.0[0].location,
            "tokens/base.tokens.json: designSeeds.primary[1]"
        );
    }
}

/// A later layer that sets `designSeeds` is checked against the merged seeds.
#[rstest]
#[case::malformed_accent(json!({"accents": ["oklch(76% 0.06 71)", "oklch(87% 0.03 46)", "oklch(76% 1.5 10)", "oklch(87% 0.03 130)"]}), false)]
#[case::wrong_shape(json!("oklch(50% 0.1 30)"), false)]
#[case::extra_key(json!({"secondary": ["oklch(50% 0.1 30)"]}), false)]
#[case::valid_accents(json!({"accents": ["oklch(76% 0.06 71)", "oklch(87% 0.03 46)", "oklch(76% 0.06 10)", "oklch(87% 0.03 130)"]}), true)]
fn p8_override_seeds_are_checked(#[case] seeds: Value, #[case] passes: bool) {
    let f = fixture();
    let project = open(&f);
    let mut document = layer_document(&[]);
    document["$extensions"]["org.xerrion.fjord"]["designSeeds"] = seeds;
    let file = f.path("seeds.tokens.json");
    fs::write(&file, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
    match compose(&project, Mode::Night, &[file]) {
        Ok(_) => assert!(passes, "malformed seeds passed"),
        Err(failure) => {
            assert!(!passes, "{failure}");
            assert_eq!(rules(&failure), [Rule::DesignSeed], "{failure}");
            assert!(
                failure.0[0]
                    .location
                    .starts_with("seeds.tokens.json: designSeeds"),
                "{failure}"
            );
        }
    }
}
