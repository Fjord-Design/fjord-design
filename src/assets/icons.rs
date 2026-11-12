//! Icon asset validation: manifest, licence and static SVG structure (A4).

use std::path::Path;

use rayon::prelude::*;
use serde_json::Value;

use super::files::{read_within, sha256_hex};
use crate::diagnostic::{Diagnostic, Outcome, Rule, finish};
use crate::json;

const ASSET_DIR: &str = "assets/icons";
const MANIFEST: &str = "assets/icons/manifest.json";
const NAMES: [&str; 16] = [
    "check",
    "close",
    "info",
    "success",
    "warning",
    "danger",
    "add",
    "remove",
    "search",
    "expand",
    "next",
    "arrow-right",
    "settings",
    "file",
    "copy",
    "loading",
];
const LICENCE_MARKERS: [&str; 2] = ["Permission is hereby granted", "Phosphor Icons"];
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const ROOT_ATTRIBUTES: [(&str, &str); 2] = [("viewBox", "0 0 256 256"), ("fill", "currentColor")];
const PATH_ATTRIBUTES: [&str; 3] = ["d", "fill-rule", "clip-rule"];
const PATH_COMMANDS: &str = "MmZzLlHhVvCcSsQqTtAa";

/// A validated icon set.
#[derive(Clone, Debug, PartialEq)]
pub struct IconReport {
    count: usize,
    manifest: Value,
}

impl IconReport {
    /// Number of icon records in the manifest.
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    /// The parsed `assets/icons/manifest.json`.
    #[must_use]
    pub fn manifest(&self) -> &Value {
        &self.manifest
    }
}

/// Validate the pinned icon set under `root` (A4).
///
/// # Errors
/// Every independent violation, each with a rule and a location.
pub fn check(root: &Path) -> Outcome<IconReport> {
    let manifest = json::read(&root.join(MANIFEST), root)?;
    let mut diags = Vec::new();
    let mut jobs = Vec::new();
    let count = check_manifest(&manifest, &mut diags, &mut jobs);
    let dir = root.join(ASSET_DIR);
    let verified: Vec<Vec<Diagnostic>> = jobs
        .par_iter()
        .map(|job| run_job(job, &dir, root))
        .collect();
    diags.extend(verified.into_iter().flatten());
    finish(IconReport { count, manifest }, diags)
}

fn fail(location: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(Rule::IconAsset, location, message)
}

fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

/// A file whose bytes must be read and verified; run in parallel.
struct Job {
    location: String,
    file: String,
    sha256: String,
    kind: Kind,
}

enum Kind {
    Licence,
    Svg,
}

/// Check the manifest and collect file jobs. Returns the number of icon records.
fn check_manifest(manifest: &Value, diags: &mut Vec<Diagnostic>, jobs: &mut Vec<Job>) -> usize {
    let Some(object) = manifest.as_object() else {
        diags.push(fail(MANIFEST, "expected a JSON object"));
        return 0;
    };
    let shape_ok = object.get("schemaVersion").and_then(Value::as_u64) == Some(1)
        && str_field(manifest, "family") == Some("Phosphor")
        && str_field(manifest, "weight") == Some("fill")
        && str_field(manifest, "license") == Some("MIT");
    if !shape_ok {
        diags.push(fail(
            MANIFEST,
            "expect schemaVersion 1, family \"Phosphor\", weight \"fill\" and licence \"MIT\"",
        ));
    }
    match (
        str_field(manifest, "licenseFile"),
        str_field(manifest, "licenseSha256"),
    ) {
        (Some(file), Some(sha256)) => jobs.push(Job {
            location: MANIFEST.to_owned(),
            file: file.to_owned(),
            sha256: sha256.to_owned(),
            kind: Kind::Licence,
        }),
        _ => diags.push(fail(
            MANIFEST,
            "licenseFile and licenseSha256 must be strings",
        )),
    }
    let Some(icons) = object.get("icons").and_then(Value::as_array) else {
        diags.push(fail(MANIFEST, "icons must be an array"));
        return 0;
    };
    let names: Vec<Option<&str>> = icons.iter().map(|icon| str_field(icon, "name")).collect();
    let exact = icons.len() == NAMES.len()
        && NAMES
            .iter()
            .all(|name| names.iter().filter(|n| **n == Some(*name)).count() == 1);
    if !exact {
        diags.push(
            fail(MANIFEST, "icons must list exactly the 16 approved names, each once")
                .with_help("check close info success warning danger add remove search expand next arrow-right settings file copy loading"),
        );
    }
    for (i, icon) in icons.iter().enumerate() {
        let location = format!("{MANIFEST}: #/icons/{i}");
        let (Some(name), Some(file), Some(sha256)) =
            (names[i], str_field(icon, "file"), str_field(icon, "sha256"))
        else {
            diags.push(fail(&location, "name, file and sha256 must be strings"));
            continue;
        };
        if file != format!("{name}.svg") {
            diags.push(fail(&location, format!("{name}: file must be {name}.svg")));
            continue;
        }
        jobs.push(Job {
            location,
            file: file.to_owned(),
            sha256: sha256.to_owned(),
            kind: Kind::Svg,
        });
    }
    icons.len()
}

fn run_job(job: &Job, dir: &Path, root: &Path) -> Vec<Diagnostic> {
    let bytes = match read_within(dir, &job.file, root, &job.location, Rule::IconAsset) {
        Ok(bytes) => bytes,
        Err(diagnostic) => return vec![diagnostic],
    };
    let mut out = Vec::new();
    if sha256_hex(&bytes) != job.sha256 {
        out.push(fail(
            &job.location,
            format!("{}: hash differs from the pinned source; inspect the file and refresh its pinned hash", job.file),
        ));
    }
    match job.kind {
        Kind::Licence => {
            let text = std::str::from_utf8(&bytes).unwrap_or_default();
            if !LICENCE_MARKERS.iter().all(|marker| text.contains(marker)) {
                out.push(fail(
                    &job.location,
                    format!("{}: original MIT licence text is missing", job.file),
                ));
            }
        }
        Kind::Svg => out.extend(check_svg(&bytes, &job.file, &job.location)),
    }
    out
}

fn path_char(c: char) -> bool {
    c.is_ascii_digit() || c.is_ascii_whitespace() || ".,+-".contains(c) || PATH_COMMANDS.contains(c)
}

/// Structure of one icon: SVG root, exact attributes, and static filled paths only.
fn check_svg(bytes: &[u8], file: &str, location: &str) -> Vec<Diagnostic> {
    let bad = |message: &str| fail(location, format!("{file}: {message}"));
    let Ok(text) = std::str::from_utf8(bytes) else {
        return vec![bad("not UTF-8")];
    };
    let options = roxmltree::ParsingOptions {
        allow_dtd: false,
        ..roxmltree::ParsingOptions::default()
    };
    let document = match roxmltree::Document::parse_with_options(text, options) {
        Ok(document) => document,
        Err(error) => return vec![bad(&format!("invalid SVG (DTDs are not allowed): {error}"))],
    };
    let svg = document.root_element();
    let mut out = Vec::new();
    let tag = svg.tag_name();
    if tag.namespace() != Some(SVG_NAMESPACE) || tag.name() != "svg" {
        out.push(bad("root must be an SVG svg element"));
    }
    let attributes_exact = svg.attributes().count() == ROOT_ATTRIBUTES.len()
        && ROOT_ATTRIBUTES
            .iter()
            .all(|(name, value)| svg.attribute(*name) == Some(*value));
    if !attributes_exact {
        out.push(bad(
            "root must have exactly viewBox=\"0 0 256 256\" and fill=\"currentColor\"",
        ));
    }
    let mut paths = 0_usize;
    for child in svg.children().filter(roxmltree::Node::is_element) {
        let name = child.tag_name();
        let is_path = name.namespace() == Some(SVG_NAMESPACE) && name.name() == "path";
        let childless = !child.children().any(|n| n.is_element());
        let attributes_ok = child
            .attributes()
            .all(|a| a.namespace().is_none() && PATH_ATTRIBUTES.contains(&a.name()));
        if !is_path || !childless || !attributes_ok {
            out.push(bad(&format!(
                "only static filled SVG path elements are permitted; found <{}>",
                name.name()
            )));
            continue;
        }
        paths += 1;
        let data_ok = child
            .attribute("d")
            .is_some_and(|d| !d.is_empty() && d.chars().all(path_char));
        if !data_ok {
            out.push(bad(
                "path data may use only path command letters, digits, . , + - and whitespace",
            ));
        }
    }
    if paths == 0 {
        out.push(bad("supply at least one filled path"));
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use std::fs;
    use std::path::Path;

    use serde_json::json;

    use super::*;
    use crate::assets::files::sha256_hex;
    use crate::assets::files::testing::{Fixture, distinct_rules};

    const MANIFEST_REL: &str = "assets/icons/manifest.json";

    fn rule_set(f: &Fixture) -> Vec<Rule> {
        distinct_rules(&check(&f.root))
    }

    /// Re-pin one icon after its SVG was changed on purpose.
    fn refresh_icon(f: &Fixture, name: &str) {
        let bytes = fs::read(f.path(&format!("assets/icons/{name}.svg"))).unwrap();
        let mut manifest = f.read_json(MANIFEST_REL);
        for icon in manifest["icons"].as_array_mut().unwrap() {
            if icon["name"] == name {
                icon["sha256"] = json!(sha256_hex(&bytes));
            }
        }
        f.write_json(MANIFEST_REL, &manifest);
    }

    fn edit_svg(f: &Fixture, name: &str, edit: impl FnOnce(String) -> String) {
        let path = f.path(&format!("assets/icons/{name}.svg"));
        let text = fs::read_to_string(&path).unwrap();
        fs::write(path, edit(text)).unwrap();
    }

    #[test]
    fn a4_real_icons_pass_with_sixteen_entries() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = check(root).expect("shipped icons must pass");
        assert_eq!(report.count(), 16);
        assert_eq!(report.manifest()["family"], "Phosphor");
    }

    #[test]
    fn a4_tampered_svg_bytes_fail() {
        let f = Fixture::new();
        edit_svg(&f, "check", |text| format!("{text}\n"));
        assert_eq!(rule_set(&f), [Rule::IconAsset]);
    }

    #[test]
    fn a4_svg_script_with_refreshed_hash_fails() {
        let f = Fixture::new();
        edit_svg(&f, "check", |text| {
            text.replace("</svg>", "<script>alert(1)</script></svg>")
        });
        refresh_icon(&f, "check");
        assert_eq!(rule_set(&f), [Rule::IconAsset]);
    }

    #[test]
    fn a4_svg_dtd_with_refreshed_hash_fails() {
        let f = Fixture::new();
        edit_svg(&f, "check", |text| {
            format!("<!DOCTYPE svg [<!ENTITY e \"x\">]>\n{text}")
        });
        refresh_icon(&f, "check");
        assert_eq!(rule_set(&f), [Rule::IconAsset]);
    }

    #[test]
    fn a4_root_attribute_drift_with_refreshed_hash_fails() {
        let f = Fixture::new();
        edit_svg(&f, "check", |text| text.replace("0 0 256 256", "0 0 24 24"));
        refresh_icon(&f, "check");
        assert_eq!(rule_set(&f), [Rule::IconAsset]);
    }

    #[test]
    fn a4_path_with_disallowed_attribute_fails() {
        let f = Fixture::new();
        edit_svg(&f, "check", |text| {
            text.replace("<path ", "<path stroke=\"red\" ")
        });
        refresh_icon(&f, "check");
        assert_eq!(rule_set(&f), [Rule::IconAsset]);
    }

    #[test]
    fn l4_replaced_licence_with_refreshed_hash_fails() {
        let f = Fixture::new();
        let text = "Replacement notice.\n";
        fs::write(f.path("assets/icons/LICENSE"), text).unwrap();
        let mut manifest = f.read_json(MANIFEST_REL);
        manifest["licenseSha256"] = json!(sha256_hex(text.as_bytes()));
        f.write_json(MANIFEST_REL, &manifest);
        assert_eq!(rule_set(&f), [Rule::IconAsset]);
    }

    #[test]
    fn a4_deleted_licence_is_io() {
        let f = Fixture::new();
        fs::remove_file(f.path("assets/icons/LICENSE")).unwrap();
        assert_eq!(rule_set(&f), [Rule::Io]);
    }

    #[test]
    fn a4_deleted_icon_is_io() {
        let f = Fixture::new();
        fs::remove_file(f.path("assets/icons/copy.svg")).unwrap();
        assert_eq!(rule_set(&f), [Rule::Io]);
    }

    #[test]
    fn a4_missing_icon_name_fails() {
        let f = Fixture::new();
        let mut manifest = f.read_json(MANIFEST_REL);
        manifest["icons"]
            .as_array_mut()
            .unwrap()
            .retain(|icon| icon["name"] != "loading");
        f.write_json(MANIFEST_REL, &manifest);
        assert_eq!(rule_set(&f), [Rule::IconAsset]);
    }

    #[test]
    fn a4_manifest_array_shape_fails() {
        let f = Fixture::new();
        fs::write(f.path(MANIFEST_REL), "[]").unwrap();
        assert_eq!(rule_set(&f), [Rule::IconAsset]);
    }
}
