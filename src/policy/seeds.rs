//! Design seed validation (P8): `designSeeds` holds three primary and four accent
//! references written as `oklch(L% C H)`.

use serde_json::Value;

use crate::diagnostic::{Diagnostic, Rule};

const PRIMARY_LEN: usize = 3;
const ACCENT_LEN: usize = 4;

/// Parse an unsigned decimal `digits(.digits)?`, or a signed one when `signed`.
fn decimal(text: &str, signed: bool) -> Option<f64> {
    let body = if signed {
        text.strip_prefix('-').unwrap_or(text)
    } else {
        text
    };
    let (int, frac) = match body.split_once('.') {
        Some((int, frac)) => (int, Some(frac)),
        None => (body, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(int) || frac.is_some_and(|f| !digits(f)) {
        return None;
    }
    text.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Parse `oklch(L% C H)` into `[L, C, H]` with `L` as a fraction in 0..=1.
///
/// # Errors
/// Returns a message naming the expected form when the text does not match or a
/// value is out of range (`L` 0..=100 %, `C` 0..=1, finite `H`).
pub fn parse(text: &str) -> Result<[f64; 3], String> {
    const FORM: &str = "use oklch(L% C H)";
    let inner = text
        .strip_prefix("oklch(")
        .and_then(|rest| rest.strip_suffix(')'))
        .filter(|inner| {
            !inner.starts_with(|c: char| c.is_ascii_whitespace())
                && !inner.ends_with(|c: char| c.is_ascii_whitespace())
        })
        .ok_or_else(|| format!("{text:?} is not an oklch() value; {FORM}"))?;
    let parts: Vec<&str> = inner.split_ascii_whitespace().collect();
    let [lightness, chroma, hue] = parts.as_slice() else {
        return Err(format!(
            "{text:?} needs three values separated by spaces; {FORM}"
        ));
    };
    let l = lightness
        .strip_suffix('%')
        .and_then(|v| decimal(v, false))
        .filter(|l| (0.0..=100.0).contains(l))
        .ok_or_else(|| {
            format!("lightness {lightness:?} must be a percentage in 0..=100; {FORM}")
        })?;
    let c = decimal(chroma, false)
        .filter(|c| (0.0..=1.0).contains(c))
        .ok_or_else(|| format!("chroma {chroma:?} must be in 0..=1; {FORM}"))?;
    let h =
        decimal(hue, true).ok_or_else(|| format!("hue {hue:?} must be a finite number; {FORM}"))?;
    Ok([l / 100.0, c, h])
}

/// Check `designSeeds` (P8). `seeds` is the object itself; `location` names its file.
#[must_use]
pub fn check(seeds: &Value, location: &str) -> Vec<Diagnostic> {
    let at = |suffix: &str| format!("{location}: designSeeds{suffix}");
    let shape = || {
        vec![Diagnostic::error(
            Rule::DesignSeed,
            at(""),
            "designSeeds must be an object with exactly `primary` (3 strings) and `accents` (4 strings)",
        )
        .with_help("retain three primary and four accent references as oklch(L% C H)")]
    };
    let Some(object) = seeds.as_object() else {
        return shape();
    };
    if object.len() != 2 {
        return shape();
    }
    let (Some(primary), Some(accents)) = (
        object.get("primary").and_then(Value::as_array),
        object.get("accents").and_then(Value::as_array),
    ) else {
        return shape();
    };
    if primary.len() != PRIMARY_LEN || accents.len() != ACCENT_LEN {
        return shape();
    }

    let mut out = Vec::new();
    for (group, entries) in [("primary", primary), ("accents", accents)] {
        for (index, entry) in entries.iter().enumerate() {
            let result = entry
                .as_str()
                .ok_or_else(|| format!("{entry} is not a string"))
                .and_then(parse);
            if let Err(message) = result {
                out.push(
                    Diagnostic::error(Rule::DesignSeed, at(&format!(".{group}[{index}]")), message)
                        .with_help("use oklch(L% C H) with L 0..=100, C 0..=1 and a finite hue"),
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use serde_json::json;

    fn rules(d: &[Diagnostic]) -> Vec<Rule> {
        d.iter().map(|d| d.rule).collect()
    }

    fn valid() -> Value {
        json!({
            "primary": ["oklch(32.9478% 0.034021 247.264)", "oklch(20.8843% 0.018341 245.245)", "oklch(34.9975% 0.034292 250.45)"],
            "accents": ["oklch(76.0906% 0.065221 71.267)", "oklch(86.9933% 0.028271 46.055)", "oklch(75.9325% 0.065969 10.27)", "oklch(86.9339% 0.02788 129.841)"]
        })
    }

    #[test]
    fn p8_valid_seeds_pass() {
        assert_eq!(check(&valid(), "f"), Vec::new());
    }

    #[test]
    fn p8_real_seeds_in_base_tokens_pass() {
        let root = env!("CARGO_MANIFEST_DIR");
        let text = std::fs::read_to_string(format!("{root}/tokens/base.tokens.json")).unwrap();
        let doc: Value = serde_json::from_str(&text).unwrap();
        let seeds = &doc["$extensions"]["org.xerrion.fjord"]["designSeeds"];
        assert_eq!(check(seeds, "tokens/base.tokens.json"), Vec::new());
    }

    #[test]
    fn p8_parses_components_with_fraction_lightness() {
        let [l, c, h] = parse("oklch(50% 0.1 -30)").unwrap();
        assert!((l - 0.5).abs() < 1e-12);
        assert!((c - 0.1).abs() < 1e-12);
        assert!((h + 30.0).abs() < 1e-12);
    }

    #[rstest::rstest]
    #[case::missing_percent("oklch(50 0.1 30)")]
    #[case::lightness_over_100("oklch(101% 0.1 30)")]
    #[case::chroma_over_1("oklch(50% 1.5 30)")]
    #[case::negative_chroma("oklch(50% -0.1 30)")]
    #[case::exponent("oklch(50% 1e-1 30)")]
    #[case::plus_sign("oklch(50% +0.1 30)")]
    #[case::infinite_hue("oklch(50% 0.1 inf)")]
    #[case::extra_space_inside("oklch( 50% 0.1 30)")]
    #[case::two_values("oklch(50% 0.1)")]
    #[case::trailing_text("oklch(50% 0.1 30) x")]
    #[case::not_oklch("rgb(1 2 3)")]
    fn p8_malformed_seed_fails(#[case] text: &str) {
        assert!(parse(text).is_err(), "{text} should fail");
        let mut seeds = valid();
        seeds["primary"][1] = json!(text);
        assert_eq!(rules(&check(&seeds, "f")), vec![Rule::DesignSeed]);
    }

    #[test]
    fn p8_hue_that_overflows_to_infinity_is_rejected() {
        let hue = format!("oklch(50% 0.1 {})", "9".repeat(400));
        assert!(parse(&hue).is_err());
    }

    #[test]
    fn p8_shape_violations_fail() {
        let mut short = valid();
        short["accents"].as_array_mut().unwrap().pop();
        assert_eq!(rules(&check(&short, "f")), vec![Rule::DesignSeed]);

        let mut extra = valid();
        extra["extra"] = json!([]);
        assert_eq!(rules(&check(&extra, "f")), vec![Rule::DesignSeed]);

        assert_eq!(rules(&check(&json!([]), "f")), vec![Rule::DesignSeed]);
    }

    #[test]
    fn p8_every_bad_entry_is_reported() {
        let mut seeds = valid();
        seeds["primary"][0] = json!("oklch(200% 0.1 1)");
        seeds["accents"][3] = json!(5);
        let d = check(&seeds, "tokens/base.tokens.json");
        assert_eq!(d.len(), 2);
        assert!(d[0].location.ends_with("designSeeds.primary[0]"));
        assert!(d[1].location.ends_with("designSeeds.accents[3]"));
    }
}
