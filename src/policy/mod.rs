//! Project policy: the contract, layer rules (C3/C4), role constraints
//! (RF6, P1–P4), usage (P5), contrast (P6), syntax separation (P7), design
//! seeds (P8) and deprecation warnings (P9).

pub mod contract;
pub mod contrast;
pub mod deprecation;
pub mod layers;
pub mod roles;
pub mod seeds;
pub mod syntax;
pub mod usage;

use crate::diagnostic::{Diagnostic, Rule};
use crate::dtcg::graph::Resolved;
use crate::dtcg::value::{Color, TypedValue};

/// Points diagnostics about a composed state at the file that defines each
/// token.
#[derive(Clone, Copy, Debug)]
pub struct Locator<'a> {
    /// Display path of each layer file, indexed by `LayerId`.
    pub files: &'a [String],
    /// Display path of the layer applied last, for tokens without origin.
    pub current: &'a str,
}

impl Locator<'_> {
    /// `<file>: <path>`, where `file` is the layer whose definition the
    /// token carries (C6), else the current layer.
    #[must_use]
    pub fn token(&self, resolved: &Resolved, path: &str) -> String {
        let file = resolved
            .get(path)
            .and_then(|t| t.origin)
            .and_then(|o| self.files.get(usize::from(o)))
            .map_or(self.current, String::as_str);
        format!("{file}: {path}")
    }
}

/// The resolved colour at `path`, if it is a colour token.
#[must_use]
pub fn color<'r>(resolved: &'r Resolved, path: &str) -> Option<&'r Color> {
    match &resolved.get(path)?.value {
        TypedValue::Color(c) => Some(c),
        _ => None,
    }
}

/// P6: every pair passes on unrounded and on HEX-rounded components; a
/// translucent colour in a pair fails. All failures are reported.
#[must_use]
pub fn check_contrast(
    resolved: &Resolved,
    pairs: &[contrast::ContrastPair],
    at: Locator<'_>,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for pair in pairs {
        let fail = |message: String| {
            Diagnostic::error(Rule::Contrast, at.token(resolved, &pair.fg), message)
        };
        let (Some(fg), Some(bg)) = (color(resolved, &pair.fg), color(resolved, &pair.bg)) else {
            out.push(fail(format!(
                "{} on {} cannot be checked: both must be colour tokens",
                pair.fg, pair.bg
            )));
            continue;
        };
        if fg.alpha < 1.0 || bg.alpha < 1.0 {
            out.push(
                fail(format!(
                    "{} on {} uses a translucent colour; contrast pairs must be opaque",
                    pair.fg, pair.bg
                ))
                .with_help("use opaque colours in checked foreground/background pairs"),
            );
            continue;
        }
        let ratio = contrast::ratio(fg.components, bg.components);
        let hex = contrast::ratio_hex(fg.components, bg.components);
        if ratio < pair.min || hex < pair.min {
            out.push(
                fail(format!(
                    "{} on {} is {ratio:.2}:1 (HEX {hex:.2}:1); needs {}:1",
                    pair.fg, pair.bg, pair.min
                ))
                .with_help(format!(
                    "adjust the colours until both the exact and the 8-bit HEX values reach {}:1 ({})",
                    pair.min, pair.basis
                )),
            );
        }
    }
    out
}

/// P7: `OKLab` separation of the syntax colours.
#[must_use]
pub fn check_syntax(resolved: &Resolved, at: Locator<'_>) -> Vec<Diagnostic> {
    let colors =
        |name: &str| color(resolved, &format!("semantic.color.{name}")).map(|c| c.components);
    match syntax::check(&colors) {
        Err(message) => vec![Diagnostic::error(Rule::SyntaxDistance, at.current, message)],
        Ok(failures) => failures
            .into_iter()
            .map(|d| {
                let second = format!("semantic.color.{}", d.second);
                Diagnostic::error(
                    Rule::SyntaxDistance,
                    at.token(resolved, &second),
                    format!(
                        "semantic.color.{} and {second} are {:.2} apart in OKLab ×100; need {}",
                        d.first, d.delta, d.minimum
                    ),
                )
                .with_help("separate these syntax colours")
            })
            .collect(),
    }
}
