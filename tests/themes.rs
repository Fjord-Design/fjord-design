//! Themes (TC "Composition", L17): every official theme and every example
//! theme composes in every mode, official themes only recolour the accent
//! roles, and no theme rewires component definitions.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use fjord::dtcg::graph::ResolvedToken;
use fjord::json;
use fjord::policy::contract::Mode;
use fjord::project::{Composition, Project, Selection, validate_tokens};
use serde_json::Value;

use common::repo;

/// The semantic colours an official theme may change (TC "Composition"):
/// the five it aliases plus the two mode roles that follow them.
const ACCENT_ROLES: [&str; 7] = [
    "semantic.color.accent",
    "semantic.color.accent-hover",
    "semantic.color.accent-soft",
    "semantic.color.accent-ink",
    "semantic.color.selection",
    "semantic.color.accent-pressed",
    "semantic.color.focus-ring",
];

const COMPACT: &str = "examples/overrides/compact-spacing.tokens.json";

fn project() -> Project {
    Project::open(&repo()).unwrap()
}

fn selection(mode: Mode, theme: Option<&Path>, overrides: &[PathBuf]) -> Selection {
    Selection {
        mode,
        theme: theme.map(Path::to_path_buf),
        overrides: overrides.to_vec(),
    }
}

fn compose_all(project: &Project, selections: &[Selection]) -> Vec<Composition> {
    let compositions = match validate_tokens(project, selections) {
        Ok(c) => c,
        Err(failure) => panic!("{failure}"),
    };
    assert_eq!(compositions.len(), selections.len());
    for (c, s) in compositions.iter().zip(selections) {
        assert_eq!(c.mode(), s.mode);
        assert_eq!(c.label(), s.label());
        assert!(c.warnings().is_empty(), "{:?}", c.warnings());
    }
    compositions
}

/// `*.tokens.json` files of `dir`, sorted.
fn layer_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".tokens.json"))
        .collect();
    files.sort();
    files
}

fn by_path(c: &Composition) -> HashMap<&str, &ResolvedToken> {
    c.tokens().map(|(p, t)| (&**p, t)).collect()
}

/// Paths whose type or resolved value (compared numerically) differs. Both
/// compositions must hold the same token set.
fn changed(before: &Composition, after: &Composition) -> BTreeSet<String> {
    let (before, after) = (by_path(before), by_path(after));
    assert_eq!(
        before.keys().collect::<BTreeSet<_>>(),
        after.keys().collect::<BTreeSet<_>>(),
        "token sets differ"
    );
    before
        .iter()
        .filter(|(path, token)| {
            let other = after[**path];
            token.ty != other.ty || !json::same(&token.json, &other.json)
        })
        .map(|(path, _)| (*path).to_owned())
        .collect()
}

/// The raw `component` subtree of the materialized document.
fn components(c: &Composition) -> &Value {
    c.expanded_document()
        .get("component")
        .expect("component group")
}

fn assert_components_unchanged(default: &Composition, themed: &Composition) {
    assert!(
        json::same(components(default), components(themed)),
        "{}: component definitions changed",
        themed.label()
    );
}

#[test]
fn cli1_official_themes_are_every_layer_beside_the_default() {
    let project = project();
    let default = &project.contract().theme;
    let expected: Vec<PathBuf> = layer_files(default.parent().unwrap())
        .into_iter()
        .filter(|p| p != default)
        .collect();
    assert_ne!(expected, Vec::<PathBuf>::new());
    assert_eq!(project.official_themes().unwrap(), expected);
}

#[test]
fn composition_official_themes_differ_only_in_accent_roles() {
    let project = project();
    let themes = project.official_themes().unwrap();
    assert_ne!(themes, Vec::<PathBuf>::new());
    let mut selections = Vec::new();
    for mode in Mode::ALL {
        selections.push(project.default_selection(mode));
        selections.extend(themes.iter().map(|t| selection(mode, Some(t), &[])));
    }
    let compositions = compose_all(&project, &selections);

    for group in compositions.chunks(themes.len() + 1) {
        let (default, variants) = group.split_first().unwrap();
        // Component bindings that alias an accent role follow it.
        let bindings: BTreeSet<String> = default
            .tokens()
            .filter(|(path, token)| {
                path.starts_with("component.")
                    && token
                        .references
                        .iter()
                        .any(|r| ACCENT_ROLES.contains(&&**r))
            })
            .map(|(path, _)| path.to_string())
            .collect();
        assert!(
            bindings.contains("component.button.primary.background"),
            "{bindings:?}"
        );
        let allowed: BTreeSet<String> = ACCENT_ROLES
            .iter()
            .map(|s| (*s).to_owned())
            .chain(bindings)
            .collect();
        for c in variants {
            let diff = changed(default, c);
            let outside: Vec<&String> = diff.difference(&allowed).collect();
            assert!(outside.is_empty(), "{}: {outside:?}", c.label());
            for path in [
                "semantic.color.accent",
                "semantic.color.focus-ring",
                "component.button.primary.background",
            ] {
                assert!(diff.contains(path), "{}: {path} unchanged", c.label());
            }
            assert_components_unchanged(default, c);
        }
    }
}

#[test]
fn composition_example_themes_keep_component_definitions() {
    let project = project();
    let examples = layer_files(&repo().join("examples/themes"));
    assert_ne!(examples, Vec::<PathBuf>::new());
    let compact = [repo().join(COMPACT)];
    let mut selections = Vec::new();
    for mode in Mode::ALL {
        selections.push(project.default_selection(mode));
        for theme in &examples {
            selections.push(selection(mode, Some(theme), &[]));
            selections.push(selection(mode, Some(theme), &compact));
        }
    }
    let compositions = compose_all(&project, &selections);

    for group in compositions.chunks(2 * examples.len() + 1) {
        let (default, variants) = group.split_first().unwrap();
        for pair in variants.chunks(2) {
            let [plain, compacted] = pair else {
                unreachable!()
            };
            assert_components_unchanged(default, plain);
            assert_components_unchanged(default, compacted);
            // The override lands on top of the theme: spacing changes, the
            // theme's colours do not.
            let diff = changed(plain, compacted);
            assert!(
                diff.contains("semantic.spacing.step-4"),
                "{}: {diff:?}",
                compacted.label()
            );
            assert!(
                diff.iter().all(|p| !p.starts_with("semantic.color.")),
                "{}: {diff:?}",
                compacted.label()
            );
            assert_eq!(compacted.layers().len(), plain.layers().len() + 1);
        }
    }
}
