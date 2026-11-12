//! Layer rules: the base and mode split of mode roles (C3) and the
//! constraints on themes and overrides (C4).

use std::collections::HashSet;

use indexmap::IndexSet;
use serde_json::{Map, Value};

use crate::diagnostic::{Diagnostic, Rule};
use crate::dtcg::graph::Resolved;
use crate::dtcg::tree::{TokenPath, visit_tokens};
use crate::dtcg::value::TokenType;
use crate::json;

const OVERRIDE_NAMESPACES: [&str; 2] = ["semantic", "component"];

/// C3: base assigns no mode role; the mode layer assigns exactly the mode
/// roles.
#[must_use]
pub fn mode_roles<'a>(
    mode_roles: impl IntoIterator<Item = &'a TokenPath>,
    base: &IndexSet<TokenPath>,
    base_location: &str,
    mode: &IndexSet<TokenPath>,
    mode_location: &str,
) -> Vec<Diagnostic> {
    let roles: IndexSet<&TokenPath> = mode_roles.into_iter().collect();
    let mut out = Vec::new();
    for path in base.iter().filter(|p| roles.contains(p)) {
        out.push(
            Diagnostic::error(
                Rule::ModeRoles,
                format!("{base_location}: {path}"),
                format!("base defines the mode role {path}"),
            )
            .with_help("define mode colour and shadow roles in every mode layer instead"),
        );
    }
    for path in roles.iter().filter(|p| !mode.contains(**p)) {
        out.push(
            Diagnostic::error(
                Rule::ModeRoles,
                format!("{mode_location}: {path}"),
                format!("the mode layer does not define the mode role {path}"),
            )
            .with_help("each mode supplies its complete set of colour and shadow roles"),
        );
    }
    for path in mode.iter().filter(|p| !roles.contains(p)) {
        out.push(
            Diagnostic::error(
                Rule::ModeRoles,
                format!("{mode_location}: {path}"),
                format!("{path} is not a mode role"),
            )
            .with_help(
                "a mode layer assigns only required semantic.color and semantic.shadow roles",
            ),
        );
    }
    out
}

/// C4 checks of one theme or override layer.
pub struct Later<'a> {
    pub location: &'a str,
    pub previous: &'a Resolved,
    /// Paths already reported by [`Later::paths`]; later checks skip them.
    reported: HashSet<TokenPath>,
    pub diags: Vec<Diagnostic>,
}

impl<'a> Later<'a> {
    #[must_use]
    pub fn new(location: &'a str, previous: &'a Resolved) -> Self {
        Self {
            location,
            previous,
            reported: HashSet::new(),
            diags: Vec::new(),
        }
    }

    fn fail(&mut self, rule: Rule, path: &TokenPath, message: String, help: &str) {
        self.reported.insert(path.clone());
        self.diags.push(
            Diagnostic::error(rule, format!("{}: {path}", self.location), message).with_help(help),
        );
    }

    /// Every assigned path is in `semantic`/`component` and already exists,
    /// and a token the layer defines with an explicit `$type` keeps its type.
    pub fn paths<'p>(
        &mut self,
        assigned: impl IntoIterator<Item = &'p TokenPath>,
        layer: &Map<String, Value>,
    ) {
        for path in assigned {
            let namespace = path.split('.').next().unwrap_or_default();
            if !OVERRIDE_NAMESPACES.contains(&namespace) {
                self.fail(
                    Rule::OverrideNamespace,
                    path,
                    format!("{path} is outside the semantic and component namespaces"),
                    "primitive changes belong in the source palette",
                );
                continue;
            }
            let Some(before) = self.previous.get(path) else {
                self.fail(
                    Rule::OverridePath,
                    path,
                    format!("{path} does not exist in the composition"),
                    "override only existing paths; add new roles to the source contract first",
                );
                continue;
            };
            let declared = definition(layer, path)
                .and_then(|t| t.get("$type"))
                .and_then(Value::as_str)
                .and_then(|t| t.parse::<TokenType>().ok());
            if let Some(ty) = declared
                && ty != before.ty
            {
                let before = before.ty;
                self.fail(
                    Rule::OverrideType,
                    path,
                    format!("the layer changes the type of {path} from {before} to {ty}"),
                    "preserve the token's type",
                );
            }
        }
    }

    /// The expanded token set equals the previous one.
    pub fn token_set(&mut self, expanded: &Map<String, Value>) {
        let mut current = HashSet::new();
        let mut added = Vec::new();
        visit_tokens(expanded, &mut |path, _| {
            if self.previous.get(path).is_none() {
                added.push(TokenPath::from(path));
            }
            current.insert(TokenPath::from(path));
        });
        for path in added {
            if !self.reported.contains(&path) {
                self.fail(
                    Rule::OverrideTokenSet,
                    &path,
                    format!("the layer adds {path}"),
                    "a theme or override keeps the token set unchanged",
                );
            }
        }
        let removed: Vec<TokenPath> = self
            .previous
            .iter()
            .map(|(p, _)| p)
            .filter(|p| !current.contains(*p))
            .cloned()
            .collect();
        for path in removed {
            self.fail(
                Rule::OverrideTokenSet,
                &path,
                format!("the layer removes {path}"),
                "a theme or override keeps the token set unchanged",
            );
        }
    }

    /// Every token keeps its effective type, and primitive definitions and
    /// values are unchanged (compared numerically).
    pub fn state(&mut self, assigned: &IndexSet<TokenPath>, current: &Resolved) {
        let previous = self.previous;
        for (path, before) in previous {
            let Some(after) = current.get(path) else {
                continue;
            };
            if self.reported.contains(path) {
                continue;
            }
            // A dependant follows a retyped target; report the root cause only.
            let follows = !assigned.contains(path)
                && after.references.iter().any(|r| {
                    previous
                        .get(r)
                        .zip(current.get(r))
                        .is_some_and(|(a, b)| a.ty != b.ty)
                });
            if before.ty != after.ty && !follows {
                self.fail(
                    Rule::OverrideType,
                    path,
                    format!(
                        "the layer changes the type of {path} from {} to {}",
                        before.ty, after.ty
                    ),
                    "preserve the token's type",
                );
            } else if path.starts_with("primitive.")
                && !(json::same(&before.json, &after.json)
                    && same_definition(&before.definition, &after.definition))
            {
                self.fail(
                    Rule::OverridePrimitive,
                    path,
                    format!("the layer changes the primitive {path}"),
                    "primitive changes belong in the source palette",
                );
            }
        }
    }
}

fn same_definition(a: &Map<String, Value>, b: &Map<String, Value>) -> bool {
    a.len() == b.len()
        && a.iter()
            .all(|(k, v)| b.get(k).is_some_and(|w| json::same(v, w)))
}

/// The token object at a dotted `path` of a layer document.
fn definition<'d>(document: &'d Map<String, Value>, path: &str) -> Option<&'d Map<String, Value>> {
    let mut node = document;
    for seg in path.split('.') {
        node = node.get(seg)?.as_object()?;
    }
    Some(node)
}
