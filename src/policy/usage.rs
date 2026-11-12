//! Semantic usage (P5): roles, interaction states, their contrast pairs and
//! the disabled-text restriction.
//!
//! [`Usage::parse`] performs the static checks once per project;
//! [`Usage::check`] checks the bindings against each composed state.

use std::collections::HashSet;
use std::path::Path;

use serde_json::{Map, Value};

use super::Locator;
use super::contrast::ContrastPair;
use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule, finish};
use crate::dtcg::graph::Resolved;
use crate::dtcg::value::TokenType;
use crate::json::{self, display_path};

const DISABLED_TOKEN: &str = "semantic.color.ink-disabled";
const DISABLED_ROLE: &str = "text-disabled";
const DISABLED_STATE: &str = "disabled";
const TEXT_MINIMUM: f64 = 4.5;
const NON_TEXT_MINIMUM: f64 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleKind {
    Fill,
    Text,
    Disabled,
    Decorative,
    Border,
    Focus,
    Overlay,
}

impl RoleKind {
    const ALL: [RoleKind; 7] = [
        RoleKind::Fill,
        RoleKind::Text,
        RoleKind::Disabled,
        RoleKind::Decorative,
        RoleKind::Border,
        RoleKind::Focus,
        RoleKind::Overlay,
    ];

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            RoleKind::Fill => "fill",
            RoleKind::Text => "text",
            RoleKind::Disabled => "disabled",
            RoleKind::Decorative => "decorative",
            RoleKind::Border => "border",
            RoleKind::Focus => "focus",
            RoleKind::Overlay => "overlay",
        }
    }

    /// The minimum contrast floor of the kind; `None` means exempt (`null`).
    fn floor(self) -> Option<f64> {
        match self {
            RoleKind::Text => Some(TEXT_MINIMUM),
            RoleKind::Border | RoleKind::Focus => Some(NON_TEXT_MINIMUM),
            RoleKind::Fill | RoleKind::Disabled | RoleKind::Decorative | RoleKind::Overlay => None,
        }
    }
}

/// The baseline roles and kinds (code-protected).
const BASELINE_ROLES: &[(&str, RoleKind)] = &[
    ("canvas", RoleKind::Fill),
    ("surface", RoleKind::Fill),
    ("surface-floating", RoleKind::Fill),
    ("surface-sunken", RoleKind::Fill),
    ("surface-hover", RoleKind::Fill),
    ("surface-pressed", RoleKind::Fill),
    ("surface-selected", RoleKind::Fill),
    ("accent-fill", RoleKind::Fill),
    ("selected-fill", RoleKind::Fill),
    ("text-selection-fill", RoleKind::Fill),
    ("overlay-surface", RoleKind::Fill),
    ("info-fill", RoleKind::Fill),
    ("success-fill", RoleKind::Fill),
    ("warning-fill", RoleKind::Fill),
    ("danger-fill", RoleKind::Fill),
    ("text", RoleKind::Text),
    ("text-secondary", RoleKind::Text),
    ("text-help", RoleKind::Text),
    ("text-placeholder", RoleKind::Text),
    ("link", RoleKind::Text),
    ("link-hover", RoleKind::Text),
    ("accent-label", RoleKind::Text),
    ("selected-label", RoleKind::Text),
    ("text-selection-label", RoleKind::Text),
    ("overlay-text", RoleKind::Text),
    ("info", RoleKind::Text),
    ("success", RoleKind::Text),
    ("warning", RoleKind::Text),
    ("danger", RoleKind::Text),
    ("destructive-label", RoleKind::Text),
    ("text-disabled", RoleKind::Disabled),
    ("border-divider", RoleKind::Decorative),
    ("border-control", RoleKind::Border),
    ("focus", RoleKind::Focus),
    ("overlay-scrim", RoleKind::Overlay),
];

const STATES: &[&str] = &[
    "default",
    "hover",
    "pressed",
    "focus-visible",
    "selected",
    "disabled",
    "invalid",
    "loading",
];

#[derive(Clone, Debug, PartialEq)]
pub struct Role {
    pub name: String,
    pub token: String,
    pub kind: RoleKind,
    pub backgrounds: Vec<String>,
    pub minimum: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub name: String,
    pub foreground: String,
    pub background: String,
    pub minimum: Option<f64>,
}

/// A statically valid `tokens/usage.json`.
#[derive(Clone, Debug)]
pub struct Usage {
    location: String,
    roles: Vec<Role>,
    states: Vec<State>,
}

fn non_blank(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str).filter(|s| !s.trim().is_empty())
}

impl Usage {
    /// Read and statically check the usage file.
    ///
    /// # Errors
    /// [`Rule::Io`]/[`Rule::Json`] when unreadable; every static
    /// [`Rule::Usage`] violation.
    pub fn read(path: &Path, root: &Path) -> Outcome<Usage> {
        let value = json::read(path, root)?;
        Self::parse(&value, &display_path(path, root))
    }

    /// Statically check a parsed usage document (P5).
    ///
    /// # Errors
    /// Every independent [`Rule::Usage`] violation.
    pub fn parse(value: &Value, location: &str) -> Outcome<Usage> {
        let mut p = Parser {
            location,
            diags: Vec::new(),
        };
        let usage = p.document(value);
        match usage {
            Some(usage) => finish(usage, p.diags),
            None => Err(Failure(p.diags)),
        }
    }

    #[must_use]
    pub fn roles(&self) -> &[Role] {
        &self.roles
    }

    #[must_use]
    pub fn states(&self) -> &[State] {
        &self.states
    }

    /// The declared contrast pairs (P6 source U), in file order.
    #[must_use]
    pub fn pairs(&self) -> Vec<ContrastPair> {
        let roles = self.roles.iter().filter_map(|r| {
            r.minimum.map(|min| {
                r.backgrounds.iter().map(move |bg| ContrastPair {
                    fg: r.token.clone(),
                    bg: bg.clone(),
                    min,
                    basis: "U role",
                })
            })
        });
        let states = self.states.iter().filter_map(|s| {
            s.minimum.map(|min| ContrastPair {
                fg: s.foreground.clone(),
                bg: s.background.clone(),
                min,
                basis: "U state",
            })
        });
        roles.flatten().chain(states).collect()
    }

    /// Dynamic P5 checks against one composed state: referenced paths are
    /// colour tokens under `semantic.color.`/`component.`, fill roles are
    /// opaque, and only the disabled role and state reach `ink-disabled`.
    #[must_use]
    pub fn check(&self, resolved: &Resolved, at: Locator<'_>) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let referenced = self
            .roles
            .iter()
            .flat_map(|r| std::iter::once(&r.token).chain(&r.backgrounds))
            .chain(
                self.states
                    .iter()
                    .flat_map(|s| [&s.foreground, &s.background]),
            );
        let mut missing = HashSet::new();
        for path in referenced {
            if !seen.insert(path.as_str()) {
                continue;
            }
            let namespace = path.starts_with("semantic.color.") || path.starts_with("component.");
            let is_color = resolved.get(path).is_some_and(|t| t.ty == TokenType::Color);
            if !namespace || !is_color {
                missing.insert(path.as_str());
                out.push(
                    Diagnostic::error(
                        Rule::Usage,
                        &self.location,
                        format!("{path} must be an existing colour token under semantic.color. or component."),
                    )
                    .with_help("bind usage roles and states to semantic or component colour roles"),
                );
            }
        }
        for role in self.roles.iter().filter(|r| r.kind == RoleKind::Fill) {
            if missing.contains(role.token.as_str()) {
                continue;
            }
            if let Some(c) = super::color(resolved, &role.token)
                && c.alpha < 1.0
            {
                out.push(
                    Diagnostic::error(
                        Rule::Usage,
                        at.token(resolved, &role.token),
                        format!(
                            "fill role {} resolves {} to alpha {}; content surfaces must be opaque",
                            role.name, role.token, c.alpha
                        ),
                    )
                    .with_help("bind fill roles to opaque colours"),
                );
            }
        }
        let active = self
            .roles
            .iter()
            .filter(|r| r.name != DISABLED_ROLE)
            .map(|r| (format!("role {}", r.name), &r.token))
            .chain(
                self.states
                    .iter()
                    .filter(|s| s.name != DISABLED_STATE)
                    .map(|s| (format!("state {}", s.name), &s.foreground)),
            );
        for (what, token) in active {
            if let Some(chain) = reaches_disabled(resolved, token) {
                out.push(
                    Diagnostic::error(
                        Rule::Usage,
                        at.token(resolved, token),
                        format!(
                            "{what} reaches {DISABLED_TOKEN} through {}; disabled text is only for unavailable controls",
                            chain.join(" → ")
                        ),
                    )
                    .with_help("bind active text, help and states to a readable colour"),
                );
            }
        }
        out
    }
}

/// The reference chain from `start` to `ink-disabled`, if any.
fn reaches_disabled(resolved: &Resolved, start: &str) -> Option<Vec<String>> {
    let mut parent: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut seen = HashSet::from([start.to_owned()]);
    let mut stack = vec![start.to_owned()];
    while let Some(path) = stack.pop() {
        if path == DISABLED_TOKEN {
            let mut chain = vec![path.clone()];
            let mut at = path;
            while let Some(p) = parent.get(&at) {
                chain.push(p.clone());
                at = p.clone();
            }
            chain.reverse();
            return Some(chain);
        }
        let Some(token) = resolved.get(&path) else {
            continue;
        };
        for next in &token.references {
            if seen.insert(next.to_string()) {
                parent.insert(next.to_string(), path.clone());
                stack.push(next.to_string());
            }
        }
    }
    None
}

struct Parser<'a> {
    location: &'a str,
    diags: Vec<Diagnostic>,
}

impl Parser<'_> {
    fn fail(&mut self, at: &str, message: impl Into<String>) {
        self.diags.push(Diagnostic::error(
            Rule::Usage,
            format!("{}: #/{at}", self.location),
            message,
        ));
    }

    fn document(&mut self, value: &Value) -> Option<Usage> {
        let Some(doc) = value.as_object() else {
            self.fail("", "usage must be a JSON object");
            return None;
        };
        if doc.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
            self.fail("schemaVersion", "schemaVersion must be the integer 1");
        }
        let roles = if let Some(roles) = doc.get("roles").and_then(Value::as_object) {
            self.roles(roles)
        } else {
            self.fail("roles", "roles must be an object");
            Vec::new()
        };
        let states = if let Some(states) = doc.get("states").and_then(Value::as_object) {
            self.states(states)
        } else {
            self.fail("states", "states must be an object");
            Vec::new()
        };
        Some(Usage {
            location: self.location.to_owned(),
            roles,
            states,
        })
    }

    fn roles(&mut self, roles: &Map<String, Value>) -> Vec<Role> {
        for (name, _) in BASELINE_ROLES {
            if !roles.contains_key(*name) {
                self.fail(
                    &format!("roles/{name}"),
                    format!("missing baseline role {name}"),
                );
            }
        }
        roles
            .iter()
            .filter_map(|(name, role)| self.role(name, role))
            .collect()
    }

    fn role(&mut self, name: &str, value: &Value) -> Option<Role> {
        let at = format!("roles/{name}");
        let Some(expected) = BASELINE_ROLES
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, k)| *k)
        else {
            self.fail(&at, format!("{name} is not a baseline role"));
            return None;
        };
        let Some(role) = value.as_object() else {
            self.fail(&at, "a role must be an object");
            return None;
        };
        let errors = self.diags.len();
        let kind = role
            .get("kind")
            .and_then(Value::as_str)
            .and_then(|k| RoleKind::ALL.into_iter().find(|r| r.as_str() == k));
        if kind != Some(expected) {
            self.fail(
                &format!("{at}/kind"),
                format!("{name} must keep kind \"{}\"", expected.as_str()),
            );
        }
        let token = role.get("token").and_then(Value::as_str);
        match token {
            None => self.fail(&format!("{at}/token"), "token must be a token path"),
            Some(t) if (name == DISABLED_ROLE) != (t == DISABLED_TOKEN) => self.fail(
                &format!("{at}/token"),
                format!("only {DISABLED_ROLE} binds {DISABLED_TOKEN}"),
            ),
            Some(_) => {}
        }
        if non_blank(role.get("description")).is_none() {
            self.fail(
                &format!("{at}/description"),
                "description must be non-blank text",
            );
        }
        let backgrounds = self.backgrounds(&at, role.get("backgrounds"));
        let minimum = self.minimum(&at, role.get("minimumContrast"), expected.floor());
        if expected.floor().is_some() && backgrounds.is_empty() {
            self.fail(
                &format!("{at}/backgrounds"),
                format!("{} roles need at least one background", expected.as_str()),
            );
        }
        (self.diags.len() == errors).then(|| Role {
            name: name.to_owned(),
            token: token.unwrap_or_default().to_owned(),
            kind: expected,
            backgrounds,
            minimum,
        })
    }

    fn backgrounds(&mut self, at: &str, value: Option<&Value>) -> Vec<String> {
        let at = format!("{at}/backgrounds");
        let Some(items) = value.and_then(Value::as_array) else {
            self.fail(&at, "backgrounds must be an array of token paths");
            return Vec::new();
        };
        let mut out: Vec<String> = Vec::with_capacity(items.len());
        for item in items {
            match item.as_str() {
                Some(path) if out.iter().any(|p| p == path) => {
                    self.fail(&at, format!("background {path} is listed twice"));
                }
                Some(path) => out.push(path.to_owned()),
                None => self.fail(&at, format!("{item} is not a token path")),
            }
        }
        out
    }

    /// `floor: None` requires `null`; otherwise a number at or above it.
    fn minimum(&mut self, at: &str, value: Option<&Value>, floor: Option<f64>) -> Option<f64> {
        let at = format!("{at}/minimumContrast");
        match (floor, value) {
            (None, Some(Value::Null)) => None,
            (None, _) => {
                self.fail(&at, "minimumContrast must be null for this exempt role");
                None
            }
            (Some(floor), Some(v)) => match v.as_f64() {
                Some(min) if min >= floor => Some(min),
                _ => {
                    self.fail(
                        &at,
                        format!("minimumContrast must be a number of at least {floor}"),
                    );
                    None
                }
            },
            (Some(floor), None) => {
                self.fail(
                    &at,
                    format!("minimumContrast must be a number of at least {floor}"),
                );
                None
            }
        }
    }

    fn states(&mut self, states: &Map<String, Value>) -> Vec<State> {
        for name in STATES {
            if !states.contains_key(*name) {
                self.fail(&format!("states/{name}"), format!("missing state {name}"));
            }
        }
        states
            .iter()
            .filter_map(|(name, state)| self.state(name, state))
            .collect()
    }

    fn state(&mut self, name: &str, value: &Value) -> Option<State> {
        let at = format!("states/{name}");
        if !STATES.contains(&name) {
            self.fail(&at, format!("{name} is not a baseline state"));
            return None;
        }
        let Some(state) = value.as_object() else {
            self.fail(&at, "a state must be an object");
            return None;
        };
        let errors = self.diags.len();
        let mut text = |key: &str| {
            let v = non_blank(state.get(key));
            if v.is_none() {
                self.fail(
                    &format!("{at}/{key}"),
                    format!("{key} must be non-blank text"),
                );
            }
            v.unwrap_or_default()
        };
        let foreground = text("foreground").to_owned();
        let background = text("background").to_owned();
        text("appliesTo");
        text("behaviour");
        let disabled = name == DISABLED_STATE;
        if disabled != (foreground == DISABLED_TOKEN) {
            self.fail(
                &format!("{at}/foreground"),
                format!("only the {DISABLED_STATE} state uses {DISABLED_TOKEN}"),
            );
        }
        let minimum = self.minimum(
            &at,
            state.get("minimumContrast"),
            (!disabled).then_some(TEXT_MINIMUM),
        );
        (self.diags.len() == errors).then(|| State {
            name: name.to_owned(),
            foreground,
            background,
            minimum,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use serde_json::json;

    use super::*;

    fn repo_usage() -> Value {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        json::read(&root.join("tokens/usage.json"), root).unwrap()
    }

    fn rules(v: &Value) -> Vec<Rule> {
        match Usage::parse(v, "tokens/usage.json") {
            Ok(_) => Vec::new(),
            Err(f) => f.0.iter().map(|d| d.rule).collect(),
        }
    }

    #[test]
    fn p5_repository_usage_passes() {
        let usage = Usage::parse(&repo_usage(), "tokens/usage.json").unwrap();
        assert_eq!(usage.roles().len(), BASELINE_ROLES.len());
        assert_eq!(usage.states().len(), STATES.len());
        // 57 role/background pairs plus the seven non-disabled states.
        assert_eq!(usage.pairs().len(), 64);
    }

    #[test]
    fn p5_schema_version_must_be_1() {
        let mut v = repo_usage();
        v["schemaVersion"] = json!(2);
        assert_eq!(rules(&v), vec![Rule::Usage]);
    }

    #[test]
    fn p5_role_kind_is_protected() {
        let mut v = repo_usage();
        v["roles"]["text-help"]["kind"] = json!("fill");
        assert_eq!(rules(&v), vec![Rule::Usage]);
    }

    #[test]
    fn p5_text_roles_need_4_5() {
        let mut v = repo_usage();
        v["roles"]["text"]["minimumContrast"] = json!(3);
        assert_eq!(rules(&v), vec![Rule::Usage]);
    }

    #[test]
    fn p5_exempt_roles_need_null() {
        let mut v = repo_usage();
        v["roles"]["canvas"]["minimumContrast"] = json!(4.5);
        assert_eq!(rules(&v), vec![Rule::Usage]);
    }

    #[test]
    fn p5_only_text_disabled_binds_ink_disabled() {
        let mut v = repo_usage();
        v["roles"]["text-help"]["token"] = json!(DISABLED_TOKEN);
        assert_eq!(rules(&v), vec![Rule::Usage]);
        let mut v = repo_usage();
        v["states"]["invalid"]["foreground"] = json!(DISABLED_TOKEN);
        assert_eq!(rules(&v), vec![Rule::Usage]);
    }

    #[test]
    fn p5_states_are_exact_and_described() {
        let mut v = repo_usage();
        v["states"].as_object_mut().unwrap().remove("loading");
        v["states"]["hover"]["behaviour"] = json!("  ");
        assert_eq!(rules(&v), vec![Rule::Usage, Rule::Usage]);
    }

    #[test]
    fn p5_backgrounds_are_unique() {
        let mut v = repo_usage();
        v["roles"]["link"]["backgrounds"] =
            json!(["semantic.color.bg-base", "semantic.color.bg-base"]);
        assert_eq!(rules(&v), vec![Rule::Usage]);
    }
}
