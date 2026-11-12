#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::needless_pass_by_value,
    reason = "rstest case tables pass owned JSON"
)]
//! Composition (C2–C7), D5, RF6, P1–P3 and P9 through `Project::open`,
//! `Project::compose` and `validate_tokens` on a temporary copy of the
//! repository's `tokens/`, with `common::layer` overrides.
//!
//! Cases already covered by the unit tests in `src/project.rs` are not
//! repeated here.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{Fixture, layer, layer_document};
use fjord::diagnostic::{Diagnostic, Failure, Rule, Severity};
use fjord::dtcg::tree::Layer;
use fjord::policy::contract::Mode;
use fjord::project::{Composition, Project, Selection, validate_tokens};
use rstest::rstest;
use serde_json::{Value, json};

fn fixture() -> Fixture {
    Fixture::project(&["tokens"])
}

fn project(f: &Fixture) -> Project {
    Project::open(f.root()).unwrap_or_else(|e| panic!("{e}"))
}

fn selection(mode: Mode, theme: Option<PathBuf>, overrides: &[PathBuf]) -> Selection {
    Selection {
        mode,
        theme,
        overrides: overrides.to_vec(),
    }
}

/// Night with the default theme and `overrides`.
fn compose(f: &Fixture, overrides: &[PathBuf]) -> Result<Composition, Failure> {
    project(f).compose(&selection(Mode::Night, None, overrides))
}

fn composed(f: &Fixture, s: &Selection) -> Composition {
    project(f).compose(s).unwrap_or_else(|e| panic!("{e}"))
}

fn errors(failure: &Failure) -> Vec<Rule> {
    failure
        .0
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.rule)
        .collect()
}

fn failed(result: Result<Composition, Failure>) -> Failure {
    match result {
        Ok(_) => panic!("composition should fail"),
        Err(failure) => failure,
    }
}

/// A token without `$description` (`layer` adds one).
fn tok(ty: &str, value: Value) -> Value {
    json!({"$type": ty, "$value": value})
}

fn px(v: f64) -> Value {
    json!({"value": v, "unit": "px"})
}

/// Provenance label of the token at `path`.
fn source(c: &Composition, path: &str) -> String {
    let token = c.get(path).unwrap_or_else(|| panic!("{path} missing"));
    let origin = token
        .origin
        .unwrap_or_else(|| panic!("{path} has no origin"));
    c.layers()[usize::from(origin)].source_label()
}

/// Rewrite the JSON file `rel` of the fixture.
fn edit(f: &Fixture, rel: &str, change: impl FnOnce(&mut Value)) {
    let path = f.path(rel);
    let mut doc: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    change(&mut doc);
    fs::write(&path, serde_json::to_string_pretty(&doc).unwrap()).unwrap();
}

/// Write a whole layer document (metadata included by `layer_document`).
fn write_layer(f: &Fixture, name: &str, extra: Value) -> PathBuf {
    let mut doc = layer_document(&[]);
    for (key, value) in extra.as_object().unwrap() {
        if key == "$extensions" {
            for (ext, v) in value.as_object().unwrap() {
                doc["$extensions"][ext] = v.clone();
            }
        } else {
            doc[key] = value.clone();
        }
    }
    let path = f.path(name);
    fs::write(&path, serde_json::to_string_pretty(&doc).unwrap()).unwrap();
    path
}

const BASE: &str = "tokens/base.tokens.json";
const NIGHT: &str = "tokens/modes/night.tokens.json";
const DEFAULT_THEME: &str = "tokens/themes/fjord.tokens.json";

/// Insert (`Some`) or remove (`None`) the token at a dotted `path` of a
/// layer document, creating missing groups.
fn set_token(doc: &mut Value, path: &str, token: Option<Value>) {
    let (parent, name) = path.rsplit_once('.').unwrap();
    let mut node = doc;
    for seg in parent.split('.') {
        node = node
            .as_object_mut()
            .unwrap()
            .entry(seg)
            .or_insert_with(|| json!({}));
    }
    let group = node.as_object_mut().unwrap();
    match token {
        Some(t) => {
            group.insert(name.to_owned(), t);
        }
        None => {
            assert!(group.remove(name).is_some(), "{path} not found");
        }
    }
}

// ----- C2 / C6: order and provenance --------------------------------------

#[test]
fn c2_layers_apply_in_order_and_dependants_follow_the_last_override() {
    let f = fixture();
    let narrow = layer(
        f.root(),
        "narrow.tokens.json",
        &[("semantic.spacing.step-4", tok("dimension", px(12.0)))],
    );
    let wide = layer(
        f.root(),
        "wide.tokens.json",
        &[("semantic.spacing.step-4", tok("dimension", px(20.0)))],
    );
    let fern = f.path("tokens/themes/fern.tokens.json");
    let c = composed(
        &f,
        &selection(
            Mode::Night,
            Some(fern.clone()),
            &[narrow.clone(), wide.clone()],
        ),
    );
    let labels: Vec<String> = c.layers().iter().map(Layer::source_label).collect();
    assert_eq!(
        labels,
        [
            "base",
            "mode:night",
            "theme:fern.tokens.json",
            "override:1:narrow.tokens.json",
            "override:2:wide.tokens.json"
        ]
    );
    assert_eq!(c.get("semantic.spacing.step-4").unwrap().json, px(20.0));
    for dependant in [
        "semantic.layout.page-gutter",
        "semantic.density.gap-comfortable",
        "component.button.primary.padding-inline",
    ] {
        assert_eq!(c.get(dependant).unwrap().json, px(20.0), "{dependant}");
        assert_eq!(source(&c, dependant), "base", "{dependant}");
    }
    assert_eq!(
        source(&c, "semantic.spacing.step-4"),
        "override:2:wide.tokens.json"
    );
    assert_eq!(
        source(&c, "semantic.color.accent"),
        "theme:fern.tokens.json"
    );
    assert_eq!(source(&c, "semantic.color.ink"), "mode:night");
    assert_eq!(source(&c, "primitive.spacing.space-4"), "base");

    let reversed = composed(&f, &selection(Mode::Night, Some(fern), &[wide, narrow]));
    assert_eq!(
        reversed.get("semantic.layout.page-gutter").unwrap().json,
        px(12.0)
    );
    assert_eq!(
        source(&reversed, "semantic.spacing.step-4"),
        "override:2:narrow.tokens.json"
    );
}

#[test]
fn c6_provenance_through_extends_and_root() {
    let f = fixture();
    edit(&f, BASE, |base| {
        base["semantic"]["spacing-scale"] = json!({
            "$type": "dimension",
            "regular": {
                "$root": {"$value": "{primitive.spacing.space-2}", "$description": "Regular step."},
                "wide": {"$value": "{primitive.spacing.space-4}", "$description": "Wide step."}
            },
            "compact": {"$extends": "{semantic.spacing-scale.regular}"}
        });
    });
    let c = composed(&f, &selection(Mode::Night, None, &[]));
    assert_eq!(source(&c, "semantic.spacing-scale.compact.$root"), "base");

    let theme = write_layer(
        &f,
        "redeclare.tokens.json",
        json!({"semantic": {"spacing-scale": {"compact": {"$extends": "{semantic.spacing-scale.regular}"}}}}),
    );
    let root = layer(
        f.root(),
        "root.tokens.json",
        &[(
            "semantic.spacing-scale.regular.$root",
            json!({"$value": "{primitive.spacing.space-3}"}),
        )],
    );
    let wide = layer(
        f.root(),
        "wide.tokens.json",
        &[(
            "semantic.spacing-scale.compact.wide",
            json!({"$value": "{primitive.spacing.space-1}"}),
        )],
    );
    let c = composed(&f, &selection(Mode::Night, Some(theme), &[root, wide]));
    assert_eq!(
        source(&c, "semantic.spacing-scale.regular.$root"),
        "override:1:root.tokens.json"
    );
    // A later change of the target changes the value, not the provenance.
    assert_eq!(
        c.get("semantic.spacing-scale.compact.$root").unwrap().json,
        px(12.0)
    );
    assert_eq!(
        source(&c, "semantic.spacing-scale.compact.$root"),
        "theme:redeclare.tokens.json"
    );
    assert_eq!(
        source(&c, "semantic.spacing-scale.compact.wide"),
        "override:2:wide.tokens.json"
    );
    assert_eq!(
        c.get("semantic.spacing-scale.compact.wide").unwrap().json,
        json!({"value": 4.0, "unit": "px"})
    );
}

// ----- C3: mode roles -------------------------------------------------------

#[rstest]
#[case::mode_misses_a_colour_role(NIGHT, "semantic.color.scrim", None)]
#[case::mode_misses_a_shadow_role(NIGHT, "semantic.shadow.shadow-pressed", None)]
#[case::mode_assigns_a_non_mode_role(
    NIGHT,
    "semantic.spacing.step-6",
    Some(json!({"$type": "dimension", "$value": "{primitive.spacing.space-6}", "$description": "Moved."}))
)]
#[case::mode_adds_an_unknown_colour(
    NIGHT,
    "semantic.color.extra",
    Some(json!({"$type": "color", "$value": "{primitive.color.night.ink}", "$description": "Extra."}))
)]
#[case::base_defines_a_shadow_role(
    BASE,
    "semantic.shadow.shadow-sm",
    Some(json!({"$type": "shadow", "$value": "{primitive.shadow.night.shadow-sm}", "$description": "Shadow."}))
)]
fn c3_mode_layer_assigns_exactly_the_mode_roles(
    #[case] file: &str,
    #[case] path: &str,
    #[case] token: Option<Value>,
) {
    let f = fixture();
    edit(&f, file, |doc| set_token(doc, path, token));
    let failure = failed(compose(&f, &[]));
    let mode_roles: Vec<&Diagnostic> = failure
        .0
        .iter()
        .filter(|d| d.rule == Rule::ModeRoles)
        .collect();
    assert_eq!(mode_roles.len(), 1, "{failure}");
    assert_eq!(mode_roles[0].location, format!("{file}: {path}"));
}

// ----- C4: themes and overrides ---------------------------------------------

#[rstest]
#[case::removed_path(
    "semantic.spacing.space-4",
    tok("dimension", px(16.0)),
    &[Rule::OverridePath]
)]
#[case::new_group(
    "semantic.extra.x",
    tok("dimension", px(16.0)),
    &[Rule::OverridePath]
)]
#[case::unchanged_primitive(
    "primitive.spacing.space-4",
    tok("dimension", px(16.0)),
    &[Rule::OverrideNamespace]
)]
#[case::untyped_retype(
    "semantic.spacing.step-6",
    json!({"$value": "{semantic.z-index.z-base}"}),
    &[Rule::OverrideType, Rule::RequiredRole]
)]
#[case::token_becomes_a_group(
    "semantic.spacing.step-6.inner",
    tok("dimension", px(24.0)),
    &[Rule::OverridePath, Rule::OverrideTokenSet, Rule::RequiredRole]
)]
fn c4_override_paths_token_set_and_types(
    #[case] path: &str,
    #[case] token: Value,
    #[case] expected: &[Rule],
) {
    let f = fixture();
    let file = layer(f.root(), "o.tokens.json", &[(path, token)]);
    let failure = failed(compose(&f, &[file]));
    let mut found = errors(&failure);
    found.sort_by_key(|r| r.id());
    found.dedup();
    let mut expected = expected.to_vec();
    expected.sort_by_key(|r| r.id());
    assert_eq!(found, expected, "{failure}");
}

#[test]
fn d5_c4_unknown_namespace_fails_in_base_and_in_overrides() {
    let f = fixture();
    let file = layer(
        f.root(),
        "o.tokens.json",
        &[("custom.x", tok("dimension", px(1.0)))],
    );
    let rules = errors(&failed(compose(&f, &[file])));
    assert!(rules.contains(&Rule::OverrideNamespace), "{rules:?}");

    edit(&f, BASE, |base| {
        base["custom"] =
            json!({"x": {"$type": "dimension", "$value": px(1.0), "$description": "Custom."}});
    });
    assert_eq!(errors(&failed(compose(&f, &[]))), [Rule::Namespace]);
}

#[rstest]
#[case::same_number(json!(1.0), true)]
#[case::changed_number(json!(2), false)]
fn c4_primitive_values_compare_numerically(#[case] n: Value, #[case] valid: bool) {
    let f = fixture();
    edit(&f, BASE, |base| {
        base["$extensions"]["org.example"] = json!({"n": 1});
        base["primitive"]["vendor"] = json!({"n": {
            "$type": "number",
            "$value": {"$ref": "#/$extensions/org.example/n"},
            "$description": "Vendor number."
        }});
    });
    let file = write_layer(
        &f,
        "o.tokens.json",
        json!({"$extensions": {"org.example": {"n": n}}}),
    );
    let result = compose(&f, &[file]);
    if valid {
        let c = result.unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(c.get("primitive.vendor.n").unwrap().json, json!(1.0));
    } else {
        assert_eq!(errors(&failed(result)), [Rule::OverridePrimitive]);
    }
}

fn body(font_size: Value, line_height: Option<Value>) -> Value {
    let mut value = json!({
        "fontFamily": "{semantic.font.sans}",
        "fontSize": font_size,
        "fontWeight": 400,
        "letterSpacing": px(0.0),
    });
    if let Some(lh) = line_height {
        value["lineHeight"] = lh;
    }
    value
}

#[rstest]
#[case::null_dimension("semantic.spacing.step-6", tok("dimension", Value::Null))]
#[case::null_field("semantic.typography.body", tok("typography", body(Value::Null, Some(json!(1.5)))))]
#[case::partial_typography("semantic.typography.body", tok("typography", body(px(15.0), None)))]
#[case::partial_dimension("semantic.spacing.step-6", tok("dimension", json!({"value": 24})))]
#[case::short_bezier("semantic.easing.easing-standard", tok("cubicBezier", json!([0.2, 0, 0])))]
#[case::partial_colour("semantic.color.ink", tok("color", json!({"colorSpace": "srgb"})))]
fn c4_null_values_and_partial_composites_fail(#[case] path: &str, #[case] token: Value) {
    let f = fixture();
    let file = layer(f.root(), "o.tokens.json", &[(path, token)]);
    assert_eq!(errors(&failed(compose(&f, &[file]))), [Rule::Value]);
}

// ----- C5: every state is validated -----------------------------------------

#[rstest]
#[case::missing_reference_in_the_theme(
    "semantic.color.accent",
    tok("color", json!("{semantic.color.absent}")),
    Rule::ReferenceMissing
)]
#[case::blank_description_in_the_theme(
    "semantic.spacing.step-6",
    json!({"$type": "dimension", "$value": "{primitive.spacing.space-6}", "$description": " "}),
    Rule::Description
)]
fn c5_an_invalid_theme_is_not_hidden_by_a_later_fix(
    #[case] path: &str,
    #[case] bad: Value,
    #[case] rule: Rule,
) {
    let f = fixture();
    let theme = layer(f.root(), "bad.tokens.json", &[(path, bad)]);
    let fix = layer(
        f.root(),
        "fix.tokens.json",
        &[
            (
                "semantic.color.accent",
                tok("color", json!("{primitive.color.night.accent}")),
            ),
            (
                "semantic.spacing.step-6",
                tok("dimension", json!("{primitive.spacing.space-6}")),
            ),
        ],
    );
    let failure = failed(project(&f).compose(&selection(Mode::Night, Some(theme), &[fix])));
    assert_eq!(errors(&failure), [rule]);
    assert!(
        failure
            .0
            .iter()
            .all(|d| d.location.starts_with("bad.tokens.json")),
        "{failure}"
    );
}

#[test]
fn c5_validate_tokens_reports_every_failing_selection_in_order() {
    let f = fixture();
    let bad = layer(
        f.root(),
        "bad.tokens.json",
        &[("semantic.size.target-touch", tok("dimension", px(43.0)))],
    );
    let p = project(&f);
    let selections = [
        selection(Mode::Aurora, None, std::slice::from_ref(&bad)),
        selection(Mode::Day, None, &[]),
        selection(Mode::Night, None, std::slice::from_ref(&bad)),
    ];
    let failure = validate_tokens(&p, &selections).expect_err("two selections fail");
    let contexts: Vec<&str> = failure
        .0
        .iter()
        .map(|d| d.context.as_deref().unwrap())
        .collect();
    assert_eq!(contexts, ["aurora", "night"]);
    assert_eq!(errors(&failure), [Rule::Layout, Rule::Layout]);

    let ok = validate_tokens(&p, &selections[1..2]).unwrap();
    assert_eq!(ok.len(), 1);
    assert_eq!(ok[0].mode(), Mode::Day);
}

// ----- RF6: hierarchy and component bindings ---------------------------------

#[rstest]
#[case::typed_pointer(tok("color", json!({"$ref": "#/semantic/color/accent/$value"})))]
#[case::untyped_pointer(json!({"$value": {"$ref": "#/semantic/color/accent/$value"}}))]
#[case::untyped_brace(json!({"$value": "{semantic.color.accent}"}))]
fn rf6_component_binds_a_semantic_role_by_brace_or_pointer(#[case] token: Value) {
    let f = fixture();
    let file = layer(
        f.root(),
        "o.tokens.json",
        &[("component.button.primary.background", token)],
    );
    let c = compose(&f, &[file]).unwrap_or_else(|e| panic!("{e}"));
    let background = c.get("component.button.primary.background").unwrap();
    assert_eq!(
        background.json,
        c.get("semantic.color.accent").unwrap().json
    );
    assert_eq!(background.references, ["semantic.color.accent".into()]);
}

#[rstest]
#[case::brace_to_a_primitive("background", tok("color", json!("{primitive.color.night.accent}")))]
#[case::pointer_to_a_primitive("background", tok("color", json!({"$ref": "#/primitive/color/night/accent/$value"})))]
#[case::brace_to_a_component("background", tok("color", json!("{component.button.primary.hover-background}")))]
#[case::pointer_to_a_semantic_field("font-size", tok("dimension", json!({"$ref": "#/semantic/typography/body/$value/fontSize"})))]
#[case::composite_with_semantic_fields("font-size", tok("dimension", json!({
    "value": {"$ref": "#/semantic/typography/body/$value/fontSize/value"}, "unit": "px"
})))]
fn rf6_other_component_values_fail(#[case] name: &str, #[case] token: Value) {
    let f = fixture();
    let path = format!("component.button.primary.{name}");
    let file = layer(f.root(), "o.tokens.json", &[(&path, token)]);
    let failure = failed(compose(&f, &[file]));
    assert_eq!(errors(&failure), [Rule::ComponentBinding], "{failure}");
}

#[test]
fn rf6_semantic_token_cannot_point_at_a_component() {
    let f = fixture();
    let file = layer(
        f.root(),
        "o.tokens.json",
        &[(
            "semantic.spacing.step-6",
            tok(
                "dimension",
                json!({"$ref": "#/component/button/primary/padding-block/$value"}),
            ),
        )],
    );
    assert_eq!(errors(&failed(compose(&f, &[file]))), [Rule::Hierarchy]);
}

#[rstest]
#[case::brace(json!("{semantic.spacing.step-1}"))]
#[case::pointer(json!({"$ref": "#/semantic/spacing/step-1/$value"}))]
#[case::property_pointer(json!({
    "value": {"$ref": "#/semantic/typography/body/$value/fontSize/value"}, "unit": "px"
}))]
fn rf6_primitive_cannot_reference_a_semantic_token(#[case] value: Value) {
    let f = fixture();
    edit(&f, BASE, |base| {
        base["primitive"]["spacing"]["space-6"]["$value"] = value;
    });
    assert_eq!(errors(&failed(compose(&f, &[]))), [Rule::Hierarchy]);
}

// ----- P1–P3: required roles, constraints, layout ---------------------------

#[rstest]
#[case::missing_semantic("semantic.radius.xs", None)]
#[case::missing_component("component.button.primary.font-family", None)]
#[case::wrong_type(
    "semantic.radius.xs",
    Some(json!({"$type": "number", "$value": "{primitive.z-index.z-base}", "$description": "Retyped."}))
)]
fn p1_required_roles_exist_with_their_type(#[case] path: &str, #[case] token: Option<Value>) {
    let f = fixture();
    edit(&f, BASE, |base| set_token(base, path, token));
    let failure = failed(compose(&f, &[]));
    assert_eq!(errors(&failure), [Rule::RequiredRole], "{failure}");
    assert!(failure.0[0].location.ends_with(&format!(": {path}")));
}

fn rem(v: f64) -> Value {
    json!({"value": v, "unit": "rem"})
}

fn ms(v: f64) -> Value {
    json!({"value": v, "unit": "ms"})
}

fn seconds(v: f64) -> Value {
    json!({"value": v, "unit": "s"})
}

/// Compose Night with one override assigning `value` (as `ty`) at `path`;
/// `Ok` or the error rules.
fn check_one(path: &str, ty: &str, value: Value) -> Result<(), Vec<Rule>> {
    let f = fixture();
    let file = layer(f.root(), "o.tokens.json", &[(path, tok(ty, value))]);
    compose(&f, &[file]).map(|_| ()).map_err(|e| errors(&e))
}

fn assert_limit(path: &str, ty: &str, value: Value, valid: bool, rule: Rule) {
    let shown = value.to_string();
    match check_one(path, ty, value) {
        Ok(()) => assert!(valid, "{path} = {shown} should fail with {rule:?}"),
        Err(found) => {
            assert!(!valid, "{path} = {shown} should pass: {found:?}");
            assert!(
                !found.is_empty() && found.iter().all(|r| *r == rule),
                "{path} = {shown}: {found:?}"
            );
        }
    }
}

#[rstest]
#[case::spacing_zero("semantic.spacing.step-6", px(0.0), true)]
#[case::spacing_negative("semantic.spacing.step-6", px(-1.0), false)]
#[case::spacing_negative_rem("semantic.spacing.step-6", rem(-0.0625), false)]
#[case::radius_zero("semantic.radius.xs", px(0.0), true)]
#[case::radius_negative("semantic.radius.xs", px(-1.0), false)]
#[case::border_width_negative("semantic.border-width.border-hairline", px(-1.0), false)]
#[case::size_zero("semantic.size.icon-control", px(0.0), false)]
#[case::size_negative("semantic.size.icon-control", px(-1.0), false)]
#[case::size_small_rem("semantic.size.icon-control", rem(0.0625), true)]
#[case::font_size_zero("semantic.font-size.text-2xs", px(0.0), false)]
#[case::font_size_small_rem("semantic.font-size.text-2xs", rem(0.0625), true)]
#[case::layout_zero("semantic.layout.content-max", px(0.0), true)]
#[case::layout_negative("semantic.layout.content-max", px(-1.0), false)]
#[case::density_negative("semantic.density.gap-compact", px(-1.0), false)]
fn p2_dimension_constraints(#[case] path: &str, #[case] value: Value, #[case] valid: bool) {
    assert_limit(path, "dimension", value, valid, Rule::RoleConstraint);
}

#[rstest]
#[case::integer_below_zero("semantic.z-index.z-base", json!(-1), true)]
#[case::fraction("semantic.z-index.z-base", json!(0.5), false)]
#[case::integral_float("semantic.z-index.z-base", json!(1.0), true)]
fn p2_z_index_integers(#[case] path: &str, #[case] value: Value, #[case] valid: bool) {
    assert_limit(path, "number", value, valid, Rule::RoleConstraint);
}

#[rstest]
#[case::target_minimum_at("target-minimum", px(24.0), true)]
#[case::target_minimum_below("target-minimum", px(23.0), false)]
#[case::target_minimum_above("target-minimum", px(25.0), true)]
#[case::target_touch_at("target-touch", px(44.0), true)]
#[case::target_touch_below("target-touch", px(43.0), false)]
#[case::target_touch_above("target-touch", px(45.0), true)]
#[case::target_touch_rem_at("target-touch", rem(2.75), true)]
#[case::target_touch_rem_below("target-touch", rem(2.6875), false)]
#[case::compact_at_minimum("control-compact", px(24.0), true)]
#[case::compact_below_minimum("control-compact", px(23.0), false)]
#[case::compact_at_comfortable("control-compact", px(40.0), true)]
#[case::compact_above_comfortable("control-compact", px(41.0), false)]
#[case::comfortable_at_compact("control-comfortable", px(34.0), true)]
#[case::comfortable_below_compact("control-comfortable", px(33.0), false)]
fn p3_size_limits(#[case] name: &str, #[case] value: Value, #[case] valid: bool) {
    assert_limit(
        &format!("semantic.size.{name}"),
        "dimension",
        value,
        valid,
        Rule::Layout,
    );
}

#[rstest]
#[case::gap_compact_at("semantic.density.gap-compact", px(16.0), true)]
#[case::gap_compact_above("semantic.density.gap-compact", px(17.0), false)]
#[case::gap_comfortable_at("semantic.density.gap-comfortable", px(12.0), true)]
#[case::gap_comfortable_below("semantic.density.gap-comfortable", px(11.0), false)]
#[case::padding_compact_at("semantic.density.padding-compact", px(20.0), true)]
#[case::padding_compact_above("semantic.density.padding-compact", px(21.0), false)]
#[case::padding_comfortable_below("semantic.density.padding-comfortable", px(11.0), false)]
#[case::sm_zero("semantic.breakpoint.breakpoint-sm", px(0.0), false)]
#[case::sm_one("semantic.breakpoint.breakpoint-sm", px(1.0), true)]
#[case::sm_below_md("semantic.breakpoint.breakpoint-sm", px(959.0), true)]
#[case::sm_at_md("semantic.breakpoint.breakpoint-sm", px(960.0), false)]
#[case::md_above_sm("semantic.breakpoint.breakpoint-md", px(641.0), true)]
#[case::md_at_sm("semantic.breakpoint.breakpoint-md", px(640.0), false)]
#[case::md_below_lg("semantic.breakpoint.breakpoint-md", px(1279.0), true)]
#[case::md_at_lg("semantic.breakpoint.breakpoint-md", px(1280.0), false)]
#[case::lg_above_md("semantic.breakpoint.breakpoint-lg", px(961.0), true)]
#[case::lg_at_md("semantic.breakpoint.breakpoint-lg", px(960.0), false)]
#[case::focus_at("semantic.border-width.border-focus", px(2.0), true)]
#[case::focus_below("semantic.border-width.border-focus", px(1.0), false)]
#[case::focus_above("semantic.border-width.border-focus", px(3.0), true)]
#[case::offset_at("semantic.layout.focus-offset", px(2.0), true)]
#[case::offset_below("semantic.layout.focus-offset", px(1.0), false)]
#[case::offset_above("semantic.layout.focus-offset", px(3.0), true)]
fn p3_density_breakpoint_and_focus_limits(
    #[case] path: &str,
    #[case] value: Value,
    #[case] valid: bool,
) {
    assert_limit(path, "dimension", value, valid, Rule::Layout);
}

#[rstest]
#[case::at_two("{semantic.border-width.border-active}", true)]
#[case::below_two("{semantic.border-width.border-hairline}", false)]
fn p3_button_focus_width(#[case] binding: &str, #[case] valid: bool) {
    assert_limit(
        "component.button.primary.focus-width",
        "dimension",
        json!(binding),
        valid,
        Rule::Layout,
    );
}

#[rstest]
#[case::base_below_sticky("base", 9.0, true)]
#[case::base_at_sticky("base", 10.0, false)]
#[case::sticky_above_base("sticky", 1.0, true)]
#[case::sticky_at_base("sticky", 0.0, false)]
#[case::sticky_below_popover("sticky", 49.0, true)]
#[case::sticky_at_popover("sticky", 50.0, false)]
#[case::popover_below_modal("popover", 59.0, true)]
#[case::popover_at_modal("popover", 60.0, false)]
#[case::modal_below_toast("modal", 69.0, true)]
#[case::modal_at_toast("modal", 70.0, false)]
#[case::toast_below_tooltip("toast", 79.0, true)]
#[case::toast_at_tooltip("toast", 80.0, false)]
#[case::tooltip_above_toast("tooltip", 71.0, true)]
#[case::tooltip_at_toast("tooltip", 70.0, false)]
fn p3_z_index_order(#[case] name: &str, #[case] value: f64, #[case] valid: bool) {
    assert_limit(
        &format!("semantic.z-index.z-{name}"),
        "number",
        json!(value),
        valid,
        Rule::Layout,
    );
}

#[rstest]
#[case::none_zero("none", ms(0.0), true)]
#[case::none_one("none", ms(1.0), false)]
#[case::fast_zero("fast", ms(0.0), false)]
#[case::fast_one("fast", ms(1.0), true)]
#[case::fast_at_base("fast", ms(180.0), true)]
#[case::fast_above_base("fast", ms(181.0), false)]
#[case::base_at_fast("base", ms(120.0), true)]
#[case::base_below_fast("base", ms(119.0), false)]
#[case::base_at_slow("base", ms(260.0), true)]
#[case::base_above_slow("base", ms(261.0), false)]
#[case::slow_at_base("slow", ms(180.0), true)]
#[case::slow_below_base("slow", ms(179.0), false)]
#[case::slow_below_limit("slow", ms(299.0), true)]
#[case::slow_at_limit("slow", ms(300.0), true)]
#[case::slow_seconds("slow", seconds(0.26), true)]
#[case::slow_seconds_at_limit("slow", seconds(0.3), true)]
#[case::slow_seconds_above_limit("slow", seconds(0.301), false)]
fn p3_duration_limits(#[case] name: &str, #[case] value: Value, #[case] valid: bool) {
    assert_limit(
        &format!("semantic.duration.duration-{name}"),
        "duration",
        value,
        valid,
        Rule::Layout,
    );
}

// ----- P9: deprecation warnings ---------------------------------------------

/// A theme that deprecates `semantic.spacing.step-6` (a definition only).
fn deprecating_theme(f: &Fixture) -> PathBuf {
    layer(
        f.root(),
        "deprecating.tokens.json",
        &[(
            "semantic.spacing.step-6",
            json!({"$type": "dimension", "$value": px(24.0), "$deprecated": "Use step-8."}),
        )],
    )
}

/// `(location, severity)` of every deprecation warning.
fn deprecations(c: &Composition) -> Vec<(String, Severity)> {
    c.warnings()
        .iter()
        .filter(|d| d.rule == Rule::Deprecated)
        .map(|d| (d.location.clone(), d.severity))
        .collect()
}

fn warned(locations: &[&str]) -> Vec<(String, Severity)> {
    locations
        .iter()
        .map(|l| ((*l).to_owned(), Severity::Warning))
        .collect()
}

#[rstest]
#[case::alias("semantic.layout.page-gutter", tok("dimension", json!("{semantic.spacing.step-6}")))]
#[case::pointer("semantic.layout.page-gutter", tok("dimension", json!({"$ref": "#/semantic/spacing/step-6/$value"})))]
#[case::property_pointer("semantic.layout.page-gutter", tok("dimension", json!({
    "value": {"$ref": "#/semantic/spacing/step-6/$value/value"}, "unit": "px"
})))]
#[case::composite_fields_once("semantic.typography.body", tok("typography", json!({
    "fontFamily": "{semantic.font.sans}",
    "fontSize": "{semantic.spacing.step-6}",
    "fontWeight": 400,
    "letterSpacing": {"$ref": "#/semantic/spacing/step-6/$value"},
    "lineHeight": 1.5
})))]
#[case::assignment("semantic.spacing.step-6", tok("dimension", px(24.0)))]
#[case::assignment_undeprecating("semantic.spacing.step-6", json!({
    "$type": "dimension", "$value": px(24.0), "$deprecated": false
}))]
fn p9_a_layer_using_or_assigning_a_deprecated_path_warns_once(
    #[case] path: &str,
    #[case] token: Value,
) {
    let f = fixture();
    let theme = deprecating_theme(&f);
    let file = layer(f.root(), "o.tokens.json", &[(path, token)]);
    let c = composed(&f, &selection(Mode::Night, Some(theme), &[file]));
    assert_eq!(
        deprecations(&c),
        warned(&[&format!("o.tokens.json: {path}")])
    );
}

#[test]
fn p9_definitions_alone_never_warn() {
    let f = fixture();
    let c = composed(
        &f,
        &selection(Mode::Night, Some(deprecating_theme(&f)), &[]),
    );
    assert_eq!(deprecations(&c), []);
}

#[test]
fn p9_once_per_layer_token_and_target() {
    let f = fixture();
    let theme = deprecating_theme(&f);
    let uses = [
        (
            "semantic.layout.page-gutter",
            tok("dimension", json!("{semantic.spacing.step-6}")),
        ),
        (
            "semantic.layout.page-gutter-wide",
            tok("dimension", json!("{semantic.spacing.step-6}")),
        ),
    ];
    let first = layer(f.root(), "first.tokens.json", &uses);
    let second = layer(f.root(), "second.tokens.json", &uses[..1]);
    let revive = layer(
        f.root(),
        "revive.tokens.json",
        &[(
            "semantic.spacing.step-6",
            json!({"$type": "dimension", "$value": px(24.0), "$deprecated": false}),
        )],
    );
    // A path stays deprecated once deprecated, even after `$deprecated: false`.
    let again = layer(
        f.root(),
        "again.tokens.json",
        &[("semantic.spacing.step-6", tok("dimension", px(24.0)))],
    );
    let c = composed(
        &f,
        &selection(Mode::Night, Some(theme), &[first, second, revive, again]),
    );
    assert_eq!(
        deprecations(&c),
        warned(&[
            "first.tokens.json: semantic.layout.page-gutter",
            "first.tokens.json: semantic.layout.page-gutter-wide",
            "second.tokens.json: semantic.layout.page-gutter",
            "revive.tokens.json: semantic.spacing.step-6",
            "again.tokens.json: semantic.spacing.step-6",
        ])
    );
}

#[test]
fn p9_earlier_layer_uses_warn_for_that_layer_only() {
    let f = fixture();
    edit(&f, BASE, |base| {
        base["semantic"]["spacing"]["step-6"]["$deprecated"] = json!("Use step-8.");
        base["semantic"]["layout"]["page-gutter"]["$value"] = json!("{semantic.spacing.step-6}");
    });
    let unrelated = layer(
        f.root(),
        "unrelated.tokens.json",
        &[("semantic.spacing.step-3", tok("dimension", px(12.0)))],
    );
    // Deprecating a path the base uses is a definition, not a use.
    let deprecate_used = layer(
        f.root(),
        "deprecate.tokens.json",
        &[(
            "semantic.spacing.step-4",
            json!({"$type": "dimension", "$value": "{primitive.spacing.space-4}", "$deprecated": true}),
        )],
    );
    let rebind = layer(
        f.root(),
        "rebind.tokens.json",
        &[(
            "component.button.primary.padding-inline",
            tok("dimension", json!("{semantic.spacing.step-4}")),
        )],
    );
    let c = composed(
        &f,
        &selection(Mode::Night, None, &[unrelated, deprecate_used, rebind]),
    );
    assert_eq!(
        deprecations(&c),
        warned(&[
            "tokens/base.tokens.json: semantic.layout.page-gutter",
            "rebind.tokens.json: component.button.primary.padding-inline",
        ])
    );
}

#[test]
fn p9_a_default_theme_replaced_by_theme_does_not_warn() {
    let f = fixture();
    edit(&f, BASE, |base| {
        base["semantic"]["spacing"]["step-6"]["$deprecated"] = json!(true);
    });
    layer(
        f.root(),
        DEFAULT_THEME,
        &[(
            "semantic.layout.page-gutter",
            tok("dimension", json!("{semantic.spacing.step-6}")),
        )],
    );
    let default = composed(&f, &selection(Mode::Night, None, &[]));
    assert_eq!(
        deprecations(&default),
        warned(&[&format!("{DEFAULT_THEME}: semantic.layout.page-gutter")])
    );
    let fern = f.path("tokens/themes/fern.tokens.json");
    let replaced = composed(&f, &selection(Mode::Night, Some(fern), &[]));
    assert_eq!(deprecations(&replaced), []);
}
