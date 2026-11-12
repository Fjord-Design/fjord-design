//! Role rules over a resolved state: the token hierarchy (RF6), required
//! roles (P1), value constraints (P2), layout safety (P3) and typography
//! stacks (P4).

use indexmap::IndexMap;

use super::Locator;
use crate::diagnostic::{Diagnostic, Rule};
use crate::dtcg::graph::Resolved;
use crate::dtcg::pointer::{self, Reference};
use crate::dtcg::tree::TokenPath;
use crate::dtcg::value::{TokenType, TypedValue};

const BUTTON: &str = "component.button.primary.";

fn namespace(path: &str) -> &str {
    path.split('.').next().unwrap_or_default()
}

/// RF6: primitives reference only primitives; semantic and component tokens
/// reference only primitive or semantic tokens; a component's whole
/// `$value` is one reference to a semantic token's whole `$value`.
#[must_use]
pub fn hierarchy(resolved: &Resolved, at: Locator<'_>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (path, token) in resolved {
        let level = namespace(path);
        if level == "component" && !binds_semantic(resolved, token.definition.get("$value")) {
            out.push(
                Diagnostic::error(
                    Rule::ComponentBinding,
                    at.token(resolved, path),
                    "a component token must be one whole-token reference to a semantic role",
                )
                .with_help("use {semantic.…} or {\"$ref\": \"#/semantic/…/$value\"}"),
            );
            continue;
        }
        for target in &token.references {
            let allowed = match level {
                "primitive" => namespace(target) == "primitive",
                _ => matches!(namespace(target), "primitive" | "semantic"),
            };
            if !allowed {
                out.push(
                    Diagnostic::error(
                        Rule::Hierarchy,
                        at.token(resolved, path),
                        format!("a {level} token cannot reference {target}"),
                    )
                    .with_help(
                        "primitives reference primitives; semantic and component tokens reference primitive or semantic tokens",
                    ),
                );
            }
        }
    }
    out
}

fn binds_semantic(resolved: &Resolved, value: Option<&serde_json::Value>) -> bool {
    let target = match value.map(pointer::parse) {
        Some(Ok(Some(Reference::Alias(path)))) => path,
        Some(Ok(Some(Reference::Pointer(segs)))) => match segs.split_last() {
            Some((last, owner)) if last == "$value" => owner.join("."),
            _ => return false,
        },
        _ => return false,
    };
    namespace(&target) == "semantic" && resolved.get(&target).is_some()
}

/// P1: every required path exists with exactly the required type.
#[must_use]
pub fn required(
    required: &IndexMap<TokenPath, TokenType>,
    resolved: &Resolved,
    at: Locator<'_>,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (path, ty) in required {
        match resolved.get(path) {
            None => out.push(
                Diagnostic::error(
                    Rule::RequiredRole,
                    format!("{}: {path}", at.current),
                    format!("required {ty} role {path} is missing"),
                )
                .with_help("restore its definition"),
            ),
            Some(token) if token.ty != *ty => out.push(Diagnostic::error(
                Rule::RequiredRole,
                at.token(resolved, path),
                format!("required role {path} must be {ty}, found {}", token.ty),
            )),
            Some(_) => {}
        }
    }
    out
}

fn px(resolved: &Resolved, path: &str) -> Option<f64> {
    match &resolved.get(path)?.value {
        TypedValue::Dimension(d) => Some(d.px()),
        _ => None,
    }
}

fn ms(resolved: &Resolved, path: &str) -> Option<f64> {
    match &resolved.get(path)?.value {
        TypedValue::Duration(d) => Some(d.ms()),
        _ => None,
    }
}

fn number(resolved: &Resolved, path: &str) -> Option<f64> {
    match &resolved.get(path)?.value {
        TypedValue::Number(n) => Some(*n),
        _ => None,
    }
}

/// P2: non-negative layout groups, positive font sizes and sizes, integer
/// z-indexes.
#[must_use]
pub fn constraints(resolved: &Resolved, at: Locator<'_>) -> Vec<Diagnostic> {
    const NON_NEGATIVE: [&str; 7] = [
        "spacing",
        "radius",
        "border-width",
        "size",
        "breakpoint",
        "layout",
        "density",
    ];
    const POSITIVE: [&str; 2] = ["font-size", "size"];
    let mut out = Vec::new();
    for (path, token) in resolved {
        let Some(rest) = path.strip_prefix("semantic.") else {
            continue;
        };
        let group = rest.split('.').next().unwrap_or_default();
        let fail = |message: String| {
            Diagnostic::error(Rule::RoleConstraint, at.token(resolved, path), message)
        };
        match &token.value {
            TypedValue::Dimension(d) => {
                let v = d.px();
                if POSITIVE.contains(&group) && v <= 0.0 {
                    out.push(fail(format!(
                        "semantic {group} values must be positive, found {v}px"
                    )));
                } else if NON_NEGATIVE.contains(&group) && v < 0.0 {
                    out.push(fail(format!(
                        "semantic {group} values must not be negative, found {v}px"
                    )));
                }
            }
            TypedValue::Number(n) if group == "z-index" && n.fract() != 0.0 => {
                out.push(fail(format!("z-index values must be integers, found {n}")));
            }
            _ => {}
        }
    }
    out
}

struct Layout<'a> {
    resolved: &'a Resolved,
    at: Locator<'a>,
    out: Vec<Diagnostic>,
}

impl Layout<'_> {
    fn fail(&mut self, path: &str, message: String, help: &str) {
        self.out.push(
            Diagnostic::error(Rule::Layout, self.at.token(self.resolved, path), message)
                .with_help(help),
        );
    }

    /// `path` ≥ `min` (px).
    fn at_least(&mut self, path: &str, min: f64, what: &str) {
        if let Some(v) = px(self.resolved, path)
            && v < min
        {
            self.fail(
                path,
                format!("{path} is {v}px; {what} needs at least {min}px"),
                "raise the value",
            );
        }
    }

    /// `lower` ≤ `upper` (px), strictly when `strict`.
    fn ordered(&mut self, lower: &str, upper: &str, strict: bool) {
        if let (Some(a), Some(b)) = (px(self.resolved, lower), px(self.resolved, upper))
            && (if strict { a >= b } else { a > b })
        {
            let op = if strict { "<" } else { "≤" };
            self.fail(
                upper,
                format!("{lower} ({a}px) {op} {upper} ({b}px) does not hold"),
                "keep the documented order",
            );
        }
    }
}

/// P3: interaction targets, densities, breakpoints, focus indicators,
/// stacking order and motion durations.
#[must_use]
pub fn layout(resolved: &Resolved, at: Locator<'_>) -> Vec<Diagnostic> {
    let size = |n: &str| format!("semantic.size.{n}");
    let mut l = Layout {
        resolved,
        at,
        out: Vec::new(),
    };
    let minimum = size("target-minimum");
    l.at_least(&minimum, 24.0, "WCAG 2.5.8 target size");
    for name in ["control-compact", "control-comfortable", "target-touch"] {
        l.ordered(&minimum, &size(name), false);
    }
    l.at_least(&size("target-touch"), 44.0, "a coarse-pointer target");
    l.ordered(
        &size("control-compact"),
        &size("control-comfortable"),
        false,
    );
    for name in ["gap", "padding"] {
        l.ordered(
            &format!("semantic.density.{name}-compact"),
            &format!("semantic.density.{name}-comfortable"),
            false,
        );
    }
    let sm = "semantic.breakpoint.breakpoint-sm";
    if let Some(v) = px(resolved, sm)
        && v <= 0.0
    {
        l.fail(
            sm,
            format!("{sm} is {v}px; it must be positive"),
            "raise the value",
        );
    }
    l.ordered(sm, "semantic.breakpoint.breakpoint-md", true);
    l.ordered(
        "semantic.breakpoint.breakpoint-md",
        "semantic.breakpoint.breakpoint-lg",
        true,
    );
    for path in [
        "semantic.border-width.border-focus",
        "semantic.layout.focus-offset",
        &format!("{BUTTON}focus-width"),
    ] {
        l.at_least(path, 2.0, "a visible focus indicator");
    }
    let z: Vec<String> = ["base", "sticky", "popover", "modal", "toast", "tooltip"]
        .iter()
        .map(|n| format!("semantic.z-index.z-{n}"))
        .collect();
    for pair in z.windows(2) {
        if let (Some(a), Some(b)) = (number(resolved, &pair[0]), number(resolved, &pair[1]))
            && a >= b
        {
            l.fail(
                &pair[1],
                format!("{} ({a}) < {} ({b}) does not hold", pair[0], pair[1]),
                "keep base < sticky < popover < modal < toast < tooltip",
            );
        }
    }
    durations(&mut l);
    l.out
}

/// none = 0 < fast ≤ base ≤ slow ≤ 300 ms (L3).
fn durations(l: &mut Layout<'_>) {
    const HELP: &str = "keep none = 0 < fast ≤ base ≤ slow ≤ 300ms";
    let d = |n: &str| format!("semantic.duration.duration-{n}");
    let [none, fast, base, slow] = ["none", "fast", "base", "slow"].map(d);
    let value = |p: &str| ms(l.resolved, p);
    let mut fails = Vec::new();
    if let Some(v) = value(&none)
        && v != 0.0
    {
        fails.push((none.clone(), format!("{none} is {v}ms; it must be 0")));
    }
    if let Some(v) = value(&fast)
        && v <= 0.0
    {
        fails.push((
            fast.clone(),
            format!("{fast} is {v}ms; it must be positive"),
        ));
    }
    for (a, b) in [(&fast, &base), (&base, &slow)] {
        if let (Some(x), Some(y)) = (value(a), value(b))
            && x > y
        {
            fails.push((
                b.clone(),
                format!("{a} ({x}ms) ≤ {b} ({y}ms) does not hold"),
            ));
        }
    }
    if let Some(v) = value(&slow)
        && v > 300.0
    {
        fails.push((
            slow.clone(),
            format!("{slow} is {v}ms; transitions stay within 300ms"),
        ));
    }
    for (path, message) in fails {
        l.fail(&path, message, HELP);
    }
}

/// P4: every `semantic.typography.*` family equals one resolved
/// `semantic.font.{sans,display,mono}` stack.
#[must_use]
pub fn typography(resolved: &Resolved, at: Locator<'_>) -> Vec<Diagnostic> {
    let stacks: Vec<&Vec<String>> = ["sans", "display", "mono"]
        .iter()
        .filter_map(
            |n| match &resolved.get(&format!("semantic.font.{n}"))?.value {
                TypedValue::FontFamily(f) => Some(f),
                _ => None,
            },
        )
        .collect();
    let mut out = Vec::new();
    for (path, token) in resolved {
        if !path.starts_with("semantic.typography.") {
            continue;
        }
        if let TypedValue::Typography(t) = &token.value
            && !stacks.contains(&&t.font_family)
        {
            out.push(
                Diagnostic::error(
                    Rule::Typography,
                    at.token(resolved, path),
                    format!(
                        "font family [{}] is not one of the semantic.font stacks",
                        t.font_family.join(", ")
                    ),
                )
                .with_help("alias fontFamily to {semantic.font.sans}, {semantic.font.display} or {semantic.font.mono}"),
            );
        }
    }
    out
}
