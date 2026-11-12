//! Deprecation warnings (P9): a layer that uses a deprecated token, or a
//! theme or override that reassigns a path deprecated in an earlier state.
//! A path stays deprecated once deprecated; definitions alone never warn.

use std::collections::HashMap;

use indexmap::IndexSet;

use crate::diagnostic::{Diagnostic, Rule};
use crate::dtcg::graph::Resolved;
use crate::dtcg::tree::{Layer, TokenPath};

/// Deprecated paths seen in any state of one composition.
#[derive(Clone, Debug, Default)]
pub struct Tracker {
    /// Path → explanation (`None` when `$deprecated: true`).
    deprecated: HashMap<TokenPath, Option<String>>,
}

impl Tracker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the deprecations of a newly resolved state. Returns the paths
    /// that were deprecated before it, for [`Tracker::check`].
    pub fn record(&mut self, resolved: &Resolved) -> HashMap<TokenPath, Option<String>> {
        let earlier = self.deprecated.clone();
        for (path, token) in resolved {
            if let Some(d) = &token.deprecated {
                self.deprecated
                    .entry(path.clone())
                    .or_insert_with(|| d.reason.clone());
            }
        }
        earlier
    }

    /// Warnings for `layer` in the state `resolved`: each token whose
    /// definition comes from the layer and references a deprecated token
    /// and, when `earlier` is given (themes and overrides), each assigned
    /// path that was deprecated in an earlier state. Once per (token, target).
    #[must_use]
    pub fn check<'a>(
        &self,
        layer: &Layer,
        location: &str,
        assigned: impl IntoIterator<Item = &'a TokenPath>,
        resolved: &Resolved,
        earlier: Option<&HashMap<TokenPath, Option<String>>>,
    ) -> Vec<Diagnostic> {
        let mut uses: IndexSet<(&TokenPath, &TokenPath, Option<&String>)> = IndexSet::new();
        for path in assigned {
            if let Some((target, reason)) = earlier.and_then(|e| e.get_key_value(path)) {
                uses.insert((path, target, reason.as_ref()));
            }
            let Some(token) = resolved.get(path).filter(|t| t.origin == Some(layer.id)) else {
                continue;
            };
            for target in &token.references {
                if let Some((target, reason)) = self.deprecated.get_key_value(target) {
                    uses.insert((path, target, reason.as_ref()));
                }
            }
        }
        uses.into_iter()
            .map(|(path, target, reason)| {
                let reason = reason.map_or("no replacement declared", String::as_str);
                let how = if path == target { "assigns" } else { "uses" };
                Diagnostic::warning(
                    Rule::Deprecated,
                    format!("{location}: {path}"),
                    format!("{how} deprecated {target}: {reason}"),
                )
            })
            .collect()
    }
}
