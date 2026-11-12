//! WCAG 2.2 contrast (P6): luminance, ratios on unrounded and HEX-rounded
//! components, and the step 5 contrast table expanded to full token paths.

use indexmap::IndexMap;

/// Names of the `semantic.color.` group unless prefixed `button.`, which means
/// `component.button.primary.`.
const SEMANTIC: &str = "semantic.color.";
const BUTTON: &str = "component.button.primary.";

/// S7: the seven surface backgrounds.
const S7: &[&str] = &[
    "bg-sunken",
    "bg-base",
    "bg-surface",
    "bg-raised",
    "bg-hover",
    "bg-pressed",
    "bg-selected",
];

/// The eleven `syn-*` colours.
const SYNTAX: &[&str] = &[
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
];

/// ANSI slots 1–15: red…white, then bright-black…bright-white (slot 0,
/// `ansi-black`, is handled separately because it is a Day-only foreground).
const ANSI_SLOTS: &[&str] = &[
    "ansi-red",
    "ansi-green",
    "ansi-yellow",
    "ansi-blue",
    "ansi-magenta",
    "ansi-cyan",
    "ansi-white",
    "ansi-bright-black",
    "ansi-bright-red",
    "ansi-bright-green",
    "ansi-bright-yellow",
    "ansi-bright-blue",
    "ansi-bright-magenta",
    "ansi-bright-cyan",
    "ansi-bright-white",
];

/// One row of the step 5 table. `foreground` × `backgrounds` produce pairs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    pub foreground: &'static [&'static str],
    pub backgrounds: &'static [&'static str],
    /// Minimum contrast ratio in every mode unless `night_minimum` applies.
    pub minimum: f64,
    /// Minimum used in Night instead of `minimum`, when set.
    pub night_minimum: Option<f64>,
    /// Only checked in Day.
    pub day_only: bool,
    pub basis: &'static str,
}

/// A required foreground/background pair with full token paths.
#[derive(Clone, Debug, PartialEq)]
pub struct ContrastPair {
    pub fg: String,
    pub bg: String,
    pub min: f64,
    pub basis: &'static str,
}

/// The step 5 contrast table. Names expand as documented on the module
/// constants: `S7`, the eleven `syn-*`, and ANSI slots 1–15. Full paths come
/// from [`token_path`].
pub const CONTRAST_TABLE: &[Row] = &[
    Row {
        foreground: &["ink", "ink-muted"],
        backgrounds: S7,
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N, U text",
    },
    Row {
        foreground: &["ink-subtle"],
        backgrounds: &["bg-base", "bg-surface", "bg-raised"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["frost", "fern", "accent", "accent-hover"],
        backgrounds: S7,
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N text-safe on every surface; links",
    },
    Row {
        foreground: &["focus-ring"],
        backgrounds: S7,
        minimum: 3.0,
        night_minimum: None,
        day_only: false,
        basis: "N, DG Focus",
    },
    Row {
        foreground: &["line-strong"],
        backgrounds: &["bg-base", "bg-surface"],
        minimum: 3.0,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["ink-inverse"],
        backgrounds: &["bg-inverse"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["on-accent"],
        backgrounds: &["accent", "accent-hover", "accent-pressed"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["accent-ink"],
        backgrounds: &["accent-soft"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["on-danger"],
        backgrounds: &["danger"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["info", "ink"],
        backgrounds: &["info-soft"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["success", "ink"],
        backgrounds: &["success-soft"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["warning", "ink"],
        backgrounds: &["warning-soft"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["danger", "ink"],
        backgrounds: &["danger-soft"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["hearth", "ink"],
        backgrounds: &["hearth-soft"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["lilac", "ink"],
        backgrounds: &["lilac-soft"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "N",
    },
    Row {
        foreground: &["ink"],
        backgrounds: &[
            "selection",
            "diff-add-bg",
            "diff-del-bg",
            "warning-soft",
            "search-match",
            "search-match-current",
        ],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "DG",
    },
    Row {
        foreground: SYNTAX,
        backgrounds: &["bg-base", "bg-sunken", "bg-surface"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "DG",
    },
    Row {
        foreground: &["syn-comment"],
        backgrounds: &["bg-base"],
        minimum: 5.0,
        night_minimum: Some(7.5),
        day_only: false,
        basis: "TC, DG",
    },
    Row {
        foreground: ANSI_SLOTS,
        backgrounds: &["bg-base"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "DG",
    },
    Row {
        foreground: &["ansi-black"],
        backgrounds: &["bg-base"],
        minimum: 4.5,
        night_minimum: None,
        day_only: true,
        basis: "TC",
    },
    Row {
        foreground: &[
            "lilac",
            "aurora",
            "ink-subtle",
            "git-added",
            "git-modified",
            "git-removed",
            "git-ignored",
        ],
        backgrounds: &["bg-base"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "DG prompt, trees",
    },
    Row {
        foreground: &["danger", "warning", "info", "ink-muted"],
        backgrounds: &["bg-base", "bg-surface"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "TC diagnostics",
    },
    Row {
        foreground: &["git-added"],
        backgrounds: &["diff-add-bg"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "DG diffs",
    },
    Row {
        foreground: &["git-modified"],
        backgrounds: &["warning-soft"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "DG diffs",
    },
    Row {
        foreground: &["git-removed"],
        backgrounds: &["diff-del-bg"],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "DG diffs",
    },
    Row {
        foreground: &["button.foreground"],
        backgrounds: &[
            "button.background",
            "button.hover-background",
            "button.pressed-background",
        ],
        minimum: 4.5,
        night_minimum: None,
        day_only: false,
        basis: "TC",
    },
    Row {
        foreground: &["button.focus-color"],
        backgrounds: S7,
        minimum: 3.0,
        night_minimum: None,
        day_only: false,
        basis: "TC",
    },
];

/// Full token path for a table name.
#[must_use]
pub fn token_path(name: &str) -> String {
    match name.strip_prefix("button.") {
        Some(rest) => format!("{BUTTON}{rest}"),
        None => format!("{SEMANTIC}{name}"),
    }
}

/// Expand [`CONTRAST_TABLE`] for a mode (`"night"`, `"day"` or `"aurora"`).
/// Duplicates are kept; combine with other sources through [`merge_pairs`].
/// Unknown mode names receive only the all-mode rows.
#[must_use]
pub fn pairs(mode: &str) -> Vec<ContrastPair> {
    let mut out = Vec::new();
    for row in CONTRAST_TABLE {
        if row.day_only && mode != "day" {
            continue;
        }
        let min = match (mode, row.night_minimum) {
            ("night", Some(night)) => night,
            _ => row.minimum,
        };
        for fg in row.foreground {
            for bg in row.backgrounds {
                out.push(ContrastPair {
                    fg: token_path(fg),
                    bg: token_path(bg),
                    min,
                    basis: row.basis,
                });
            }
        }
    }
    out
}

/// De-duplicate by `(fg, bg)`, keeping the highest minimum (its basis comes
/// with it). Order is the first-seen order of each pair.
#[must_use]
pub fn merge_pairs(pairs: Vec<ContrastPair>) -> Vec<ContrastPair> {
    let mut merged: IndexMap<(String, String), ContrastPair> = IndexMap::new();
    for pair in pairs {
        let key = (pair.fg.clone(), pair.bg.clone());
        match merged.get(&key) {
            Some(existing) if existing.min >= pair.min => {}
            _ => {
                merged.insert(key, pair);
            }
        }
    }
    merged.into_values().collect()
}

/// Relative luminance of sRGB components in 0..=1 (WCAG 2.2).
#[must_use]
pub fn luminance(rgb: [f64; 3]) -> f64 {
    let [r, g, b] = rgb.map(|c| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// WCAG contrast ratio of two colours on unrounded components.
#[must_use]
pub fn ratio(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Components rounded to 8-bit HEX precision (half-up), as `k / 255`.
/// Inputs are sRGB components in 0..=1.
#[must_use]
pub fn hex_rounded(rgb: [f64; 3]) -> [f64; 3] {
    rgb.map(|c| (c * 255.0).round() / 255.0)
}

/// Contrast ratio on HEX-rounded components.
#[must_use]
pub fn ratio_hex(a: [f64; 3], b: [f64; 3]) -> f64 {
    ratio(hex_rounded(a), hex_rounded(b))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    const BLACK: [f64; 3] = [0.0, 0.0, 0.0];
    const WHITE: [f64; 3] = [1.0, 1.0, 1.0];

    fn hex(s: &str) -> [f64; 3] {
        let n = u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap();
        [16, 8, 0].map(|shift| f64::from((n >> shift) & 0xff) / 255.0)
    }

    #[test]
    fn p6_black_on_white_is_21() {
        assert!(close(ratio(BLACK, WHITE), 21.0, 1e-9));
    }

    #[test]
    fn p6_767676_on_white_is_about_4_5422() {
        assert!(close(ratio(hex("#767676"), WHITE), 4.5422, 1e-3));
    }

    #[test]
    fn p6_777777_on_white_is_about_4_4781() {
        assert!(close(ratio(hex("#777777"), WHITE), 4.4781, 1e-3));
    }

    #[test]
    fn p6_hex_rounding_half_up() {
        let rounded = hex_rounded([0.5, 0.5 / 255.0, 1.0]);
        assert!(close(rounded[0] * 255.0, 128.0, 1e-9));
        assert!(close(rounded[1] * 255.0, 1.0, 1e-9));
        assert!(close(rounded[2] * 255.0, 255.0, 1e-9));
    }

    #[test]
    fn p6_hex_only_failure_is_visible() {
        // [0.465]*3 on white passes unrounded (~4.504) but rounds to 119/255 and fails (~4.487).
        let c = [0.465; 3];
        assert!(ratio(c, WHITE) >= 4.5);
        assert!(ratio_hex(c, WHITE) < 4.5);
    }

    #[test]
    fn p6_luminance_threshold_is_linear_segment() {
        // 0.04045 is the linear/gamma boundary: the segment below it divides by 12.92.
        assert!(close(luminance([0.04045; 3]), 0.04045 / 12.92, 1e-12));
    }

    #[test]
    fn p6_ratio_is_symmetric() {
        let a = hex("#1d2433");
        let b = hex("#f0c674");
        assert!(close(ratio(a, b), ratio(b, a), 1e-12));
    }

    #[test]
    fn p6_table_names_expand_to_full_paths() {
        assert_eq!(token_path("ink"), "semantic.color.ink");
        assert_eq!(
            token_path("button.background"),
            "component.button.primary.background"
        );
    }

    #[test]
    fn p6_table_row_counts_match_plan() {
        // 11 syn-* × 3 backgrounds + syn-comment × 1 extra (5) row + 15 ansi slots.
        let rows = |fg: &str| pairs("day").into_iter().filter(|p| p.fg == fg).count();
        assert_eq!(rows("semantic.color.syn-regex"), 3);
        assert_eq!(rows("semantic.color.ansi-bright-white"), 1);
        assert_eq!(rows("semantic.color.ansi-black"), 1);
        assert_eq!(rows("semantic.color.focus-ring"), 7);
        assert_eq!(rows("component.button.primary.focus-color"), 7);
    }

    #[test]
    fn p6_syn_comment_is_stricter_in_night() {
        let min_for = |mode: &str| {
            merge_pairs(pairs(mode))
                .iter()
                .find(|p| p.fg == "semantic.color.syn-comment" && p.bg == "semantic.color.bg-base")
                .map(|p| p.min)
                .unwrap()
        };
        assert!(close(min_for("night"), 7.5, 0.0));
        assert!(close(min_for("day"), 5.0, 0.0));
    }

    #[test]
    fn p6_ansi_black_is_day_only() {
        let has = |mode: &str| {
            pairs(mode)
                .iter()
                .any(|p| p.fg == "semantic.color.ansi-black")
        };
        assert!(has("day"));
        assert!(!has("night"));
        assert!(!has("aurora"));
    }

    #[test]
    fn p6_button_pairs_use_component_paths() {
        let all = pairs("night");
        assert!(
            all.iter()
                .any(|p| p.fg == "component.button.primary.foreground"
                    && p.bg == "component.button.primary.hover-background"
                    && p.min.to_bits() == 4.5f64.to_bits())
        );
    }

    #[test]
    fn p6_merge_keeps_highest_minimum_and_first_order() {
        let mk = |fg: &str, bg: &str, min: f64, basis: &'static str| ContrastPair {
            fg: fg.into(),
            bg: bg.into(),
            min,
            basis,
        };
        let merged = merge_pairs(vec![
            mk("a", "x", 4.5, "one"),
            mk("b", "x", 3.0, "two"),
            mk("a", "x", 7.5, "three"),
            mk("b", "x", 2.0, "four"),
        ]);
        assert_eq!(merged.len(), 2);
        assert_eq!(
            (merged[0].fg.as_str(), merged[0].min, merged[0].basis),
            ("a", 7.5, "three")
        );
        assert_eq!(
            (merged[1].fg.as_str(), merged[1].min, merged[1].basis),
            ("b", 3.0, "two")
        );
    }

    #[test]
    fn p6_merge_removes_all_night_duplicates() {
        let merged = merge_pairs(pairs("night"));
        let mut keys: Vec<_> = merged.iter().map(|p| (&p.fg, &p.bg)).collect();
        let before = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), before);
        let comment = merged
            .iter()
            .find(|p| p.fg == "semantic.color.syn-comment" && p.bg == "semantic.color.bg-base")
            .unwrap();
        assert!(close(comment.min, 7.5, 0.0));
    }
}
