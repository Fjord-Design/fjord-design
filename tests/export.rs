//! Integration tests for the exports (O1–O3) and their determinism.
//!
//! Exports are rendered from real compositions: the repository's own
//! sources (read only) or temporary copies with extra layers.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use common::{Fixture, layer, layer_document, repo, rules};
use fjord::diagnostic::Rule;
use fjord::dtcg::graph::resolve;
use fjord::dtcg::value::{Color, TypedValue};
use fjord::export::{Format, render};
use fjord::json::same;
use fjord::policy::contract::Mode;
use fjord::project::{Composition, Project, Selection, validate_tokens};
use rstest::rstest;
use serde_json::{Map, Value, json};

const BASE: &str = "tokens/base.tokens.json";
const EXTENSION: &str = "org.xerrion.fjord";
const VENDOR: &str = "com.example.vendor";
const THEME_FILE: &str = "vendor-theme.tokens.json";
const OVERRIDE_FILE: &str = "vendor-override.tokens.json";

/// CSS Fonts 4 generic families plus the two system aliases written bare (L14).
const BARE_FAMILIES: [&str; 15] = [
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui",
    "ui-serif",
    "ui-sans-serif",
    "ui-monospace",
    "ui-rounded",
    "math",
    "emoji",
    "fangsong",
    "-apple-system",
    "BlinkMacSystemFont",
];

// ---------------------------------------------------------------------------
// Helpers

fn open(root: &std::path::Path) -> Project {
    Project::open(root).unwrap_or_else(|failure| panic!("{failure}"))
}

fn compose(
    project: &Project,
    mode: Mode,
    theme: Option<PathBuf>,
    overrides: Vec<PathBuf>,
) -> Composition {
    let selection = Selection {
        mode,
        theme,
        overrides,
    };
    project
        .compose(&selection)
        .unwrap_or_else(|failure| panic!("{}:\n{failure}", selection.label()))
}

fn rendered(c: &Composition, format: Format) -> Vec<u8> {
    render(c, format).unwrap_or_else(|failure| panic!("{format:?}:\n{failure}"))
}

fn export_json(c: &Composition, format: Format) -> Value {
    serde_json::from_slice(&rendered(c, format)).unwrap()
}

fn read_json(f: &Fixture, rel: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(f.path(rel)).unwrap()).unwrap()
}

fn write_json(path: &std::path::Path, value: &Value) {
    fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

fn edit_base(f: &Fixture, edit: impl FnOnce(&mut Value)) {
    let mut base = read_json(f, BASE);
    edit(&mut base);
    write_json(&f.path(BASE), &base);
}

/// The node at a dotted path (`$root` and other `$` keys included).
fn at<'v>(value: &'v Value, dotted: &str) -> &'v Value {
    dotted.split('.').fold(value, |node, key| &node[key])
}

/// Every token object of a DTCG document with its dotted path, in document order.
fn dtcg_tokens(doc: &Map<String, Value>) -> Vec<(String, &Map<String, Value>)> {
    fn walk<'d>(
        group: &'d Map<String, Value>,
        prefix: &str,
        out: &mut Vec<(String, &'d Map<String, Value>)>,
    ) {
        for (key, value) in group {
            if key.starts_with('$') && key != "$root" {
                continue;
            }
            let Value::Object(node) = value else { continue };
            let path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            if node.contains_key("$value") {
                out.push((path, node));
            } else {
                walk(node, &path, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(doc, "", &mut out);
    out
}

/// `--fjord-token-` + path with `.` → `-` and `$` → `\$`.
fn property(path: &str) -> String {
    format!(
        "--fjord-token-{}",
        path.replace('.', "-").replace('$', "\\$")
    )
}

/// The declarations of a stylesheet; panics on a repeated property.
fn declarations(css: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in css.lines() {
        let Some(declaration) = line.strip_prefix("  ") else {
            continue;
        };
        let (name, value) = declaration
            .strip_suffix(';')
            .and_then(|d| d.split_once(": "))
            .unwrap_or_else(|| panic!("malformed declaration {line:?}"));
        assert!(
            out.insert(name.to_owned(), value.to_owned()).is_none(),
            "{name} declared twice"
        );
    }
    out
}

fn css_of(c: &Composition) -> BTreeMap<String, String> {
    declarations(&String::from_utf8(rendered(c, Format::Css)).unwrap())
}

/// 8-bit sRGB HEX, half-up rounding, alpha byte only below 255.
fn hex(color: &Color) -> String {
    let byte = |v: f64| {
        let scaled = (v * 255.0).round();
        (0..=255_u8)
            .find(|b| (f64::from(*b) - scaled).abs() < 0.5)
            .unwrap_or_else(|| panic!("{v} is outside 0..=1"))
    };
    let [r, g, b] = color.components.map(byte);
    let alpha = byte(color.alpha);
    if alpha < 255 {
        format!("#{r:02x}{g:02x}{b:02x}{alpha:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}

fn family_list(families: &[&str]) -> String {
    families
        .iter()
        .map(|f| {
            if BARE_FAMILIES.contains(f) {
                (*f).to_owned()
            } else {
                format!("\"{f}\"")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Re-resolve the DTCG export alone and compare it with the composition.
fn assert_export_resolves_to_composition(c: &Composition) {
    let Value::Object(doc) = export_json(c, Format::Dtcg) else {
        panic!("{}: the export is not an object", c.label())
    };
    let again =
        resolve(&doc, "export").unwrap_or_else(|failure| panic!("{}:\n{failure}", c.label()));
    let exported: BTreeSet<&str> = again.iter().map(|(path, _)| &**path).collect();
    let composed: BTreeSet<&str> = c.tokens().map(|(path, _)| &**path).collect();
    assert_eq!(exported, composed, "{}: token sets differ", c.label());
    for (path, token) in c.tokens() {
        let other = again.get(path).unwrap();
        assert_eq!(other.ty, token.ty, "{}: {path}", c.label());
        assert!(
            same(&other.json, &token.json),
            "{}: {path}: {} != {}",
            c.label(),
            other.json,
            token.json
        );
        assert_eq!(other.value, token.value, "{}: {path}", c.label());
    }
}

/// A token definition taken from the fixture's base file, with a vendor extension.
fn base_token_with_vendor(f: &Fixture, path: &str, mark: &str) -> Value {
    let mut token = at(&read_json(f, BASE), path).clone();
    token["$extensions"] = json!({VENDOR: {"layer": mark}});
    token
}

/// Write a layer whose root and `semantic.spacing` group carry `{VENDOR: {mark: true}}`.
fn vendor_layer(f: &Fixture, name: &str, mark: &str, tokens: &[(&str, Value)]) -> PathBuf {
    let mut doc = layer_document(tokens);
    doc["$extensions"][VENDOR] = json!({mark: true});
    doc["semantic"]["spacing"]["$extensions"] = json!({VENDOR: {mark: true}});
    let path = f.path(name);
    write_json(&path, &doc);
    path
}

/// A copy of `tokens/` whose base adds:
/// - vendor extensions at root, group (`primitive.spacing`) and token
///   (`primitive.spacing.space-4`) level;
/// - `primitive.spacing-alias`, which `$extends` `primitive.spacing` and
///   replaces `space-2` locally;
/// - `primitive.test-root` with a `$root` token and a child named `root`,
///   both typed by the group;
/// - `primitive.untyped.alias`, typed only by its alias target.
///
/// It also writes a theme (vendor marks + `semantic.spacing.step-4`) and an
/// override (vendor marks + `semantic.spacing.step-2`); the theme's Fjord
/// metadata declares contract 0.5.3 and an extra key.
struct Rich {
    f: Fixture,
    theme: PathBuf,
    override_file: PathBuf,
}

fn rich() -> Rich {
    let f = Fixture::project(&["tokens"]);
    edit_base(&f, |base| {
        base["$extensions"][VENDOR] = json!({"base": true});
        let primitive = base["primitive"].as_object_mut().unwrap();
        primitive["spacing"]["$extensions"] = json!({VENDOR: {"layer": "base"}});
        primitive["spacing"]["space-4"]["$extensions"] = json!({VENDOR: {"layer": "base"}});
        primitive.insert(
            "spacing-alias".to_owned(),
            json!({
                "$description": "Spacing scale inherited through $extends.",
                "$extends": "{primitive.spacing}",
                "space-2": {
                    "$type": "dimension",
                    "$value": {"value": 5, "unit": "px"},
                    "$description": "Local replacement of an inherited token."
                }
            }),
        );
        primitive.insert(
            "test-root".to_owned(),
            json!({
                "$type": "dimension",
                "$root": {"$value": {"value": 2, "unit": "px"}, "$description": "The group's own value."},
                "root": {"$value": {"value": 3, "unit": "px"}, "$description": "A child named root."}
            }),
        );
        primitive.insert(
            "untyped".to_owned(),
            json!({
                "alias": {"$value": "{primitive.duration.duration-fast}", "$description": "Typed by its target."}
            }),
        );
    });
    let step4 = base_token_with_vendor(&f, "semantic.spacing.step-4", "theme");
    let theme = vendor_layer(
        &f,
        THEME_FILE,
        "theme",
        &[("semantic.spacing.step-4", step4)],
    );
    let mut doc = read_json(&f, THEME_FILE);
    doc["$extensions"][EXTENSION]["contractVersion"] = json!("0.5.3");
    doc["$extensions"][EXTENSION]["themeNote"] = json!("kept");
    write_json(&theme, &doc);
    let step2 = base_token_with_vendor(&f, "semantic.spacing.step-2", "override");
    let override_file = vendor_layer(
        &f,
        OVERRIDE_FILE,
        "override",
        &[("semantic.spacing.step-2", step2)],
    );
    Rich {
        f,
        theme,
        override_file,
    }
}

impl Rich {
    fn compose(&self, mode: Mode) -> Composition {
        let project = open(self.f.root());
        compose(
            &project,
            mode,
            Some(self.theme.clone()),
            vec![self.override_file.clone()],
        )
    }
}

// ---------------------------------------------------------------------------
// O1 DTCG

#[test]
fn o1_every_official_composition_round_trips_through_the_dtcg_export() {
    let project = open(&repo());
    let mut themes = vec![None];
    themes.extend(project.official_themes().unwrap().into_iter().map(Some));
    assert!(themes.len() > 1, "no official themes found");
    let selections: Vec<Selection> = Mode::ALL
        .iter()
        .flat_map(|&mode| {
            themes.iter().map(move |theme| Selection {
                mode,
                theme: theme.clone(),
                overrides: Vec::new(),
            })
        })
        .collect();
    let compositions =
        validate_tokens(&project, &selections).unwrap_or_else(|failure| panic!("{failure}"));
    assert_eq!(compositions.len(), 3 * themes.len());
    for c in &compositions {
        assert_export_resolves_to_composition(c);
    }
}

#[test]
fn o1_layered_composition_round_trips_through_the_dtcg_export() {
    let rich = rich();
    for mode in Mode::ALL {
        assert_export_resolves_to_composition(&rich.compose(mode));
    }
}

#[test]
fn o1_vendor_extensions_survive_theme_and_override() {
    let rich = rich();
    let export = export_json(&rich.compose(Mode::Night), Format::Dtcg);
    // Root: deep-merged across base, theme and override.
    assert_eq!(
        export["$extensions"][VENDOR],
        json!({"base": true, "theme": true, "override": true})
    );
    // Group: base's group untouched; the semantic group merges both later layers.
    assert_eq!(
        at(&export, "primitive.spacing.$extensions")[VENDOR],
        json!({"layer": "base"})
    );
    assert_eq!(
        at(&export, "semantic.spacing.$extensions")[VENDOR],
        json!({"theme": true, "override": true})
    );
    // Token: each layer's own token keeps its mark.
    for (path, mark) in [
        ("primitive.spacing.space-4", "base"),
        ("semantic.spacing.step-4", "theme"),
        ("semantic.spacing.step-2", "override"),
    ] {
        assert_eq!(
            at(&export, path)["$extensions"][VENDOR],
            json!({"layer": mark}),
            "{path}"
        );
    }
}

#[test]
fn o1_fjord_metadata_is_deep_merged_with_the_export_metadata() {
    let rich = rich();
    let base_metadata = read_json(&rich.f, BASE)["$extensions"][EXTENSION].clone();
    for mode in Mode::ALL {
        let export = export_json(&rich.compose(mode), Format::Dtcg);
        let metadata = &export["$extensions"][EXTENSION];
        assert_eq!(metadata["schemaVersion"], json!(2));
        // The theme declared 0.5.3; the export states the contract it was built for.
        assert_eq!(metadata["contractVersion"], json!("0.5.0"));
        assert_eq!(metadata["dtcgVersion"], json!("2025.10"));
        assert_eq!(metadata["mode"], json!(mode.as_str()));
        assert_eq!(metadata["colorSpace"], json!("srgb"));
        // Other keys of the composed metadata are kept.
        assert_eq!(metadata["themeNote"], json!("kept"));
        for key in ["name", "designSeeds", "fontAssets", "iconAssets"] {
            assert!(same(&metadata[key], &base_metadata[key]), "{key}");
        }
    }
}

#[test]
fn o1_metadata_on_the_shipped_export() {
    let project = open(&repo());
    let c = compose(&project, Mode::Aurora, None, Vec::new());
    let export = export_json(&c, Format::Dtcg);
    let metadata = export["$extensions"][EXTENSION].as_object().unwrap();
    for (key, value) in [
        ("schemaVersion", json!(2)),
        ("contractVersion", json!("0.5.0")),
        ("dtcgVersion", json!("2025.10")),
        ("mode", json!("aurora")),
        ("colorSpace", json!("srgb")),
        ("fontAssets", json!("assets/fonts/manifest.json")),
        ("iconAssets", json!("assets/icons/manifest.json")),
    ] {
        assert_eq!(metadata.get(key), Some(&value), "{key}");
    }
}

#[test]
fn o1_every_token_carries_its_effective_type() {
    let rich = rich();
    let c = rich.compose(Mode::Day);
    let export = export_json(&c, Format::Dtcg);
    let tokens = dtcg_tokens(export.as_object().unwrap());
    assert_eq!(tokens.len(), c.tokens().count());
    for (path, token) in &tokens {
        let expected = c
            .get(path)
            .unwrap_or_else(|| panic!("{path} not composed"))
            .ty;
        assert_eq!(
            token.get("$type"),
            Some(&json!(expected.as_str())),
            "{path}"
        );
    }
    // Types not written on the token: from the alias target (stamped as the
    // first key) and from the group.
    let alias = at(&export, "primitive.untyped.alias").as_object().unwrap();
    assert_eq!(alias.keys().next().map(String::as_str), Some("$type"));
    assert_eq!(alias["$type"], json!("duration"));
    for path in ["primitive.test-root.$root", "primitive.test-root.root"] {
        assert_eq!(at(&export, path)["$type"], json!("dimension"), "{path}");
    }
}

#[test]
fn o1_extends_is_kept_and_inherited_tokens_are_materialized() {
    let rich = rich();
    let c = rich.compose(Mode::Night);
    let export = export_json(&c, Format::Dtcg);
    let group = at(&export, "primitive.spacing-alias");
    assert_eq!(group["$extends"], json!("{primitive.spacing}"));
    // Every inherited token is written out, with its definition.
    let source = at(&export, "primitive.spacing").as_object().unwrap();
    for (name, token) in source
        .iter()
        .filter(|(k, v)| !k.starts_with('$') && v.get("$value").is_some())
    {
        let materialized = &group[name];
        assert!(materialized.is_object(), "spacing-alias.{name} is missing");
        if name == "space-2" {
            assert!(same(
                &materialized["$value"],
                &json!({"value": 5, "unit": "px"})
            ));
        } else {
            assert!(same(&materialized["$value"], &token["$value"]), "{name}");
        }
    }
    assert_eq!(
        at(&export, "primitive.spacing-alias.space-4.$extensions")[VENDOR],
        json!({"layer": "base"})
    );
    assert!(c.get("primitive.spacing-alias.space-4").is_some());
}

// ---------------------------------------------------------------------------
// O2 resolved JSON

fn is_source_label(label: &str) -> bool {
    if label == "base" {
        return true;
    }
    if let Some(mode) = label.strip_prefix("mode:") {
        return Mode::ALL.iter().any(|m| m.as_str() == mode);
    }
    if let Some(file) = label.strip_prefix("theme:") {
        return !file.is_empty() && !file.contains('/');
    }
    if let Some(rest) = label.strip_prefix("override:") {
        return rest.split_once(':').is_some_and(|(n, file)| {
            !n.is_empty()
                && n.bytes().all(|b| b.is_ascii_digit())
                && !n.starts_with('0')
                && !file.is_empty()
                && !file.contains('/')
        });
    }
    false
}

#[test]
fn o2_resolved_schema_order_sources_and_originals() {
    let rich = rich();
    let pointer = layer(
        rich.f.root(),
        "pointer.tokens.json",
        &[(
            "semantic.spacing.step-3",
            json!({"$type": "dimension", "$value": {"$ref": "#/primitive/spacing/space-3/$value"}}),
        )],
    );
    let project = open(rich.f.root());
    let c = compose(
        &project,
        Mode::Night,
        Some(rich.theme.clone()),
        vec![pointer, rich.override_file.clone()],
    );
    let resolved = export_json(&c, Format::Resolved);
    let top: Vec<&str> = resolved
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(top, ["metadata", "tokens"]);
    assert_eq!(
        resolved["metadata"],
        json!({"schemaVersion": 2, "contractVersion": "0.5.0", "dtcgVersion": "2025.10", "mode": "night", "colorSpace": "srgb"})
    );

    let tokens = resolved["tokens"].as_object().unwrap();
    let paths: Vec<&String> = tokens.keys().collect();
    let mut sorted = paths.clone();
    sorted.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    assert_eq!(paths, sorted, "token paths are not in byte order");
    assert_eq!(tokens.len(), c.tokens().count());

    for (path, record) in tokens {
        let keys: BTreeSet<&str> = record
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            BTreeSet::from(["type", "value", "source", "original"]),
            "{path}"
        );
        let token = c.get(path).unwrap();
        assert_eq!(record["type"], json!(token.ty.as_str()), "{path}");
        assert!(same(&record["value"], &token.json), "{path}");
        let source = record["source"]
            .as_str()
            .unwrap_or_else(|| panic!("{path}: no source"));
        assert!(is_source_label(source), "{path}: {source}");
    }

    for (path, source) in [
        ("primitive.spacing.space-3", "base"),
        ("primitive.spacing-alias.space-4", "base"),
        ("semantic.color.ink", "mode:night"),
        ("semantic.spacing.step-4", "theme:vendor-theme.tokens.json"),
        ("semantic.spacing.step-3", "override:1:pointer.tokens.json"),
        (
            "semantic.spacing.step-2",
            "override:2:vendor-override.tokens.json",
        ),
    ] {
        assert_eq!(tokens[path]["source"], json!(source), "{path}");
    }

    let step3 = &tokens["semantic.spacing.step-3"];
    assert_eq!(
        step3["original"],
        json!({"$ref": "#/primitive/spacing/space-3/$value"})
    );
    assert!(same(
        &step3["value"],
        &tokens["primitive.spacing.space-3"]["value"]
    ));
    assert!(step3["value"].is_object());
    // A brace alias stays as authored too.
    assert_eq!(
        tokens["semantic.spacing.step-4"]["original"],
        json!("{primitive.spacing.space-4}")
    );
}

// ---------------------------------------------------------------------------
// O3 CSS

#[test]
fn o3_shipped_css_values() {
    let project = open(&repo());
    for mode in Mode::ALL {
        let c = compose(&project, mode, None, Vec::new());
        let css = css_of(&c);
        let mut translucent = 0;
        for (path, token) in c.tokens() {
            let name = property(path);
            match &token.value {
                TypedValue::Typography(_) => {
                    assert!(!css.contains_key(&name), "{name} not expanded");
                    for field in [
                        "font-family",
                        "font-size",
                        "font-weight",
                        "letter-spacing",
                        "line-height",
                    ] {
                        assert!(
                            css.contains_key(&format!("{name}-{field}")),
                            "{name}-{field}"
                        );
                    }
                }
                TypedValue::Color(color) => {
                    let expected = hex(color);
                    translucent += usize::from(expected.len() == 9);
                    assert_eq!(css[&name], expected, "{path}");
                }
                TypedValue::FontFamily(families) => {
                    let families: Vec<&str> = families.iter().map(String::as_str).collect();
                    assert_eq!(css[&name], family_list(&families), "{path}");
                }
                _ => assert!(css.contains_key(&name), "{name}"),
            }
        }
        assert!(translucent > 0, "{mode:?}: no colour with alpha below 1");
        assert_eq!(css[&property("semantic.color.scrim")].len(), 9);
        for unit in ["px", "rem", "ms", "s"] {
            for (name, value) in &css {
                assert!(!value.contains(&format!(".0{unit}")), "{name}: {value}");
            }
        }
        let body = property("semantic.typography.body");
        assert_eq!(css[&format!("{body}-font-size")], "15px");
        assert_eq!(css[&format!("{body}-font-weight")], "400");
        assert_eq!(css[&format!("{body}-letter-spacing")], "0px");
        assert_eq!(css[&format!("{body}-line-height")], "1.6");
        assert_eq!(
            css[&format!("{body}-font-family")],
            r#""Noto Sans", "Open Sans", system-ui, -apple-system, "Segoe UI", sans-serif"#
        );
        assert_eq!(
            css[&property("semantic.font.mono")],
            r#""Cascadia Code", "Fira Code", "JetBrains Mono", ui-monospace, "Menlo", "Consolas", monospace"#
        );
    }
}

#[test]
fn o3_shipped_layered_shadow_renders_comma_joined() {
    let project = open(&repo());
    let c = compose(&project, Mode::Night, None, Vec::new());
    assert_eq!(
        css_of(&c)[&property("semantic.shadow.shadow-sm")],
        "inset 0px 1px 0px 0px #ffffff0f, 0px 1px 2px 0px #03060a59"
    );
}

#[test]
fn o3_dollar_root_and_child_named_root_get_distinct_properties() {
    let rich = rich();
    let css = css_of(&rich.compose(Mode::Night));
    assert_eq!(css["--fjord-token-primitive-test-root-\\$root"], "2px");
    assert_eq!(css["--fjord-token-primitive-test-root-root"], "3px");
}

#[test]
fn o3_single_shadow_object_and_single_string_family() {
    let f = Fixture::project(&["tokens"]);
    let file = layer(
        f.root(),
        "single.tokens.json",
        &[
            (
                "semantic.shadow.shadow-sm",
                json!({"$type": "shadow", "$value": {
                    "color": {"colorSpace": "srgb", "components": [0, 0, 0], "alpha": 0.5},
                    "offsetX": {"value": 0, "unit": "px"},
                    "offsetY": {"value": 2.0, "unit": "px"},
                    "blur": {"value": 4.5, "unit": "px"},
                    "spread": {"value": -1, "unit": "px"}
                }}),
            ),
            (
                "semantic.font.sans",
                json!({"$type": "fontFamily", "$value": "Noto Sans"}),
            ),
        ],
    );
    let c = compose(&open(f.root()), Mode::Day, None, vec![file]);
    let css = css_of(&c);
    assert_eq!(
        css[&property("semantic.shadow.shadow-sm")],
        "0px 2px 4.5px -1px #00000080"
    );
    for path in ["semantic.font.sans", "component.button.primary.font-family"] {
        assert_eq!(css[&property(path)], "\"Noto Sans\"", "{path}");
    }
    assert_eq!(
        css[&format!("{}-font-family", property("semantic.typography.body"))],
        "\"Noto Sans\""
    );
}

#[test]
fn o3_generic_family_keywords_stay_bare_and_names_are_quoted() {
    let f = Fixture::project(&["tokens"]);
    let mut stack = vec!["Raleway"];
    stack.extend(BARE_FAMILIES);
    stack.extend(["Segoe UI", "Helvetica Neue", "Menlo"]);
    let file = layer(
        f.root(),
        "generic.tokens.json",
        &[(
            "semantic.font.display",
            json!({"$type": "fontFamily", "$value": stack}),
        )],
    );
    let c = compose(&open(f.root()), Mode::Night, None, vec![file]);
    let expected = format!(
        "\"Raleway\", {}, \"Segoe UI\", \"Helvetica Neue\", \"Menlo\"",
        BARE_FAMILIES.join(", ")
    );
    let css = css_of(&c);
    assert_eq!(css[&property("semantic.font.display")], expected);
    assert_eq!(
        css[&format!("{}-font-family", property("semantic.typography.display"))],
        expected
    );
}

#[rstest]
#[case::dash_and_dot(&[
    ("primitive.collide.b-c", json!({"$type": "dimension", "$value": {"value": 1, "unit": "px"}, "$description": "One."})),
    ("primitive.collide-b.c", json!({"$type": "dimension", "$value": {"value": 2, "unit": "px"}, "$description": "Two."})),
])]
#[case::typography_expansion(&[
    ("primitive.collide-type", json!({"$type": "typography", "$value": {
        "fontFamily": ["Noto Sans", "sans-serif"],
        "fontSize": {"value": 15, "unit": "px"},
        "fontWeight": 400,
        "letterSpacing": {"value": 0, "unit": "px"},
        "lineHeight": 1.5
    }, "$description": "Expands to -font-size among others."})),
    ("primitive.collide-type-font-size", json!({"$type": "dimension", "$value": {"value": 2, "unit": "px"}, "$description": "Same property name."})),
])]
fn o3_css_name_collision_fails(#[case] tokens: &[(&str, Value)]) {
    let f = Fixture::project(&["tokens"]);
    edit_base(&f, |base| {
        let primitive = base["primitive"].as_object_mut().unwrap();
        for (path, token) in tokens {
            let mut segments = path.split('.').skip(1).peekable();
            let mut node = &mut *primitive;
            while let Some(segment) = segments.next() {
                if segments.peek().is_none() {
                    node.insert(segment.to_owned(), token.clone());
                } else {
                    node = node
                        .entry(segment)
                        .or_insert_with(|| json!({}))
                        .as_object_mut()
                        .unwrap();
                }
            }
        }
    });
    let c = compose(&open(f.root()), Mode::Night, None, Vec::new());
    let failure = render(&c, Format::Css).unwrap_err();
    assert_eq!(rules(&failure), [Rule::CssName]);
    // The JSON formats have no property names to collide.
    rendered(&c, Format::Dtcg);
    rendered(&c, Format::Resolved);
}

// ---------------------------------------------------------------------------
// Determinism

#[rstest]
#[case::dtcg(Format::Dtcg)]
#[case::resolved(Format::Resolved)]
#[case::css(Format::Css)]
fn determinism_same_composition_gives_identical_bytes(#[case] format: Format) {
    let rich = rich();
    for mode in Mode::ALL {
        let first = rich.compose(mode);
        let second = rich.compose(mode);
        let bytes = rendered(&first, format);
        assert_eq!(bytes, rendered(&first, format), "{mode:?}: render twice");
        assert_eq!(bytes, rendered(&second, format), "{mode:?}: compose twice");
        assert!(bytes.ends_with(b"\n"));
    }
}
