//! Syntax colour separation (P7): `OKLab` distance between the nine separated
//! `syn-*` colours, with 36 pairs in total.

// The OKLab matrices are kept at the digits Björn Ottosson publishes, so they can be
// compared against the source directly; digit grouping would obscure that.
#![allow(clippy::unreadable_literal, clippy::many_single_char_names)]

/// The nine `syn-*` colours whose pairwise distance is checked, as names relative to
/// `semantic.color.`. The first seven are the main syntax roles.
pub const SYNTAX_ROLES: &[&str] = &[
    "syn-comment",
    "syn-keyword",
    "syn-string",
    "syn-number",
    "syn-type",
    "syn-function",
    "syn-property",
    "syn-operator",
    "syn-regex",
];

/// Number of main roles at the start of `SYNTAX_ROLES`; a pair of two main roles needs 7.
const MAIN_ROLES: usize = 7;

/// Björn Ottosson's `OKLab` from sRGB components in 0..=1.
#[must_use]
pub fn oklab(rgb: [f64; 3]) -> [f64; 3] {
    let [r, g, b] = rgb.map(|v| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    });
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

/// Euclidean `OKLab` distance of two sRGB colours, scaled ×100.
#[must_use]
pub fn delta_e(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (a, b) = (oklab(a), oklab(b));
    100.0
        * a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
}

/// Minimum ΔE×100 required between two syntax roles.
#[must_use]
pub fn minimum_distance(first: usize, second: usize) -> f64 {
    if first < MAIN_ROLES && second < MAIN_ROLES {
        7.0
    } else {
        5.0
    }
}

/// A checked pair of syntax roles.
#[derive(Clone, Debug, PartialEq)]
pub struct SyntaxDistance {
    pub first: &'static str,
    pub second: &'static str,
    pub delta: f64,
    pub minimum: f64,
}

/// Every pair of `SYNTAX_ROLES` (36 pairs), with its distance and required minimum.
/// `colors` maps a name relative to `semantic.color.` to sRGB components; a missing
/// colour is an `Err` naming it.
///
/// # Errors
/// Returns the name of the first role that `colors` cannot resolve.
pub fn distances(colors: &dyn Fn(&str) -> Option<[f64; 3]>) -> Result<Vec<SyntaxDistance>, String> {
    let resolved = SYNTAX_ROLES
        .iter()
        .map(|name| colors(name).ok_or_else(|| format!("semantic.color.{name} is not defined")))
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(SYNTAX_ROLES.len() * (SYNTAX_ROLES.len() - 1) / 2);
    for (i, (&first, &first_rgb)) in SYNTAX_ROLES.iter().zip(&resolved).enumerate() {
        for (j, (&second, &second_rgb)) in
            SYNTAX_ROLES.iter().zip(&resolved).enumerate().skip(i + 1)
        {
            out.push(SyntaxDistance {
                first,
                second,
                delta: delta_e(first_rgb, second_rgb),
                minimum: minimum_distance(i, j),
            });
        }
    }
    Ok(out)
}

/// The pairs whose distance is below their minimum (P7 failures).
///
/// # Errors
/// See [`distances`].
pub fn check(colors: &dyn Fn(&str) -> Option<[f64; 3]>) -> Result<Vec<SyntaxDistance>, String> {
    Ok(distances(colors)?
        .into_iter()
        .filter(|d| d.delta < d.minimum)
        .collect())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn hex(s: &str) -> [f64; 3] {
        let n = u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap();
        [16, 8, 0].map(|shift| f64::from((n >> shift) & 0xff) / 255.0)
    }

    #[test]
    fn p7_oklab_of_pure_red_matches_ottosson() {
        let [l, a, b] = oklab([1.0, 0.0, 0.0]);
        assert!((l - 0.6279553606).abs() <= 5e-9, "L = {l}");
        assert!((a - 0.2248630611).abs() <= 5e-9, "a = {a}");
        assert!((b - 0.1258462985).abs() <= 5e-9, "b = {b}");
    }

    #[test]
    fn p7_identical_colours_have_zero_distance() {
        assert!(delta_e(hex("#336699"), hex("#336699")).abs() < 1e-12);
    }

    #[test]
    fn p7_there_are_36_pairs_with_the_right_minimums() {
        let all = distances(&|_| Some([0.5, 0.5, 0.5])).unwrap();
        assert_eq!(all.len(), 36);
        let main_pairs = all
            .iter()
            .filter(|d| d.minimum.to_bits() == 7.0f64.to_bits())
            .count();
        let touching = all
            .iter()
            .filter(|d| d.minimum.to_bits() == 5.0f64.to_bits())
            .count();
        assert_eq!(main_pairs, 21);
        assert_eq!(touching, 15);
        for d in &all {
            let separated = |name: &str| name == "syn-operator" || name == "syn-regex";
            if separated(d.first) || separated(d.second) {
                assert_eq!(d.minimum, 5.0, "{} / {}", d.first, d.second);
            } else {
                assert_eq!(d.minimum, 7.0, "{} / {}", d.first, d.second);
            }
        }
    }

    #[test]
    fn p7_identical_type_and_keyword_fails() {
        let palette = |name: &str| match name {
            "syn-type" | "syn-keyword" => Some(hex("#7aa2f7")),
            _ => Some(hex("#c0caf5")),
        };
        let failures = check(&palette).unwrap();
        assert!(
            failures
                .iter()
                .any(|d| d.first == "syn-keyword" && d.second == "syn-type"
                    || d.first == "syn-type" && d.second == "syn-keyword")
        );
    }

    #[test]
    fn p7_distinct_palette_passes() {
        let palette = |name: &str| {
            Some(match name {
                "syn-comment" => hex("#6a737d"),
                "syn-keyword" => hex("#ff7b72"),
                "syn-string" => hex("#a5d6ff"),
                "syn-number" => hex("#79c0ff"),
                "syn-type" => hex("#ffa657"),
                "syn-function" => hex("#d2a8ff"),
                "syn-property" => hex("#7ee787"),
                "syn-operator" => hex("#ff0000"),
                _ => hex("#f0f0f0"),
            })
        };
        let failures = check(&palette).unwrap();
        assert!(failures.is_empty(), "{failures:?}");
    }

    #[test]
    fn p7_missing_colour_is_an_error() {
        let err = check(&|name| {
            if name == "syn-regex" {
                None
            } else {
                Some([0.1; 3])
            }
        })
        .unwrap_err();
        assert!(err.contains("syn-regex"));
    }
}
