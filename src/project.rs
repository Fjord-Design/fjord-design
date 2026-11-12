//! A Fjord project: the contract, usage and compositions of layers (C2–C5).
//!
//! [`Project::compose`] applies base and mode, then the theme and each
//! override, and fully validates the composition after every layer. It
//! stops after the first layer that produced errors, so a later layer
//! cannot hide an earlier invalid one. The final state is checked for
//! contrast (P6) and syntax separation (P7).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexSet;
use rayon::prelude::*;
use serde_json::{Map, Value};

use crate::assets::{FontReport, IconReport};
use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule, Severity, finish};
use crate::dtcg::graph::{Resolved, ResolvedToken};
use crate::dtcg::tree::{Expanded, Layer, LayerId, LayerKind, TokenPath, Tree, visit_tokens};
use crate::dtcg::value::TypedValue;
use crate::json::{self, display_path};
use crate::policy::contract::{self, Contract, EXTENSION, Mode};
use crate::policy::contrast::{self, ContrastPair};
use crate::policy::deprecation::Tracker;
use crate::policy::layers::{self, Later};
use crate::policy::usage::Usage;
use crate::policy::{self, Locator, roles, seeds};

const FONT_MANIFEST: &str = "assets/fonts/manifest.json";
const ICON_MANIFEST: &str = "assets/icons/manifest.json";
const LAYER_SUFFIX: &str = ".tokens.json";

/// A token project rooted at a directory holding `tokens/contract.json`.
#[derive(Clone, Debug)]
pub struct Project {
    root: PathBuf,
    contract: Contract,
    usage: Usage,
    /// P6 pairs per mode (indexed like [`Mode::ALL`]), de-duplicated.
    pairs: [Vec<ContrastPair>; 3],
}

/// What to compose: a mode, an optional theme replacing the contract's
/// default theme, and overrides applied left to right (C2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub mode: Mode,
    pub theme: Option<PathBuf>,
    pub overrides: Vec<PathBuf>,
}

impl Selection {
    /// `night`, or `night · fern` with an explicit theme.
    #[must_use]
    pub fn label(&self) -> String {
        match &self.theme {
            None => self.mode.as_str().to_owned(),
            Some(theme) => format!("{} · {}", self.mode, theme_name(theme)),
        }
    }
}

/// `fern` for `…/fern.tokens.json`.
fn theme_name(path: &Path) -> String {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    match name.strip_suffix(LAYER_SUFFIX) {
        Some(stem) => stem.to_owned(),
        None => path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
    }
}

impl Project {
    /// Open the project at `root`: read the contract (IN3) and statically
    /// check the usage file (P5).
    ///
    /// # Errors
    /// Every contract or usage violation, or the I/O error that prevented
    /// reading them.
    pub fn open(root: &Path) -> Outcome<Project> {
        let contract = Contract::read(root)?;
        let usage = Usage::read(&contract.usage, root)?;
        let declared = usage.pairs();
        let pairs = Mode::ALL.map(|mode| {
            let mut all = contrast::pairs(mode.as_str());
            all.extend(declared.iter().cloned());
            contrast::merge_pairs(all)
        });
        Ok(Project {
            root: root.to_path_buf(),
            contract,
            usage,
            pairs,
        })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    #[must_use]
    pub fn usage(&self) -> &Usage {
        &self.usage
    }

    /// The selection of `mode` with the default theme and no overrides.
    #[must_use]
    pub fn default_selection(&self, mode: Mode) -> Selection {
        Selection {
            mode,
            theme: None,
            overrides: Vec::new(),
        }
    }

    /// Every `*.tokens.json` beside the default theme, sorted, excluding it.
    ///
    /// # Errors
    /// [`Rule::Io`] when the theme directory cannot be listed.
    pub fn official_themes(&self) -> Outcome<Vec<PathBuf>> {
        let default = &self.contract.theme;
        let dir = default.parent().unwrap_or(&self.root);
        let io = |e: std::io::Error| {
            Failure::one(Diagnostic::error(
                Rule::Io,
                display_path(dir, &self.root),
                format!("cannot list themes: {e}"),
            ))
        };
        let canonical_default = default.canonicalize().ok();
        let mut themes = Vec::new();
        for entry in fs::read_dir(dir).map_err(io)? {
            let path = entry.map_err(io)?.path();
            let is_layer = path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with(LAYER_SUFFIX));
            let is_default = path == *default
                || canonical_default.is_some() && path.canonicalize().ok() == canonical_default;
            if is_layer && path.is_file() && !is_default {
                themes.push(path);
            }
        }
        themes.sort();
        Ok(themes)
    }

    /// Compose and fully validate one selection.
    ///
    /// # Errors
    /// Every error of the first layer that failed (with the warnings found
    /// so far), or the final-state contrast and syntax failures. Every
    /// diagnostic carries the selection label as context.
    pub fn compose(&self, selection: &Selection) -> Outcome<Composition> {
        let label = selection.label();
        let mut composer = Composer {
            project: self,
            layers: Vec::new(),
            locations: Vec::new(),
            tree: Tree::new(),
            tracker: Tracker::new(),
            diags: Vec::new(),
        };
        let state = composer.run(selection);
        let mut seen = HashSet::new();
        let diags: Vec<Diagnostic> = composer
            .diags
            .into_iter()
            .filter(|d| {
                d.severity == Severity::Error
                    || seen.insert((d.rule, d.location.clone(), d.message.clone()))
            })
            .map(|d| d.with_context(label.clone()))
            .collect();
        match state {
            Some((expanded, resolved)) if diags.iter().all(|d| d.severity != Severity::Error) => {
                Ok(Composition {
                    mode: selection.mode,
                    label,
                    layers: composer.layers,
                    locations: composer.locations,
                    expanded,
                    resolved,
                    warnings: diags,
                    pair_count: self.pairs[selection.mode.index()].len(),
                })
            }
            _ => Err(Failure(diags)),
        }
    }
}

/// Compose `selections` in parallel and validate each.
///
/// # Errors
/// The diagnostics of every failed composition, in selection order.
pub fn validate_tokens(project: &Project, selections: &[Selection]) -> Outcome<Vec<Composition>> {
    let results: Vec<Outcome<Composition>> =
        selections.par_iter().map(|s| project.compose(s)).collect();
    if results.iter().all(Result::is_ok) {
        return Ok(results.into_iter().filter_map(Result::ok).collect());
    }
    Err(Failure(
        results
            .into_iter()
            .filter_map(Result::err)
            .flat_map(|f| f.0)
            .collect(),
    ))
}

/// A3: base `fontAssets`/`iconAssets` name the pinned manifests, and every
/// typography role's first family has a bundled face of the role's style
/// (`$extensions.org.xerrion.fjord.fontStyle`, default `normal`) covering
/// its weight. `icons` is taken as proof that the icon set passed A4.
///
/// # Errors
/// Every [`Rule::FontAsset`]/[`Rule::IconAsset`] violation.
pub fn check_asset_bindings(
    composition: &Composition,
    fonts: &FontReport,
    _icons: &IconReport,
) -> Outcome<()> {
    let mut diags = Vec::new();
    if let (Some(base), Some(location)) =
        (composition.layers.first(), composition.locations.first())
    {
        let metadata = base
            .document
            .get("$extensions")
            .and_then(|e| e.get(EXTENSION));
        for (key, expected, rule) in [
            ("fontAssets", FONT_MANIFEST, Rule::FontAsset),
            ("iconAssets", ICON_MANIFEST, Rule::IconAsset),
        ] {
            if metadata.and_then(|m| m.get(key)).and_then(Value::as_str) != Some(expected) {
                diags.push(Diagnostic::error(
                    rule,
                    format!("{location}: #/$extensions/{EXTENSION}/{key}"),
                    format!("{key} must reference {expected}"),
                ));
            }
        }
    }
    for (path, token) in composition.tokens() {
        let TypedValue::Typography(t) = &token.value else {
            continue;
        };
        if !path.starts_with("semantic.typography.") {
            continue;
        }
        let location = composition.locate(path);
        let style = token
            .definition
            .get("$extensions")
            .and_then(|e| e.get(EXTENSION))
            .and_then(|m| m.get("fontStyle"));
        let style = match style {
            None => "normal",
            Some(Value::String(s)) => s.as_str(),
            Some(other) => {
                diags.push(Diagnostic::error(
                    Rule::FontAsset,
                    location,
                    format!("fontStyle must be a string, found {other}"),
                ));
                continue;
            }
        };
        let family = t.font_family.first().map_or("", String::as_str);
        if !fonts.supports(family, style, t.font_weight) {
            diags.push(
                Diagnostic::error(
                    Rule::FontAsset,
                    location,
                    format!(
                        "no bundled {family} face supports style {style} at weight {}",
                        t.font_weight
                    ),
                )
                .with_help("use a bundled family, style and weight range"),
            );
        }
    }
    finish((), diags)
}

/// A validated mode × theme × overrides composition.
#[derive(Clone, Debug)]
pub struct Composition {
    mode: Mode,
    label: String,
    layers: Vec<Layer>,
    /// Display path of each layer file.
    locations: Vec<String>,
    expanded: Expanded,
    resolved: Resolved,
    warnings: Vec<Diagnostic>,
    pair_count: usize,
}

impl Composition {
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// `night` or `night · fern`.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[must_use]
    pub fn get(&self, path: &str) -> Option<&ResolvedToken> {
        self.resolved.get(path)
    }

    /// Tokens in composed order.
    pub fn tokens(&self) -> impl Iterator<Item = (&TokenPath, &ResolvedToken)> {
        self.resolved.iter()
    }

    /// De-duplicated warnings, labelled with the composition.
    #[must_use]
    pub fn warnings(&self) -> &[Diagnostic] {
        &self.warnings
    }

    /// Applied layers in merge order; a token origin `LayerId` indexes it.
    #[must_use]
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// The composed document with inheritance materialized (O1 body).
    #[must_use]
    pub fn expanded_document(&self) -> &Map<String, Value> {
        self.expanded.document()
    }

    /// The file of every applied layer.
    #[must_use]
    pub fn layer_files(&self) -> Vec<PathBuf> {
        self.layers.iter().map(|l| l.file.clone()).collect()
    }

    /// Number of contrast pairs checked (P6).
    #[must_use]
    pub fn pair_count(&self) -> usize {
        self.pair_count
    }

    fn locate(&self, path: &str) -> String {
        Locator {
            files: &self.locations,
            current: self.locations.last().map_or("", String::as_str),
        }
        .token(&self.resolved, path)
    }
}

struct Composer<'p> {
    project: &'p Project,
    layers: Vec<Layer>,
    locations: Vec<String>,
    tree: Tree,
    tracker: Tracker,
    diags: Vec<Diagnostic>,
}

/// The paths a layer assigns: from the expansion right after it when
/// available, otherwise its direct token definitions.
fn assigned(layer: &Layer, expanded: Option<&Expanded>) -> IndexSet<TokenPath> {
    if let Some(e) = expanded {
        e.assigned(layer).iter().cloned().collect()
    } else {
        let mut out = IndexSet::new();
        visit_tokens(&layer.document, &mut |path, _| {
            out.insert(TokenPath::from(path));
        });
        out
    }
}

impl Composer<'_> {
    fn has_errors_since(&self, start: usize) -> bool {
        self.diags[start..]
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    fn locator(&self) -> Locator<'_> {
        Locator {
            files: &self.locations,
            current: self.locations.last().map_or("", String::as_str),
        }
    }

    /// Read layer number `index` with IN2; `None` (with diagnostics) when
    /// unusable.
    fn read(&mut self, index: usize, kind: LayerKind, file: &Path) -> Option<Layer> {
        let root = &self.project.root;
        let location = display_path(file, root);
        let value = match json::read(file, root) {
            Ok(value) => value,
            Err(failure) => {
                self.diags.extend(failure.0);
                return None;
            }
        };
        let problems = contract::check_layer_metadata(&value, &location);
        let Value::Object(document) = value else {
            self.diags.extend(problems);
            return None;
        };
        if !problems.is_empty() {
            self.diags.extend(problems);
            return None;
        }
        let Ok(id) = LayerId::try_from(index) else {
            self.diags.push(Diagnostic::error(
                Rule::Io,
                location,
                "too many layers in one composition",
            ));
            return None;
        };
        Some(Layer {
            id,
            kind,
            file: file.to_path_buf(),
            document,
        })
    }

    fn push(&mut self, layer: Layer) {
        self.locations
            .push(display_path(&layer.file, &self.project.root));
        self.tree.apply(&layer);
        self.layers.push(layer);
    }

    /// Expand and resolve `tree`, collecting failures.
    fn resolve(&mut self, tree: &Tree, location: &str) -> (Option<Expanded>, Option<Resolved>) {
        let expanded = match tree.expand(location) {
            Ok(e) => e,
            Err(f) => {
                self.diags.extend(f.0);
                return (None, None);
            }
        };
        match Resolved::from_expanded(&expanded, &self.layers, location) {
            Ok(r) => (Some(expanded), Some(r)),
            Err(f) => {
                self.diags.extend(f.0);
                (Some(expanded), None)
            }
        }
    }

    /// Full validation of one state (C5): RF6, P1–P5 and P8.
    fn validate(&mut self, resolved: &Resolved, expanded: &Expanded, seeds_from: Option<usize>) {
        let at = self.locator();
        let project = self.project;
        let mut found = roles::hierarchy(resolved, at);
        found.extend(roles::required(&project.contract.required, resolved, at));
        found.extend(roles::constraints(resolved, at));
        found.extend(roles::layout(resolved, at));
        found.extend(roles::typography(resolved, at));
        found.extend(project.usage.check(resolved, at));
        if let Some(layer) = seeds_from {
            let seeds = expanded
                .document()
                .get("$extensions")
                .and_then(|e| e.get(EXTENSION))
                .and_then(|m| m.get("designSeeds"))
                .unwrap_or(&Value::Null);
            found.extend(seeds::check(seeds, &self.locations[layer]));
        }
        self.diags.extend(found);
    }

    fn sets_seeds(layer: &Layer) -> bool {
        layer
            .document
            .get("$extensions")
            .and_then(|e| e.get(EXTENSION))
            .is_some_and(|m| m.get("designSeeds").is_some())
    }

    fn run(&mut self, selection: &Selection) -> Option<(Expanded, Resolved)> {
        let project = self.project;
        let contract = &project.contract;
        let mode = selection.mode;

        // Base and mode (C2 steps 1–2).
        let base = self.read(0, LayerKind::Base, &contract.base);
        let mode_layer = self.read(1, LayerKind::Mode(mode.as_str()), contract.mode_file(mode));
        let (base, mode_layer) = (base?, mode_layer?);
        self.push(base);
        self.push(mode_layer);
        let location = self.locations[1].clone();
        let tree = self.tree.clone();
        let (expanded, resolved) = self.resolve(&tree, &location);
        let base_paths = assigned(&self.layers[0], expanded.as_ref());
        let mode_paths = assigned(&self.layers[1], expanded.as_ref());
        self.diags.extend(layers::mode_roles(
            contract.mode_roles(),
            &base_paths,
            &self.locations[0],
            &mode_paths,
            &self.locations[1],
        ));
        let (Some(mut expanded), Some(mut resolved)) = (expanded, resolved) else {
            return None;
        };
        self.validate(&resolved, &expanded, Some(0));
        self.tracker.record(&resolved);
        for (i, paths) in [(0, &base_paths), (1, &mode_paths)] {
            let warnings =
                self.tracker
                    .check(&self.layers[i], &self.locations[i], paths, &resolved, None);
            self.diags.extend(warnings);
        }
        if self.has_errors_since(0) {
            return None;
        }

        // Theme and overrides (C2 steps 3–4).
        let theme = selection.theme.as_deref().unwrap_or(&contract.theme);
        let files = std::iter::once(theme).chain(selection.overrides.iter().map(PathBuf::as_path));
        for (n, file) in files.enumerate() {
            let start = self.diags.len();
            let location = display_path(file, &project.root);
            let kind = if n == 0 {
                LayerKind::Theme
            } else if let Ok(n) = u16::try_from(n) {
                LayerKind::Override(n)
            } else {
                self.diags.push(Diagnostic::error(
                    Rule::Io,
                    location,
                    "too many overrides in one composition",
                ));
                return None;
            };
            let layer = self.read(self.layers.len(), kind, file)?;
            let mut tree = self.tree.clone();
            tree.apply(&layer);
            let seeds = Self::sets_seeds(&layer);
            self.layers.push(layer);
            self.locations.push(location.clone());
            let (next_expanded, next_resolved) = self.resolve(&tree, &location);
            let layer = &self.layers[self.layers.len() - 1];
            let paths = assigned(layer, next_expanded.as_ref());
            let mut c4 = Later::new(&location, &resolved);
            c4.paths(&paths, &layer.document);
            if let Some(next) = &next_expanded {
                c4.token_set(next.document());
            }
            if let Some(next) = &next_resolved {
                c4.state(&paths, next);
            }
            let c4 = c4.diags;
            self.diags.extend(c4);
            let (Some(next_expanded), Some(next_resolved)) = (next_expanded, next_resolved) else {
                return None;
            };
            self.validate(
                &next_resolved,
                &next_expanded,
                seeds.then_some(self.layers.len() - 1),
            );
            let earlier = self.tracker.record(&next_resolved);
            let layer = &self.layers[self.layers.len() - 1];
            let warnings =
                self.tracker
                    .check(layer, &location, &paths, &next_resolved, Some(&earlier));
            self.diags.extend(warnings);
            if self.has_errors_since(start) {
                return None;
            }
            self.tree = tree;
            expanded = next_expanded;
            resolved = next_resolved;
        }

        // Final contrast (P6) and syntax separation (P7).
        let at = self.locator();
        let mut found = policy::check_contrast(&resolved, &project.pairs[mode.index()], at);
        found.extend(policy::check_syntax(&resolved, at));
        self.diags.extend(found);
        Some((expanded, resolved))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use serde_json::json;
    use tempfile::TempDir;

    use super::*;

    /// A tempdir copy of the repository's `tokens/`, `assets/` and `examples/`.
    struct Fixture {
        _dir: TempDir,
        root: PathBuf,
    }

    fn copy_tree(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    impl Fixture {
        fn new() -> Self {
            let dir = TempDir::new().unwrap();
            let root = dir.path().canonicalize().unwrap();
            let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
            for name in ["tokens", "assets", "examples"] {
                copy_tree(&repo.join(name), &root.join(name));
            }
            Self { _dir: dir, root }
        }

        fn project(&self) -> Project {
            Project::open(&self.root).unwrap()
        }

        /// An IN2-valid override layer assigning `token` at the dotted `path`.
        fn layer(&self, name: &str, path: &str, token: &Value) -> PathBuf {
            self.layer_with(name, path, token, "0.5.0")
        }

        fn layer_with(&self, name: &str, path: &str, token: &Value, version: &str) -> PathBuf {
            let mut node = token.clone();
            for seg in path.rsplit('.') {
                node = json!({ seg: node });
            }
            let Value::Object(mut document) = node else {
                unreachable!()
            };
            document.insert(
                "$extensions".to_owned(),
                json!({EXTENSION: {"schemaVersion": 2, "contractVersion": version, "dtcgVersion": "2025.10"}}),
            );
            let file = self.root.join(format!("{name}.tokens.json"));
            fs::write(
                &file,
                serde_json::to_vec_pretty(&Value::Object(document)).unwrap(),
            )
            .unwrap();
            file
        }
    }

    fn rules(failure: &Failure) -> Vec<Rule> {
        failure
            .0
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.rule)
            .collect()
    }

    fn color(hex: &str) -> Value {
        let n = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap();
        let c = [16, 8, 0].map(|s| f64::from((n >> s) & 0xff) / 255.0);
        json!({"colorSpace": "srgb", "components": c})
    }

    fn token(ty: &str, value: &Value) -> Value {
        json!({"$type": ty, "$value": value, "$description": "Negative case."})
    }

    #[test]
    fn c2_every_shipped_composition_is_valid() {
        let f = Fixture::new();
        let project = f.project();
        let mut themes = vec![None];
        themes.extend(project.official_themes().unwrap().into_iter().map(Some));
        assert_eq!(themes.len(), 6, "{themes:?}");
        let mut examples: Vec<PathBuf> = fs::read_dir(f.root.join("examples/themes"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        examples.sort();
        assert_eq!(examples.len(), 3);
        themes.extend(examples.into_iter().map(Some));
        let compact = f
            .root
            .join("examples/overrides/compact-spacing.tokens.json");
        let mut selections = Vec::new();
        for mode in Mode::ALL {
            for theme in &themes {
                for overrides in [vec![], vec![compact.clone()]] {
                    selections.push(Selection {
                        mode,
                        theme: theme.clone(),
                        overrides,
                    });
                }
            }
        }
        assert_eq!(selections.len(), 54);
        let compositions = match validate_tokens(&project, &selections) {
            Ok(c) => c,
            Err(failure) => panic!("{failure}"),
        };
        assert_eq!(compositions.len(), 54);
        for (c, s) in compositions.iter().zip(&selections) {
            assert_eq!(c.mode(), s.mode);
            assert_eq!(c.label(), s.label());
            assert!(c.warnings().is_empty(), "{:?}", c.warnings());
            assert_eq!(c.layers().len(), 3 + s.overrides.len());
            assert!(c.pair_count() > 0);
        }
        assert_eq!(compositions[2].label(), "night · cabin");
    }

    #[test]
    fn c6_origins_index_layers() {
        let f = Fixture::new();
        let project = f.project();
        let compact = f
            .root
            .join("examples/overrides/compact-spacing.tokens.json");
        let selection = Selection {
            mode: Mode::Day,
            theme: Some(f.root.join("tokens/themes/fern.tokens.json")),
            overrides: vec![compact.clone()],
        };
        let c = project.compose(&selection).unwrap();
        let label = |path: &str| {
            let origin = c.get(path).unwrap().origin.unwrap();
            c.layers()[usize::from(origin)].source_label()
        };
        assert_eq!(
            label("semantic.spacing.step-4"),
            "override:1:compact-spacing.tokens.json"
        );
        assert_eq!(label("semantic.color.ink"), "mode:day");
        assert_eq!(label("semantic.color.accent"), "theme:fern.tokens.json");
        assert_eq!(c.layer_files().last(), Some(&compact));
    }

    #[rstest::rstest]
    #[case::unknown_path("semantic.color.unknown", token("color", &color("#808080")), Rule::OverridePath)]
    #[case::primitive("primitive.size.target-touch", token("dimension", &json!({"value": 44, "unit": "px"})), Rule::OverrideNamespace)]
    #[case::type_change("semantic.color.ink", token("number", &json!(42)), Rule::OverrideType)]
    #[case::null_color("semantic.color.ink", token("color", &Value::Null), Rule::Value)]
    #[case::partial_shadow("semantic.shadow.shadow-sm", token("shadow", &json!({"blur": {"value": 4, "unit": "px"}})), Rule::Value)]
    #[case::component_literal("component.button.primary.background", token("color", &json!({"colorSpace": "srgb", "components": [1, 1, 1], "hex": "#ffffff"})), Rule::ComponentBinding)]
    #[case::hierarchy("semantic.spacing.step-4", token("dimension", &json!("{component.button.primary.padding-block}")), Rule::Hierarchy)]
    #[case::cycle("semantic.color.accent", token("color", &json!("{semantic.color.focus-ring}")), Rule::ReferenceCycle)]
    #[case::missing("semantic.color.accent", token("color", &json!("{semantic.color.absent}")), Rule::ReferenceMissing)]
    #[case::contrast("semantic.color.ansi-bright-black", token("color", &color("#738290")), Rule::Contrast)]
    #[case::target_touch("semantic.size.target-touch", token("dimension", &json!({"value": 43, "unit": "px"})), Rule::Layout)]
    #[case::disabled_text("semantic.color.ink-muted", token("color", &json!("{semantic.color.ink-disabled}")), Rule::Usage)]
    #[case::syntax("semantic.color.syn-type", token("color", &json!("{semantic.color.syn-keyword}")), Rule::SyntaxDistance)]
    #[case::slow_motion("semantic.duration.duration-slow", token("duration", &json!({"value": 301, "unit": "ms"})), Rule::Layout)]
    fn negative_override_fails(#[case] path: &str, #[case] value: Value, #[case] rule: Rule) {
        let f = Fixture::new();
        let file = f.layer("negative", path, &value);
        let selection = Selection {
            mode: Mode::Night,
            theme: None,
            overrides: vec![file],
        };
        let failure = f.project().compose(&selection).unwrap_err();
        assert!(
            rules(&failure).contains(&rule),
            "expected {rule:?}: {failure}"
        );
        assert!(
            failure
                .0
                .iter()
                .all(|d| d.context.as_deref() == Some("night"))
        );
    }

    #[test]
    fn in2_old_contract_layer_fails() {
        let f = Fixture::new();
        let file = f.layer_with(
            "old",
            "semantic.color.ink",
            &token("color", &color("#ffffff")),
            "0.4.0",
        );
        let selection = Selection {
            mode: Mode::Night,
            theme: None,
            overrides: vec![file],
        };
        let failure = f.project().compose(&selection).unwrap_err();
        assert_eq!(rules(&failure), vec![Rule::LayerMetadata]);
    }

    #[test]
    fn c5_invalid_layer_is_not_hidden_by_a_later_one() {
        let f = Fixture::new();
        let bad = f.layer(
            "bad",
            "semantic.size.target-touch",
            &token("dimension", &json!({"value": 43, "unit": "px"})),
        );
        let fix = f.layer(
            "fix",
            "semantic.size.target-touch",
            &token("dimension", &json!({"value": 44, "unit": "px"})),
        );
        let selection = Selection {
            mode: Mode::Night,
            theme: None,
            overrides: vec![bad, fix],
        };
        let failure = f.project().compose(&selection).unwrap_err();
        assert_eq!(rules(&failure), vec![Rule::Layout]);
        assert!(failure.0[0].location.starts_with("bad.tokens.json"));
    }

    #[test]
    fn c3_base_defining_a_mode_role_fails() {
        let f = Fixture::new();
        let path = f.root.join("tokens/base.tokens.json");
        let mut base = json::read(&path, &f.root).unwrap();
        base["semantic"]["color"] = json!({"ink": token("color", &color("#ffffff"))});
        fs::write(&path, serde_json::to_vec(&base).unwrap()).unwrap();
        let failure = f
            .project()
            .compose(&f.project().default_selection(Mode::Day))
            .unwrap_err();
        assert!(rules(&failure).contains(&Rule::ModeRoles), "{failure}");
    }

    #[test]
    fn c4_removing_a_token_fails() {
        let f = Fixture::new();
        let file = f.layer(
            "remove",
            "semantic.spacing",
            &token("dimension", &json!({"value": 1, "unit": "px"})),
        );
        let selection = Selection {
            mode: Mode::Night,
            theme: None,
            overrides: vec![file],
        };
        let failure = f.project().compose(&selection).unwrap_err();
        assert!(
            rules(&failure).contains(&Rule::OverrideTokenSet),
            "{failure}"
        );
    }

    #[test]
    fn p9_deprecated_use_and_reassignment_warn_once() {
        let f = Fixture::new();
        let mut deprecated = token("color", &color("#123456"));
        deprecated["$deprecated"] = json!("Use semantic.color.ink.");
        let theme = f.layer("deprecating", "semantic.color.ink-muted", &deprecated);
        let using = f.layer(
            "using",
            "semantic.color.ink-subtle",
            &token("color", &json!("{semantic.color.ink-muted}")),
        );
        let reassign = f.layer(
            "reassign",
            "semantic.color.ink-muted",
            &token("color", &json!("{semantic.color.ink}")),
        );
        let selection = Selection {
            mode: Mode::Night,
            theme: Some(theme),
            overrides: vec![using, reassign],
        };
        // Contrast of the deliberately dark colour fails; warnings still come first.
        let diags = match f.project().compose(&selection) {
            Ok(c) => c.warnings().to_vec(),
            Err(failure) => failure.0,
        };
        let warnings: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.rule == Rule::Deprecated)
            .collect();
        let locations: Vec<&str> = warnings.iter().map(|d| d.location.as_str()).collect();
        assert!(
            locations.contains(&"using.tokens.json: semantic.color.ink-subtle"),
            "{locations:?}"
        );
        assert!(
            locations.contains(&"reassign.tokens.json: semantic.color.ink-muted"),
            "{locations:?}"
        );
        assert!(warnings.iter().all(|d| d.severity == Severity::Warning));
    }

    #[test]
    fn official_themes_exclude_the_default() {
        let f = Fixture::new();
        let names: Vec<String> = f
            .project()
            .official_themes()
            .unwrap()
            .iter()
            .map(|p| theme_name(p))
            .collect();
        assert_eq!(names, ["cabin", "fern", "hearth", "mountain", "water"]);
    }

    #[test]
    fn a3_asset_bindings_pass_and_detect_unsupported_weight() {
        let f = Fixture::new();
        let project = f.project();
        let fonts = crate::assets::check_fonts(&f.root).unwrap();
        let icons = crate::assets::check_icons(&f.root).unwrap();
        let c = project
            .compose(&project.default_selection(Mode::Night))
            .unwrap();
        check_asset_bindings(&c, &fonts, &icons).unwrap();

        let path = f.root.join("tokens/base.tokens.json");
        let mut base = json::read(&path, &f.root).unwrap();
        base["semantic"]["typography"]["code"]["$value"]["fontWeight"] = json!(800);
        base["$extensions"][EXTENSION]["iconAssets"] = json!("assets/icons/other.json");
        fs::write(&path, serde_json::to_vec(&base).unwrap()).unwrap();
        let c = project
            .compose(&project.default_selection(Mode::Night))
            .unwrap();
        let failure = check_asset_bindings(&c, &fonts, &icons).unwrap_err();
        assert_eq!(rules(&failure), vec![Rule::IconAsset, Rule::FontAsset]);
    }
}
