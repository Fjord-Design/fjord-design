//! Project contract: modes, version constants, layer metadata (IN2) and
//! `tokens/contract.json` with its code-protected required roles (IN3).

use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use indexmap::IndexMap;
use serde_json::{Map, Value, json};

use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule};
use crate::dtcg::tree::TokenPath;
use crate::dtcg::value::TokenType;
use crate::json::{self, display_path};

pub const SCHEMA_VERSION: u64 = 2;
pub const CONTRACT_VERSION: &str = "0.5.0";
pub const DTCG_VERSION: &str = "2025.10";

/// The vendor extension key that holds Fjord metadata.
pub const EXTENSION: &str = "org.xerrion.fjord";

const CONTRACT_FILE: &str = "tokens/contract.json";
const TOKENS_DIR: &str = "tokens";

/// A colour mode: one mode layer is applied after base (C2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Night,
    Day,
    Aurora,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Night, Mode::Day, Mode::Aurora];

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Night => "night",
            Mode::Day => "day",
            Mode::Aurora => "aurora",
        }
    }

    /// Position in [`Mode::ALL`].
    #[must_use]
    pub(crate) fn index(self) -> usize {
        match self {
            Mode::Night => 0,
            Mode::Day => 1,
            Mode::Aurora => 2,
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Mode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Mode::ALL
            .into_iter()
            .find(|m| m.as_str() == s)
            .ok_or_else(|| format!("unknown mode \"{s}\"; choose night, day or aurora"))
    }
}

/// `0.5.<n>` with `n` free of leading zeros.
fn is_contract_version(v: &str) -> bool {
    v.strip_prefix("0.5.").is_some_and(|n| {
        !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) && (n == "0" || !n.starts_with('0'))
    })
}

/// IN2: a layer declares `$extensions.org.xerrion.fjord` with integer
/// `schemaVersion: 2`, `contractVersion` `0.5.<n>` and `dtcgVersion: "2025.10"`.
#[must_use]
pub fn check_layer_metadata(document: &Value, location: &str) -> Vec<Diagnostic> {
    let at = format!("{location}: #/$extensions/{EXTENSION}");
    let fail = |message: String| {
        Diagnostic::error(Rule::LayerMetadata, &at, message).with_help(format!(
            "declare {{\"schemaVersion\": {SCHEMA_VERSION}, \"contractVersion\": \"{CONTRACT_VERSION}\", \"dtcgVersion\": \"{DTCG_VERSION}\"}} in $extensions.{EXTENSION}"
        ))
    };
    let Some(metadata) = document
        .get("$extensions")
        .and_then(|e| e.get(EXTENSION))
        .and_then(Value::as_object)
    else {
        return vec![fail(
            "the layer declares no Fjord metadata object".to_owned(),
        )];
    };
    let mut out = Vec::new();
    if metadata.get("schemaVersion").and_then(Value::as_u64) != Some(SCHEMA_VERSION) {
        out.push(fail(format!(
            "schemaVersion is {}, expected the integer {SCHEMA_VERSION}",
            shown(metadata.get("schemaVersion"))
        )));
    }
    if !metadata
        .get("contractVersion")
        .and_then(Value::as_str)
        .is_some_and(is_contract_version)
    {
        out.push(fail(format!(
            "contractVersion is {}, expected a 0.5.x contract",
            shown(metadata.get("contractVersion"))
        )));
    }
    if metadata.get("dtcgVersion").and_then(Value::as_str) != Some(DTCG_VERSION) {
        out.push(fail(format!(
            "dtcgVersion is {}, expected \"{DTCG_VERSION}\"",
            shown(metadata.get("dtcgVersion"))
        )));
    }
    out
}

fn shown(v: Option<&Value>) -> String {
    v.map_or_else(|| "missing".to_owned(), Value::to_string)
}

/// The 71 named mode colour roles under `semantic.color.`; the 16 ANSI
/// roles are added by [`baseline_roles`].
const COLOR_ROLES: &[&str] = &[
    "bg-sunken",
    "bg-base",
    "bg-surface",
    "bg-raised",
    "bg-hover",
    "bg-pressed",
    "bg-selected",
    "bg-inverse",
    "line",
    "line-strong",
    "ink",
    "ink-muted",
    "ink-subtle",
    "ink-disabled",
    "ink-inverse",
    "accent",
    "accent-hover",
    "accent-pressed",
    "accent-soft",
    "accent-ink",
    "on-accent",
    "focus-ring",
    "selection",
    "frost",
    "aurora",
    "lilac",
    "sage",
    "fern",
    "latte",
    "foam",
    "rose",
    "info",
    "success",
    "warning",
    "danger",
    "hearth",
    "hearth-hover",
    "hearth-soft",
    "lilac-soft",
    "info-soft",
    "success-soft",
    "warning-soft",
    "danger-soft",
    "on-danger",
    "syn-comment",
    "syn-keyword",
    "syn-string",
    "syn-function",
    "syn-type",
    "syn-variable",
    "syn-property",
    "syn-number",
    "syn-operator",
    "syn-punctuation",
    "syn-regex",
    "git-added",
    "git-modified",
    "git-removed",
    "git-ignored",
    "diff-add-bg",
    "diff-del-bg",
    "search-match",
    "search-match-current",
    "chart-1",
    "chart-2",
    "chart-3",
    "chart-4",
    "chart-5",
    "chart-6",
    "chart-grid",
    "scrim",
];

const ANSI_HUES: &[&str] = &[
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
];

/// Groups of baseline roles: `(prefix, names, type)`.
const BASELINE: &[(&str, &[&str], TokenType)] = &[
    (
        "semantic.shadow.",
        &[
            "shadow-sm",
            "shadow-raised",
            "shadow-overlay",
            "shadow-pressed",
        ],
        TokenType::Shadow,
    ),
    (
        "semantic.typography.",
        &[
            "display",
            "title-1",
            "title-2",
            "title-3",
            "title-4",
            "lead",
            "body",
            "body-strong",
            "label",
            "small",
            "caption",
            "overline",
            "code",
            "code-small",
            "code-italic",
        ],
        TokenType::Typography,
    ),
    (
        "semantic.font.",
        &["sans", "display", "mono"],
        TokenType::FontFamily,
    ),
    (
        "semantic.font-size.",
        &[
            "text-2xs",
            "text-xs",
            "text-sm",
            "text-md",
            "text-base",
            "text-lg",
            "text-xl",
            "text-2xl",
            "text-3xl",
        ],
        TokenType::Dimension,
    ),
    (
        "semantic.spacing.",
        &[
            "step-1", "step-2", "step-3", "step-4", "step-5", "step-6", "step-8", "step-10",
            "step-12", "step-16",
        ],
        TokenType::Dimension,
    ),
    (
        "semantic.radius.",
        &["none", "window", "xs", "panel", "control", "full"],
        TokenType::Dimension,
    ),
    (
        "semantic.border-width.",
        &["border-hairline", "border-focus", "border-active"],
        TokenType::Dimension,
    ),
    (
        "semantic.size.",
        &[
            "control-compact",
            "control-comfortable",
            "target-minimum",
            "target-touch",
            "icon-control",
            "icon-status",
            "icon-medium",
            "icon-default",
            "icon-empty",
        ],
        TokenType::Dimension,
    ),
    (
        "semantic.breakpoint.",
        &["breakpoint-sm", "breakpoint-md", "breakpoint-lg"],
        TokenType::Dimension,
    ),
    (
        "semantic.layout.",
        &[
            "content-max",
            "focus-offset",
            "page-gutter",
            "page-gutter-wide",
        ],
        TokenType::Dimension,
    ),
    (
        "semantic.density.",
        &[
            "gap-comfortable",
            "gap-compact",
            "padding-comfortable",
            "padding-compact",
        ],
        TokenType::Dimension,
    ),
    (
        "semantic.duration.",
        &[
            "duration-none",
            "duration-fast",
            "duration-base",
            "duration-slow",
        ],
        TokenType::Duration,
    ),
    (
        "semantic.easing.",
        &["easing-standard", "easing-exit"],
        TokenType::CubicBezier,
    ),
    (
        "semantic.z-index.",
        &[
            "z-base",
            "z-sticky",
            "z-popover",
            "z-modal",
            "z-toast",
            "z-tooltip",
        ],
        TokenType::Number,
    ),
    (
        "component.button.primary.",
        &[
            "background",
            "foreground",
            "hover-background",
            "pressed-background",
            "disabled-foreground",
            "focus-color",
        ],
        TokenType::Color,
    ),
    (
        "component.button.primary.",
        &[
            "focus-width",
            "radius",
            "padding-inline",
            "padding-block",
            "font-size",
        ],
        TokenType::Dimension,
    ),
    (
        "component.button.primary.",
        &["font-family"],
        TokenType::FontFamily,
    ),
];

/// The code-protected baseline of required roles (IN3), in a stable order.
#[must_use]
pub fn baseline_roles() -> Vec<(String, TokenType)> {
    let mut out: Vec<(String, TokenType)> = COLOR_ROLES
        .iter()
        .map(|name| (format!("semantic.color.{name}"), TokenType::Color))
        .collect();
    for prefix in ["ansi-", "ansi-bright-"] {
        out.extend(
            ANSI_HUES
                .iter()
                .map(|hue| (format!("semantic.color.{prefix}{hue}"), TokenType::Color)),
        );
    }
    for (prefix, names, ty) in BASELINE {
        out.extend(names.iter().map(|name| (format!("{prefix}{name}"), *ty)));
    }
    out
}

/// Mode roles (C3): required paths under `semantic.color.` and `semantic.shadow.`.
#[must_use]
pub fn is_mode_role(path: &str) -> bool {
    path.starts_with("semantic.color.") || path.starts_with("semantic.shadow.")
}

/// The parsed `tokens/contract.json` (IN3). Source paths are joined to the
/// project root.
#[derive(Clone, Debug)]
pub struct Contract {
    pub base: PathBuf,
    /// Mode layer files, indexed like [`Mode::ALL`].
    pub modes: [PathBuf; 3],
    pub theme: PathBuf,
    pub usage: PathBuf,
    pub required: IndexMap<TokenPath, TokenType>,
}

impl Contract {
    /// Read and check `tokens/contract.json` under `root` (IN3).
    ///
    /// # Errors
    /// [`Rule::Io`]/[`Rule::Json`] when unreadable, and every independent
    /// [`Rule::Contract`] violation.
    pub fn read(root: &Path) -> Outcome<Contract> {
        let value = json::read(&root.join(CONTRACT_FILE), root)?;
        let location = display_path(&root.join(CONTRACT_FILE), root);
        let Value::Object(c) = value else {
            return Err(Failure::one(Diagnostic::error(
                Rule::Contract,
                location,
                "the contract must be a JSON object",
            )));
        };
        let mut check = Checker {
            root,
            tokens: root.join(TOKENS_DIR),
            location,
            diags: Vec::new(),
        };
        check.versions(&c);
        let base = check.source(&c, "base");
        let theme = check.source(&c, "theme");
        let usage = check.source(&c, "usage");
        let modes = check.modes(&c);
        check.exact(
            &c,
            "mergeOrder",
            &json!(["base", "mode", "theme", "overrides"]),
        );
        check.exact(&c, "overrideNamespaces", &json!(["semantic", "component"]));
        let required = check.required(&c);
        match (base, theme, usage, modes, required) {
            (Some(base), Some(theme), Some(usage), Some(modes), Some(required))
                if check.diags.is_empty() =>
            {
                Ok(Contract {
                    base,
                    modes,
                    theme,
                    usage,
                    required,
                })
            }
            _ => Err(Failure(check.diags)),
        }
    }

    #[must_use]
    pub fn mode_file(&self, mode: Mode) -> &Path {
        &self.modes[mode.index()]
    }

    /// Mode roles (C3), in contract order.
    pub fn mode_roles(&self) -> impl Iterator<Item = &TokenPath> {
        self.required.keys().filter(|p| is_mode_role(p))
    }
}

struct Checker<'a> {
    root: &'a Path,
    tokens: PathBuf,
    location: String,
    diags: Vec<Diagnostic>,
}

impl Checker<'_> {
    fn fail(&mut self, key: &str, message: impl Into<String>) {
        self.diags.push(Diagnostic::error(
            Rule::Contract,
            format!("{}: #/{key}", self.location),
            message,
        ));
    }

    fn versions(&mut self, c: &Map<String, Value>) {
        if c.get("schemaVersion").and_then(Value::as_u64) != Some(SCHEMA_VERSION) {
            self.fail(
                "schemaVersion",
                format!("schemaVersion must be the integer {SCHEMA_VERSION}"),
            );
        }
        if !c
            .get("version")
            .and_then(Value::as_str)
            .is_some_and(is_contract_version)
        {
            self.fail("version", "version must be a 0.5.x contract version");
        }
        if c.get("dtcgVersion").and_then(Value::as_str) != Some(DTCG_VERSION) {
            self.fail(
                "dtcgVersion",
                format!("dtcgVersion must be \"{DTCG_VERSION}\""),
            );
        }
    }

    /// A string path that resolves inside `tokens/`.
    fn path(&mut self, key: &str, value: Option<&Value>) -> Option<PathBuf> {
        let Some(name) = value.and_then(Value::as_str) else {
            self.fail(key, "must be a path string relative to tokens/");
            return None;
        };
        let mut depth = 0usize;
        let mut inside = !name.is_empty();
        for part in Path::new(name).components() {
            match part {
                Component::Normal(_) => depth += 1,
                Component::CurDir => {}
                Component::ParentDir if depth > 0 => depth -= 1,
                _ => inside = false,
            }
        }
        let path = self.tokens.join(name);
        if inside && depth > 0 {
            // A symlink may still escape; compare canonical paths when it exists.
            if let (Ok(real), Ok(dir)) = (path.canonicalize(), self.tokens.canonicalize()) {
                inside = real.starts_with(dir);
            }
        } else {
            inside = false;
        }
        if inside {
            Some(path)
        } else {
            self.fail(
                key,
                format!(
                    "\"{name}\" resolves outside {}",
                    display_path(&self.tokens, self.root)
                ),
            );
            None
        }
    }

    fn source(&mut self, c: &Map<String, Value>, key: &str) -> Option<PathBuf> {
        self.path(key, c.get(key))
    }

    fn modes(&mut self, c: &Map<String, Value>) -> Option<[PathBuf; 3]> {
        let Some(modes) = c.get("modes").and_then(Value::as_object) else {
            self.fail("modes", "modes must map night, day and aurora to files");
            return None;
        };
        if modes.len() != Mode::ALL.len()
            || Mode::ALL.iter().any(|m| !modes.contains_key(m.as_str()))
        {
            self.fail("modes", "modes must declare exactly night, day and aurora");
        }
        let [night, day, aurora] = Mode::ALL.map(|m| {
            let key = format!("modes/{}", m.as_str());
            modes.get(m.as_str()).and_then(|v| self.path(&key, Some(v)))
        });
        Some([night?, day?, aurora?])
    }

    fn exact(&mut self, c: &Map<String, Value>, key: &str, expected: &Value) {
        if c.get(key) != Some(expected) {
            self.fail(key, format!("{key} must be {expected}"));
        }
    }

    fn required(&mut self, c: &Map<String, Value>) -> Option<IndexMap<TokenPath, TokenType>> {
        let Some(map) = c.get("required").and_then(Value::as_object) else {
            self.fail("required", "required must map token paths to types");
            return None;
        };
        let mut out = IndexMap::with_capacity(map.len());
        let mut ok = true;
        for (path, ty) in map {
            if let Some(Ok(ty)) = ty.as_str().map(str::parse::<TokenType>) {
                out.insert(TokenPath::from(path.as_str()), ty);
            } else {
                ok = false;
                self.fail(
                    &format!("required/{path}"),
                    format!("{ty} is not a supported token type"),
                );
            }
        }
        for (path, ty) in baseline_roles() {
            match map.get(&path).and_then(Value::as_str) {
                Some(found) if found == ty.as_str() => {}
                found => {
                    ok = false;
                    self.fail(
                        &format!("required/{path}"),
                        format!(
                            "baseline role {path} must be required with type {ty}, found {}",
                            found.unwrap_or("nothing")
                        ),
                    );
                }
            }
        }
        ok.then_some(out)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn rules(d: &[Diagnostic]) -> Vec<Rule> {
        d.iter().map(|d| d.rule).collect()
    }

    fn layer(meta: &Value) -> Value {
        json!({"$extensions": {EXTENSION: meta}})
    }

    #[test]
    fn mode_round_trips() {
        for m in Mode::ALL {
            assert_eq!(m.as_str().parse::<Mode>().unwrap(), m);
            assert_eq!(Mode::ALL[m.index()], m);
        }
        assert!("dusk".parse::<Mode>().is_err());
    }

    #[test]
    fn in2_valid_metadata_passes() {
        let doc = layer(
            &json!({"schemaVersion": 2, "contractVersion": "0.5.12", "dtcgVersion": "2025.10"}),
        );
        assert_eq!(check_layer_metadata(&doc, "f"), Vec::new());
    }

    #[rstest::rstest]
    #[case(json!({"schemaVersion": 2.0, "contractVersion": "0.5.0", "dtcgVersion": "2025.10"}))]
    #[case(json!({"schemaVersion": "2", "contractVersion": "0.5.0", "dtcgVersion": "2025.10"}))]
    #[case(json!({"schemaVersion": 2, "contractVersion": "0.4.0", "dtcgVersion": "2025.10"}))]
    #[case(json!({"schemaVersion": 2, "contractVersion": "0.5.01", "dtcgVersion": "2025.10"}))]
    #[case(json!({"schemaVersion": 2, "contractVersion": "0.5.", "dtcgVersion": "2025.10"}))]
    #[case(json!({"schemaVersion": 2, "contractVersion": "0.5.0", "dtcgVersion": "2025.1"}))]
    #[case(json!({"schemaVersion": 2, "contractVersion": "0.5.0"}))]
    fn in2_bad_metadata_fails(#[case] meta: Value) {
        assert_eq!(
            rules(&check_layer_metadata(&layer(&meta), "f")),
            vec![Rule::LayerMetadata]
        );
    }

    #[test]
    fn in2_missing_extension_fails() {
        assert_eq!(
            rules(&check_layer_metadata(&json!({"semantic": {}}), "f")),
            vec![Rule::LayerMetadata]
        );
        assert_eq!(
            rules(&check_layer_metadata(&json!([]), "f")),
            vec![Rule::LayerMetadata]
        );
    }

    #[test]
    fn in3_baseline_has_every_documented_role() {
        let roles = baseline_roles();
        let colors = roles.iter().filter(|(_, t)| *t == TokenType::Color).count();
        // 71 named + 16 ANSI semantic colours + 6 button colours.
        assert_eq!(colors, 71 + 16 + 6);
        let modes = roles.iter().filter(|(p, _)| is_mode_role(p)).count();
        assert_eq!(modes, 87 + 4);
        let mut paths: Vec<_> = roles.iter().map(|(p, _)| p).collect();
        paths.sort();
        paths.dedup();
        assert_eq!(paths.len(), roles.len());
    }

    #[test]
    fn in3_repository_contract_passes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let contract = Contract::read(root).unwrap();
        assert!(
            contract
                .mode_file(Mode::Day)
                .ends_with("modes/day.tokens.json")
        );
        assert_eq!(contract.mode_roles().count(), 91);
    }
}
