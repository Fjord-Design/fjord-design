//! Exports of one validated composition (O1–O5): a self-contained DTCG
//! document, a flat resolved JSON, CSS custom properties, and the guarded
//! atomic write that puts them on disk.

mod css;
mod dtcg;
mod resolved;
mod write;

use std::path::PathBuf;

use serde_json::{Map, Value};

use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule};
use crate::policy::contract::{CONTRACT_VERSION, DTCG_VERSION, Mode, SCHEMA_VERSION};
use crate::project::Composition;

pub use css::css;
pub use dtcg::dtcg;
pub use resolved::resolved;
pub use write::write;

/// Output format of `fjord export`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// Self-contained DTCG document with aliases preserved.
    Dtcg,
    /// Flat JSON of resolved values with their sources.
    Resolved,
    /// CSS custom properties.
    Css,
}

impl Format {
    /// File name suffix of the default output (O5).
    #[must_use]
    pub fn suffix(self) -> &'static str {
        match self {
            Format::Dtcg => "dtcg.json",
            Format::Resolved => "resolved.json",
            Format::Css => "tokens.css",
        }
    }
}

/// File name suffix of the default `fjord build` output (O5).
pub const BUILD_SUFFIX: &str = "tokens.json";

/// Root-relative default output path: `dist/fjord.<mode>.<suffix>` (O5, L10).
#[must_use]
pub fn default_output(mode: Mode, suffix: &str) -> PathBuf {
    PathBuf::from("dist").join(format!("fjord.{}.{suffix}", mode.as_str()))
}

/// Render a composition in `format`. JSON is pretty-printed with a final newline.
///
/// # Errors
/// [`Rule::CssName`] when two CSS properties would share a name.
pub fn render(c: &Composition, format: Format) -> Outcome<Vec<u8>> {
    match format {
        Format::Dtcg => json_bytes(&dtcg(c)),
        Format::Resolved => json_bytes(&resolved(c)),
        Format::Css => css(c).map(String::into_bytes),
    }
}

fn json_bytes(value: &Value) -> Outcome<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| {
        Failure::one(Diagnostic::error(
            Rule::Json,
            "export",
            format!("cannot serialize the export: {e}"),
        ))
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// The `org.xerrion.fjord` metadata stamped on every export (O1, O2).
fn metadata(mode: Mode) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("schemaVersion".to_owned(), SCHEMA_VERSION.into());
    map.insert("contractVersion".to_owned(), CONTRACT_VERSION.into());
    map.insert("dtcgVersion".to_owned(), DTCG_VERSION.into());
    map.insert("mode".to_owned(), mode.as_str().into());
    map.insert("colorSpace".to_owned(), "srgb".into());
    map
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use serde_json::json;

    #[test]
    fn o5_default_outputs_carry_the_fjord_prefix() {
        assert_eq!(
            default_output(Mode::Night, BUILD_SUFFIX),
            PathBuf::from("dist/fjord.night.tokens.json")
        );
        assert_eq!(
            default_output(Mode::Day, Format::Dtcg.suffix()),
            PathBuf::from("dist/fjord.day.dtcg.json")
        );
        assert_eq!(
            default_output(Mode::Aurora, Format::Resolved.suffix()),
            PathBuf::from("dist/fjord.aurora.resolved.json")
        );
        assert_eq!(
            default_output(Mode::Night, Format::Css.suffix()),
            PathBuf::from("dist/fjord.night.tokens.css")
        );
    }

    #[test]
    fn o1_metadata_has_the_contract_versions() {
        let m = metadata(Mode::Aurora);
        assert_eq!(
            Value::Object(m),
            json!({"schemaVersion": 2, "contractVersion": "0.5.0", "dtcgVersion": "2025.10", "mode": "aurora", "colorSpace": "srgb"})
        );
    }
}
