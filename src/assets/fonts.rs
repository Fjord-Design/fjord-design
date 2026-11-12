//! Font asset validation: manifest shape, licences, WOFF2 files and inspection evidence (A1, A2).

use std::path::Path;

use rayon::prelude::*;
use serde_json::Value;

use super::files::{read_within, sha256_hex};
use crate::diagnostic::{Diagnostic, Outcome, Rule, finish};
use crate::json::{self, same};

const ASSET_DIR: &str = "assets/fonts";
const MANIFEST: &str = "assets/fonts/manifest.json";
const INSPECTION: &str = "assets/fonts/inspection.json";
const LICENCE_MARKERS: [&str; 2] = ["SIL OPEN FONT LICENSE", "Version 1.1"];
const PROBE_LETTERS: &str = "æøåÆØÅéÉ";
const WOFF2_MIN_LEN: usize = 48;

/// The approved families and the styles each must provide (L5).
const APPROVED: [(&str, &[&str]); 5] = [
    ("Noto Sans", &["normal", "italic"]),
    ("Raleway", &["normal", "italic"]),
    ("Open Sans", &["normal", "italic"]),
    ("Cascadia Code", &["normal", "italic"]),
    ("Fira Code", &["normal"]),
];

/// One bundled font face, as declared by the manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFace {
    pub family: String,
    pub style: String,
    /// Path relative to `assets/fonts/`.
    pub file: String,
    pub min: u16,
    pub max: u16,
}

/// A validated font set.
#[derive(Clone, Debug)]
pub struct FontReport {
    faces: Vec<FontFace>,
    manifest: Value,
}

impl FontReport {
    #[must_use]
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    #[must_use]
    pub fn faces(&self) -> &[FontFace] {
        &self.faces
    }

    /// The parsed `assets/fonts/manifest.json`.
    #[must_use]
    pub fn manifest(&self) -> &Value {
        &self.manifest
    }

    /// Whether a bundled face of `family` and `style` covers `weight`.
    #[must_use]
    pub fn supports(&self, family: &str, style: &str, weight: f64) -> bool {
        self.faces.iter().any(|f| {
            f.family == family
                && f.style == style
                && f64::from(f.min) <= weight
                && weight <= f64::from(f.max)
        })
    }
}

/// Validate the pinned font set under `root` (A1, A2).
///
/// # Errors
/// Every independent violation, each with a rule and a location.
pub fn check(root: &Path) -> Outcome<FontReport> {
    let manifest = json::read(&root.join(MANIFEST), root)?;
    let mut collected = Collected::default();
    if check_manifest(&manifest, &mut collected) {
        match json::read(&root.join(INSPECTION), root) {
            Ok(inspection) => {
                check_inspection(&inspection, &collected.declared, &mut collected.diags);
            }
            Err(failure) => collected.diags.extend(failure.0),
        }
    }
    let dir = root.join(ASSET_DIR);
    let verified: Vec<Vec<Diagnostic>> = collected
        .jobs
        .par_iter()
        .map(|job| run_job(job, &dir, root))
        .collect();
    collected.diags.extend(verified.into_iter().flatten());
    let Collected { diags, faces, .. } = collected;
    finish(FontReport { faces, manifest }, diags)
}

#[derive(Default)]
struct Collected {
    diags: Vec<Diagnostic>,
    jobs: Vec<Job>,
    faces: Vec<FontFace>,
    declared: Vec<Declared>,
}

/// A manifest face, kept to cross-check the inspection evidence.
struct Declared {
    file: String,
    family: String,
    style: String,
    weight: Value,
    sha256: String,
}

/// A file whose bytes must be read and verified; run in parallel.
struct Job {
    location: String,
    file: String,
    sha256: String,
    check: Check,
}

enum Check {
    Licence,
    Woff2 { bytes: u64 },
}

fn fail(location: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(Rule::FontAsset, location, message)
}

fn family_location(family: usize) -> String {
    format!("{MANIFEST}: #/families/{family}")
}

fn face_location(family: usize, face: usize) -> String {
    format!("{MANIFEST}: #/families/{family}/faces/{face}")
}

fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn approved_styles(family: &str) -> Option<&'static [&'static str]> {
    APPROVED
        .iter()
        .find(|(name, _)| *name == family)
        .map(|(_, styles)| *styles)
}

/// An ordered `[min, max]` CSS weight range with 1 ≤ min ≤ max ≤ 1000.
fn weight_range(value: &Value) -> Option<(u16, u16)> {
    let [min, max] = value.as_array()?.as_slice() else {
        return None;
    };
    let min = u16::try_from(min.as_u64()?).ok()?;
    let max = u16::try_from(max.as_u64()?).ok()?;
    ((1..=1000).contains(&min) && min <= max && max <= 1000).then_some((min, max))
}

/// Check the manifest structure and collect its files. Returns `false` when the
/// manifest is too malformed to cross-check the inspection evidence.
fn check_manifest(manifest: &Value, c: &mut Collected) -> bool {
    let Some(object) = manifest.as_object() else {
        c.diags.push(fail(MANIFEST, "expected a JSON object"));
        return false;
    };
    if object.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        c.diags
            .push(fail(MANIFEST, "schemaVersion must be the integer 1"));
    }
    if object.get("fontDisplay").and_then(Value::as_str) != Some("swap") {
        c.diags.push(fail(MANIFEST, "fontDisplay must be \"swap\""));
    }
    let Some(families) = object.get("families").and_then(Value::as_array) else {
        c.diags.push(fail(MANIFEST, "families must be an array"));
        return false;
    };
    let names: Vec<Option<&str>> = families.iter().map(|f| str_field(f, "family")).collect();
    let exact = families.len() == APPROVED.len()
        && APPROVED
            .iter()
            .all(|(name, _)| names.iter().filter(|n| **n == Some(*name)).count() == 1);
    if !exact {
        c.diags.push(
            fail(
                MANIFEST,
                "families must be exactly the five approved families, each once",
            )
            .with_help("Noto Sans, Raleway, Open Sans, Cascadia Code and Fira Code"),
        );
    }
    for (i, family) in families.iter().enumerate() {
        let Some(expected) = names[i].and_then(approved_styles) else {
            c.diags
                .push(fail(&family_location(i), "unknown font family"));
            continue;
        };
        check_family(i, family, names[i].unwrap_or_default(), expected, c);
    }
    true
}

fn check_family(i: usize, family: &Value, name: &str, expected: &[&str], c: &mut Collected) {
    let location = family_location(i);
    if str_field(family, "license") != Some("OFL-1.1") {
        c.diags.push(fail(
            &location,
            format!("{name}: license must be \"OFL-1.1\""),
        ));
    }
    match (
        str_field(family, "licenseFile"),
        str_field(family, "licenseSha256"),
    ) {
        (Some(file), Some(sha256)) => c.jobs.push(Job {
            location: location.clone(),
            file: file.to_owned(),
            sha256: sha256.to_owned(),
            check: Check::Licence,
        }),
        _ => c.diags.push(fail(
            &location,
            format!("{name}: licenseFile and licenseSha256 must be strings"),
        )),
    }
    let Some(faces) = family.get("faces").and_then(Value::as_array) else {
        c.diags
            .push(fail(&location, format!("{name}: faces must be an array")));
        return;
    };
    let mut styles: Vec<&str> = Vec::new();
    for (j, face) in faces.iter().enumerate() {
        check_face(i, j, face, name, expected, &mut styles, c);
    }
    if styles.len() != expected.len() || expected.iter().any(|s| !styles.contains(s)) {
        c.diags.push(fail(
            &location,
            format!("{name}: styles must be exactly {}", expected.join(" and ")),
        ));
    }
}

fn check_face<'a>(
    i: usize,
    j: usize,
    face: &'a Value,
    name: &str,
    expected: &[&str],
    styles: &mut Vec<&'a str>,
    c: &mut Collected,
) {
    let location = face_location(i, j);
    let (Some(file), Some(style), Some(weight), Some(sha256), Some(bytes)) = (
        str_field(face, "file"),
        str_field(face, "style"),
        face.get("weight"),
        str_field(face, "sha256"),
        face.get("bytes").and_then(Value::as_u64),
    ) else {
        c.diags.push(fail(
            &location,
            "file, style, weight, sha256 and bytes are required",
        ));
        return;
    };
    if Path::new(file).extension().is_none_or(|e| e != "woff2") {
        c.diags.push(fail(
            &location,
            format!("{file}: file must be a .woff2 inside assets/fonts"),
        ));
    }
    if c.declared.iter().any(|d| d.file == file) {
        c.diags.push(fail(
            &location,
            format!("{file}: file is listed more than once"),
        ));
    }
    if !expected.contains(&style) {
        c.diags.push(fail(
            &location,
            format!("{file}: style {style:?} is not approved for {name}"),
        ));
    } else if styles.contains(&style) {
        c.diags.push(fail(
            &location,
            format!("{file}: style {style:?} is listed more than once in {name}"),
        ));
    }
    styles.push(style);
    match weight_range(weight) {
        Some((min, max)) => c.faces.push(FontFace {
            family: name.to_owned(),
            style: style.to_owned(),
            file: file.to_owned(),
            min,
            max,
        }),
        None => c.diags.push(fail(
            &location,
            format!("{file}: weight must be an ordered [min, max] pair of integers from 1 to 1000"),
        )),
    }
    c.declared.push(Declared {
        file: file.to_owned(),
        family: name.to_owned(),
        style: style.to_owned(),
        weight: weight.clone(),
        sha256: sha256.to_owned(),
    });
    c.jobs.push(Job {
        location,
        file: file.to_owned(),
        sha256: sha256.to_owned(),
        check: Check::Woff2 { bytes },
    });
}

fn check_inspection(inspection: &Value, declared: &[Declared], diags: &mut Vec<Diagnostic>) {
    let probe_ok = str_field(inspection, "probe").is_some_and(|probe| {
        (0x20_u8..=0x7E).all(|b| probe.contains(char::from(b)))
            && PROBE_LETTERS.chars().all(|c| probe.contains(c))
    });
    if !probe_ok {
        diags.push(fail(
            INSPECTION,
            "probe must include printable ASCII 0x20–0x7E and æøåÆØÅéÉ",
        ));
    }
    let Some(faces) = inspection.get("faces").and_then(Value::as_array) else {
        diags.push(fail(INSPECTION, "faces must be an array"));
        return;
    };
    let mut seen: Vec<&str> = Vec::new();
    for face in faces {
        let Some(file) = str_field(face, "file") else {
            diags.push(fail(INSPECTION, "each inspected face needs a file"));
            continue;
        };
        if seen.contains(&file) {
            diags.push(fail(
                INSPECTION,
                format!("{file}: evidence is listed more than once"),
            ));
            continue;
        }
        seen.push(file);
        let Some(d) = declared.iter().find(|d| d.file == file) else {
            diags.push(fail(
                INSPECTION,
                format!("{file}: evidence for a file the manifest does not declare"),
            ));
            continue;
        };
        let agrees = str_field(face, "sha256") == Some(d.sha256.as_str())
            && str_field(face, "family") == Some(d.family.as_str())
            && str_field(face, "style") == Some(d.style.as_str())
            && face.get("weight").is_some_and(|w| same(w, &d.weight))
            && face.get("variable") == Some(&Value::Bool(true))
            && face
                .get("missingProbeCodepoints")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty);
        if !agrees {
            diags.push(fail(
                INSPECTION,
                format!("{file}: family, style, weight, sha256 or glyph evidence differs from the manifest; rerun scripts/inspect_fonts.py"),
            ));
        }
    }
    for d in declared {
        if !seen.contains(&d.file.as_str()) {
            diags.push(fail(
                INSPECTION,
                format!("{}: no inspection evidence", d.file),
            ));
        }
    }
}

fn run_job(job: &Job, dir: &Path, root: &Path) -> Vec<Diagnostic> {
    let bytes = match read_within(dir, &job.file, root, &job.location, Rule::FontAsset) {
        Ok(bytes) => bytes,
        Err(diagnostic) => return vec![diagnostic],
    };
    let mut out = Vec::new();
    if sha256_hex(&bytes) != job.sha256 {
        out.push(fail(
            &job.location,
            format!(
                "{}: hash differs from the manifest; inspect the file and refresh its pinned hash",
                job.file
            ),
        ));
    }
    match job.check {
        Check::Licence => {
            let text = std::str::from_utf8(&bytes).unwrap_or_default();
            if !LICENCE_MARKERS.iter().all(|marker| text.contains(marker)) {
                out.push(fail(
                    &job.location,
                    format!("{}: original OFL-1.1 licence text is missing", job.file),
                ));
            }
        }
        Check::Woff2 { bytes: expected } => {
            if u64::try_from(bytes.len()).unwrap_or(u64::MAX) != expected {
                out.push(fail(
                    &job.location,
                    format!("{}: byte size differs from the manifest", job.file),
                ));
            }
            if !woff2_header_ok(&bytes) {
                out.push(fail(
                    &job.location,
                    format!(
                        "{}: not a WOFF2 file (signature or declared length invalid)",
                        job.file
                    ),
                ));
            }
        }
    }
    out
}

fn woff2_header_ok(bytes: &[u8]) -> bool {
    if bytes.len() < WOFF2_MIN_LEN || !bytes.starts_with(b"wOF2") {
        return false;
    }
    let declared = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    u64::from(declared) == u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use std::fs;
    use std::path::Path;

    use serde_json::{Value, json};

    use super::*;
    use crate::assets::files::sha256_hex;
    use crate::assets::files::testing::{Fixture, distinct_rules, rules};

    const MANIFEST_REL: &str = "assets/fonts/manifest.json";
    const INSPECTION_REL: &str = "assets/fonts/inspection.json";
    const NOTO: &str = "noto-sans/NotoSans-latin.woff2";
    const NOTO_ITALIC: &str = "noto-sans/NotoSansItalic-latin.woff2";

    fn rule_set(f: &Fixture) -> Vec<Rule> {
        distinct_rules(&check(&f.root))
    }

    fn for_each_face(manifest: &mut Value, mut f: impl FnMut(&mut Value)) {
        for family in manifest["families"].as_array_mut().unwrap() {
            for face in family["faces"].as_array_mut().unwrap() {
                f(face);
            }
        }
    }

    /// Re-pin `file` after its bytes were changed on purpose: manifest and evidence hashes.
    fn refresh(f: &Fixture, file: &str) {
        let bytes = fs::read(f.path(&format!("assets/fonts/{file}"))).unwrap();
        let sha = sha256_hex(&bytes);
        let mut manifest = f.read_json(MANIFEST_REL);
        for_each_face(&mut manifest, |face| {
            if face["file"] == file {
                face["sha256"] = json!(sha);
                face["bytes"] = json!(bytes.len());
            }
        });
        f.write_json(MANIFEST_REL, &manifest);
        let mut inspection = f.read_json(INSPECTION_REL);
        for face in inspection["faces"].as_array_mut().unwrap() {
            if face["file"] == file {
                face["sha256"] = json!(sha);
            }
        }
        f.write_json(INSPECTION_REL, &inspection);
    }

    fn set_licence_sha(f: &Fixture, family_index: usize, sha: &str) {
        let mut manifest = f.read_json(MANIFEST_REL);
        manifest["families"][family_index]["licenseSha256"] = json!(sha);
        f.write_json(MANIFEST_REL, &manifest);
    }

    #[test]
    fn a1_a2_real_assets_pass_with_nine_faces() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = check(root).expect("shipped fonts must pass");
        assert_eq!(report.face_count(), 9);
        assert!(report.supports("Noto Sans", "normal", 400.0));
        assert!(report.supports("Raleway", "italic", 700.0));
        assert!(!report.supports("Open Sans", "normal", 850.0));
        assert!(!report.supports("Fira Code", "italic", 400.0));
    }

    #[test]
    fn a1_tampered_font_bytes_fail() {
        let f = Fixture::new();
        let path = f.path(&format!("assets/fonts/{NOTO}"));
        let mut bytes = fs::read(&path).unwrap();
        let middle = bytes.len() / 2;
        bytes[middle] ^= 0x01;
        fs::write(&path, bytes).unwrap();
        assert_eq!(rule_set(&f), [Rule::FontAsset]);
    }

    #[test]
    fn l4_replaced_licence_with_refreshed_hash_fails() {
        let f = Fixture::new();
        let text = "A replacement notice without the required terms.\n";
        fs::write(f.path("assets/fonts/noto-sans/OFL.txt"), text).unwrap();
        set_licence_sha(&f, 0, &sha256_hex(text.as_bytes()));
        assert_eq!(rule_set(&f), [Rule::FontAsset]);
    }

    #[test]
    fn a1_deleted_licence_is_io() {
        let f = Fixture::new();
        fs::remove_file(f.path("assets/fonts/noto-sans/OFL.txt")).unwrap();
        assert_eq!(rule_set(&f), [Rule::Io]);
    }

    #[test]
    fn a2_inspection_drift_fails() {
        let f = Fixture::new();
        let mut inspection = f.read_json(INSPECTION_REL);
        inspection["faces"][0]["sha256"] = json!("0".repeat(64));
        f.write_json(INSPECTION_REL, &inspection);
        assert_eq!(rule_set(&f), [Rule::FontAsset]);
    }

    #[test]
    fn a2_short_probe_fails() {
        let f = Fixture::new();
        let mut inspection = f.read_json(INSPECTION_REL);
        let probe = inspection["probe"].as_str().unwrap().replace('é', "");
        inspection["probe"] = json!(probe);
        f.write_json(INSPECTION_REL, &inspection);
        assert_eq!(rule_set(&f), [Rule::FontAsset]);
    }

    #[test]
    fn a1_broken_woff2_header_with_refreshed_hash_fails() {
        let f = Fixture::new();
        let path = f.path(&format!("assets/fonts/{NOTO}"));
        let mut bytes = fs::read(&path).unwrap();
        bytes[0] = b'x';
        fs::write(&path, bytes).unwrap();
        refresh(&f, NOTO);
        assert_eq!(rule_set(&f), [Rule::FontAsset]);
    }

    #[test]
    fn l5_missing_italic_face_fails() {
        let f = Fixture::new();
        let mut manifest = f.read_json(MANIFEST_REL);
        let faces = manifest["families"][0]["faces"].as_array_mut().unwrap();
        faces.retain(|face| face["file"] != NOTO_ITALIC);
        f.write_json(MANIFEST_REL, &manifest);
        assert_eq!(rule_set(&f), [Rule::FontAsset]);
    }

    #[test]
    fn a1_weight_range_outside_css_limits_fails() {
        let f = Fixture::new();
        let mut manifest = f.read_json(MANIFEST_REL);
        manifest["families"][0]["faces"][0]["weight"] = json!([100, 1001]);
        f.write_json(MANIFEST_REL, &manifest);
        assert_eq!(rule_set(&f), [Rule::FontAsset]);
    }

    #[test]
    fn a1_dot_dot_escape_fails() {
        let f = Fixture::new();
        fs::copy(
            f.path(&format!("assets/fonts/{NOTO}")),
            f.path("assets/escape.woff2"),
        )
        .unwrap();
        let mut manifest = f.read_json(MANIFEST_REL);
        manifest["families"][0]["faces"][0]["file"] = json!("../escape.woff2");
        f.write_json(MANIFEST_REL, &manifest);
        assert_eq!(rule_set(&f), [Rule::FontAsset]);
    }

    #[cfg(unix)]
    #[test]
    fn a1_symlink_escape_fails() {
        let f = Fixture::new();
        let link = f.path(&format!("assets/fonts/{NOTO}"));
        let outside = f.path("outside.woff2");
        fs::copy(&link, &outside).unwrap();
        fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        assert_eq!(rule_set(&f), [Rule::FontAsset]);
    }

    #[test]
    fn a1_manifest_array_shape_fails() {
        let f = Fixture::new();
        fs::write(f.path(MANIFEST_REL), "[]").unwrap();
        let failure = check(&f.root).unwrap_err();
        assert_eq!(rules(&failure), [Rule::FontAsset]);
    }

    #[test]
    fn a1_malformed_manifest_json_is_json_rule() {
        let f = Fixture::new();
        fs::write(f.path(MANIFEST_REL), "{").unwrap();
        assert_eq!(rule_set(&f), [Rule::Json]);
    }

    #[test]
    fn woff2_header_requires_magic_and_declared_length() {
        let mut good = b"wOF2".to_vec();
        good.extend_from_slice(&[0, 0, 0, 0]);
        good.extend_from_slice(&48_u32.to_be_bytes());
        good.resize(48, 0);
        assert!(woff2_header_ok(&good));
        let mut wrong_length = good.clone();
        wrong_length[11] = 49;
        assert!(!woff2_header_ok(&wrong_length));
        assert!(!woff2_header_ok(&good[..47]));
    }
}
