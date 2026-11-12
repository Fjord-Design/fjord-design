//! CSS custom properties (O3).

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule};
use crate::dtcg::value::{Color, Dimension, Duration, LengthUnit, Shadow, TimeUnit, TypedValue};
use crate::json::number_text;
use crate::policy::contract::CONTRACT_VERSION;
use crate::project::Composition;

const PREFIX: &str = "--fjord-token-";

/// Font family names written without quotes: the CSS Fonts 4 generic
/// families plus the two system-font aliases (L14).
const BARE_FAMILIES: &[&str] = &[
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

/// Typography expands to one property per field, in this order.
const TYPOGRAPHY_FIELDS: [&str; 5] = [
    "font-family",
    "font-size",
    "font-weight",
    "letter-spacing",
    "line-height",
];

/// Render the composition as a `:root` block of custom properties, sorted by token path.
///
/// # Errors
/// [`Rule::CssName`] for every pair of tokens whose property names collide.
pub fn css(c: &Composition) -> Outcome<String> {
    let mut tokens: Vec<(&str, &TypedValue)> =
        c.tokens().map(|(path, t)| (&**path, &t.value)).collect();
    tokens.sort_unstable_by(|a, b| a.0.cmp(b.0));
    stylesheet(c.mode().as_str(), &tokens).map_err(|mut failure| {
        for d in &mut failure.0 {
            d.context = Some(c.label().to_owned());
        }
        failure
    })
}

/// The stylesheet for `tokens`, given in output order.
fn stylesheet(mode: &str, tokens: &[(&str, &TypedValue)]) -> Outcome<String> {
    let mut out =
        format!("/* Fjord contract {CONTRACT_VERSION}; mode {mode}; sRGB HEX */\n:root {{\n");
    let mut owners: HashMap<String, &str> = HashMap::new();
    let mut diags = Vec::new();
    for &(path, value) in tokens {
        for (name, text) in declarations(&property_name(path), value) {
            if let Some(other) = owners.get(&name) {
                diags.push(
                    Diagnostic::error(
                        Rule::CssName,
                        path,
                        format!("CSS property {name} is also produced by {other}"),
                    )
                    .with_help("rename one of the tokens so their CSS property names differ"),
                );
                continue;
            }
            let _ = writeln!(out, "  {name}: {text};");
            owners.insert(name, path);
        }
    }
    out.push_str("}\n");
    if diags.is_empty() {
        Ok(out)
    } else {
        Err(Failure(diags))
    }
}

/// `--fjord-token-` + the path with `.` → `-` and `$` → `\$`.
fn property_name(path: &str) -> String {
    let mut name = String::with_capacity(PREFIX.len() + path.len() + 1);
    name.push_str(PREFIX);
    for ch in path.chars() {
        match ch {
            '.' => name.push('-'),
            '$' => name.push_str("\\$"),
            _ => name.push(ch),
        }
    }
    name
}

/// The declarations for one token: one, or five for typography.
fn declarations(name: &str, value: &TypedValue) -> Vec<(String, String)> {
    let text = match value {
        TypedValue::Color(c) => color(c),
        TypedValue::Dimension(d) => dimension(d),
        TypedValue::Duration(d) => duration(d),
        TypedValue::Number(n) | TypedValue::FontWeight(n) => number_text(*n),
        TypedValue::FontFamily(families) => font_family(families),
        TypedValue::CubicBezier(points) => cubic_bezier(points),
        TypedValue::Shadow(s) => shadow(s),
        TypedValue::Typography(t) => {
            let values = [
                font_family(&t.font_family),
                dimension(&t.font_size),
                number_text(t.font_weight),
                dimension(&t.letter_spacing),
                number_text(t.line_height),
            ];
            return TYPOGRAPHY_FIELDS
                .iter()
                .zip(values)
                .map(|(field, text)| (format!("{name}-{field}"), text))
                .collect();
        }
    };
    vec![(name.to_owned(), text)]
}

fn color(c: &Color) -> String {
    c.to_hex()
}

fn dimension(d: &Dimension) -> String {
    let unit = match d.unit {
        LengthUnit::Px => "px",
        LengthUnit::Rem => "rem",
    };
    number_text(d.value) + unit
}

fn duration(d: &Duration) -> String {
    let unit = match d.unit {
        TimeUnit::Ms => "ms",
        TimeUnit::S => "s",
    };
    number_text(d.value) + unit
}

fn cubic_bezier(points: &[f64; 4]) -> String {
    let [a, b, c, d] = points.map(number_text);
    format!("cubic-bezier({a}, {b}, {c}, {d})")
}

fn shadow(s: &Shadow) -> String {
    s.layers()
        .iter()
        .map(|layer| {
            format!(
                "{}{} {} {} {} {}",
                if layer.inset { "inset " } else { "" },
                dimension(&layer.offset_x),
                dimension(&layer.offset_y),
                dimension(&layer.blur),
                dimension(&layer.spread),
                color(&layer.color),
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn font_family(families: &[String]) -> String {
    families
        .iter()
        .map(|family| family_name(family))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A generic family keyword stays bare; any other name becomes a CSS string.
fn family_name(family: &str) -> String {
    if BARE_FAMILIES.contains(&family) {
        return family.to_owned();
    }
    let mut out = String::with_capacity(family.len() + 2);
    out.push('"');
    for ch in family.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            // A raw newline or other control character cannot appear in a CSS string.
            c if c.is_control() => {
                let _ = write!(out, "\\{:x} ", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::dtcg::value::{ShadowLayer, Typography};

    fn px(value: f64) -> Dimension {
        Dimension {
            value,
            unit: LengthUnit::Px,
        }
    }

    fn rgb(components: [f64; 3], alpha: f64) -> Color {
        Color { components, alpha }
    }

    fn layer(inset: bool) -> ShadowLayer {
        ShadowLayer {
            color: rgb([0.0, 0.0, 0.0], 0.5),
            offset_x: px(0.0),
            offset_y: px(1.0),
            blur: px(2.5),
            spread: px(-1.0),
            inset,
        }
    }

    #[test]
    fn o3_names_replace_dots_and_escape_dollar() {
        assert_eq!(
            property_name("semantic.color.ink"),
            "--fjord-token-semantic-color-ink"
        );
        assert_eq!(
            property_name("primitive.space.$root"),
            "--fjord-token-primitive-space-\\$root"
        );
        assert_ne!(
            property_name("primitive.space.$root"),
            property_name("primitive.space.root")
        );
    }

    #[test]
    fn o3_colors_are_hex_with_alpha_only_below_one() {
        assert_eq!(color(&rgb([1.0, 1.0, 1.0], 1.0)), "#ffffff");
        assert_eq!(color(&rgb([0.5, 0.5 / 255.0, 1.0], 1.0)), "#8001ff");
        assert_eq!(color(&rgb([0.0, 0.0, 0.0], 0.5)), "#00000080");
    }

    #[test]
    fn o3_dimensions_and_durations_keep_units_with_canonical_numbers() {
        assert_eq!(dimension(&px(15.0)), "15px");
        assert_eq!(
            dimension(&Dimension {
                value: 0.875,
                unit: LengthUnit::Rem
            }),
            "0.875rem"
        );
        assert_eq!(
            duration(&Duration {
                value: 120.0,
                unit: TimeUnit::Ms
            }),
            "120ms"
        );
        assert_eq!(
            duration(&Duration {
                value: 0.26,
                unit: TimeUnit::S
            }),
            "0.26s"
        );
    }

    #[test]
    fn o3_cubic_bezier_and_numbers() {
        assert_eq!(
            cubic_bezier(&[0.2, 0.0, 0.0, 1.0]),
            "cubic-bezier(0.2, 0, 0, 1)"
        );
        let decl = declarations("--n", &TypedValue::Number(10.0));
        assert_eq!(decl, vec![("--n".to_owned(), "10".to_owned())]);
        let decl = declarations("--w", &TypedValue::FontWeight(600.0));
        assert_eq!(decl[0].1, "600");
    }

    #[test]
    fn o3_shadow_single_object_and_layers() {
        assert_eq!(
            shadow(&Shadow::Single(layer(false))),
            "0px 1px 2.5px -1px #00000080"
        );
        assert_eq!(
            shadow(&Shadow::Layers(vec![layer(true), layer(false)])),
            "inset 0px 1px 2.5px -1px #00000080, 0px 1px 2.5px -1px #00000080"
        );
    }

    #[test]
    fn l14_font_families_quote_names_and_keep_generic_keywords_bare() {
        let stack: Vec<String> = [
            "Noto Sans",
            "Open Sans",
            "system-ui",
            "-apple-system",
            "Segoe UI",
            "sans-serif",
        ]
        .map(str::to_owned)
        .into();
        assert_eq!(
            font_family(&stack),
            r#""Noto Sans", "Open Sans", system-ui, -apple-system, "Segoe UI", sans-serif"#
        );
        for keyword in BARE_FAMILIES {
            assert_eq!(family_name(keyword), *keyword);
        }
        assert_eq!(family_name("ui-sans-serif"), "ui-sans-serif");
        assert_eq!(family_name("Sans-Serif"), "\"Sans-Serif\"");
    }

    #[test]
    fn l14_single_string_family_is_one_name() {
        let decl = declarations("--f", &TypedValue::FontFamily(vec!["Noto Sans".to_owned()]));
        assert_eq!(decl[0].1, "\"Noto Sans\"");
    }

    #[test]
    fn family_names_escape_quotes_backslashes_and_controls() {
        assert_eq!(family_name(r#"A "B" \C"#), r#""A \"B\" \\C""#);
        assert_eq!(family_name("A\nB"), "\"A\\a B\"");
    }

    #[test]
    fn o3_typography_expands_to_five_properties() {
        let t = TypedValue::Typography(Typography {
            font_family: vec!["Noto Sans".to_owned(), "sans-serif".to_owned()],
            font_size: px(15.0),
            font_weight: 400.0,
            letter_spacing: px(0.0),
            line_height: 1.5,
        });
        let names: Vec<(String, String)> =
            declarations("--fjord-token-semantic-typography-body", &t);
        assert_eq!(
            names,
            [
                ("font-family", "\"Noto Sans\", sans-serif"),
                ("font-size", "15px"),
                ("font-weight", "400"),
                ("letter-spacing", "0px"),
                ("line-height", "1.5"),
            ]
            .map(|(f, v)| (
                format!("--fjord-token-semantic-typography-body-{f}"),
                v.to_owned()
            ))
        );
    }

    #[test]
    fn o3_stylesheet_layout() {
        let ink = TypedValue::Color(rgb([0.0, 0.0, 0.0], 1.0));
        let gap = TypedValue::Dimension(px(4.0));
        let out = stylesheet(
            "night",
            &[("semantic.color.ink", &ink), ("semantic.gap", &gap)],
        )
        .unwrap();
        assert_eq!(
            out,
            "/* Fjord contract 0.5.0; mode night; sRGB HEX */\n:root {\n  --fjord-token-semantic-color-ink: #000000;\n  --fjord-token-semantic-gap: 4px;\n}\n"
        );
    }

    #[test]
    fn o3_name_collisions_fail() {
        let gap = TypedValue::Dimension(px(4.0));
        let failure = stylesheet(
            "night",
            &[("semantic.a.b-c", &gap), ("semantic.a-b.c", &gap)],
        )
        .unwrap_err();
        assert_eq!(failure.0.len(), 1);
        assert_eq!(failure.0[0].rule, Rule::CssName);

        let body = TypedValue::Typography(Typography {
            font_family: vec!["serif".to_owned()],
            font_size: px(15.0),
            font_weight: 400.0,
            letter_spacing: px(0.0),
            line_height: 1.5,
        });
        let failure = stylesheet(
            "night",
            &[("semantic.body", &body), ("semantic.body-font-size", &gap)],
        )
        .unwrap_err();
        assert_eq!(failure.0[0].rule, Rule::CssName);
    }
}
