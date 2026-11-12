//! Pinned font and icon assets: sources, licences, hashes and structure (A1–A4).

mod files;
mod fonts;
mod icons;

pub use fonts::{FontFace, FontReport, check as check_fonts};
pub use icons::{IconReport, check as check_icons};
