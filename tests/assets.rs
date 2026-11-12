//! Integration tests for the pinned font and icon assets (A1–A4, L4, L5).
//!
//! Every test works on a temporary copy of `tokens/` and `assets/`. Cases
//! already covered by the unit tests in `src/assets` are not repeated; the
//! cases here vary the file, the field or the failure mode, and exercise the
//! A3 bindings through real compositions.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs;

use common::{Fixture, layer, rules};
use fjord::assets::{FontReport, IconReport, check_fonts, check_icons};
use fjord::diagnostic::{Outcome, Rule};
use fjord::policy::contract::Mode;
use fjord::project::{Composition, Project, Selection, check_asset_bindings};
use rstest::rstest;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const FONT_MANIFEST: &str = "assets/fonts/manifest.json";
const INSPECTION: &str = "assets/fonts/inspection.json";
const ICON_MANIFEST: &str = "assets/icons/manifest.json";
const ICON_LICENCE: &str = "assets/icons/LICENSE";
const BASE: &str = "tokens/base.tokens.json";
const EXTENSION: &str = "org.xerrion.fjord";

// ---------------------------------------------------------------------------
// Helpers

fn fixture() -> Fixture {
    Fixture::project(&["tokens", "assets"])
}

fn read_json(f: &Fixture, rel: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(f.path(rel)).unwrap()).unwrap()
}

fn write_json(f: &Fixture, rel: &str, value: &Value) {
    let mut text = serde_json::to_string_pretty(value).unwrap();
    text.push('\n');
    fs::write(f.path(rel), text).unwrap();
}

fn edit_json(f: &Fixture, rel: &str, edit: impl FnOnce(&mut Value)) {
    let mut value = read_json(f, rel);
    edit(&mut value);
    write_json(f, rel, &value);
}

fn edit_text(f: &Fixture, rel: &str, edit: impl FnOnce(String) -> String) {
    let path = f.path(rel);
    let text = fs::read_to_string(&path).unwrap();
    fs::write(path, edit(text)).unwrap();
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn font_path(file: &str) -> String {
    format!("assets/fonts/{file}")
}

fn fonts_ok(f: &Fixture) -> FontReport {
    check_fonts(f.root()).unwrap_or_else(|failure| panic!("fonts must pass:\n{failure}"))
}

fn icons_ok(f: &Fixture) -> IconReport {
    check_icons(f.root()).unwrap_or_else(|failure| panic!("icons must pass:\n{failure}"))
}

/// The rules of a failed outcome; panics when it succeeded.
fn failed<T>(outcome: Outcome<T>) -> Vec<Rule> {
    match outcome {
        Ok(_) => panic!("expected a failure"),
        Err(failure) => rules(&failure),
    }
}

/// At least one diagnostic, and every one under `rule`.
fn assert_only(found: &[Rule], rule: Rule) {
    assert!(!found.is_empty(), "expected {rule:?}");
    assert!(
        found.iter().all(|r| *r == rule),
        "expected only {rule:?}: {found:?}"
    );
}

/// Re-pin a face whose bytes were changed on purpose: manifest `sha256` and
/// `bytes`, and the inspection `sha256`, so only the other checks can fail.
fn refresh_face(f: &Fixture, file: &str) {
    let bytes = fs::read(f.path(&font_path(file))).unwrap();
    let sha = sha256(&bytes);
    edit_json(f, FONT_MANIFEST, |manifest| {
        for family in manifest["families"].as_array_mut().unwrap() {
            for face in family["faces"].as_array_mut().unwrap() {
                if face["file"] == file {
                    face["sha256"] = json!(sha);
                    face["bytes"] = json!(bytes.len());
                }
            }
        }
    });
    edit_json(f, INSPECTION, |inspection| {
        for face in inspection["faces"].as_array_mut().unwrap() {
            if face["file"] == file {
                face["sha256"] = json!(sha);
            }
        }
    });
}

/// Re-pin the licence of family `index` after its text changed on purpose.
fn refresh_font_licence(f: &Fixture, index: usize) {
    edit_json(f, FONT_MANIFEST, |manifest| {
        let family = &mut manifest["families"][index];
        let file = family["licenseFile"].as_str().unwrap().to_owned();
        let bytes = fs::read(f.path(&font_path(&file))).unwrap();
        family["licenseSha256"] = json!(sha256(&bytes));
    });
}

/// Rename a face's `file` in both the manifest and the inspection evidence.
fn rename_face(f: &Fixture, old: &str, new: &str) {
    edit_json(f, FONT_MANIFEST, |manifest| {
        for family in manifest["families"].as_array_mut().unwrap() {
            for face in family["faces"].as_array_mut().unwrap() {
                if face["file"] == old {
                    face["file"] = json!(new);
                }
            }
        }
    });
    edit_json(f, INSPECTION, |inspection| {
        for face in inspection["faces"].as_array_mut().unwrap() {
            if face["file"] == old {
                face["file"] = json!(new);
            }
        }
    });
}

fn refresh_icon(f: &Fixture, name: &str) {
    let bytes = fs::read(f.path(&format!("assets/icons/{name}.svg"))).unwrap();
    edit_json(f, ICON_MANIFEST, |manifest| {
        for icon in manifest["icons"].as_array_mut().unwrap() {
            if icon["name"] == name {
                icon["sha256"] = json!(sha256(&bytes));
            }
        }
    });
}

fn refresh_icon_licence(f: &Fixture) {
    let bytes = fs::read(f.path(ICON_LICENCE)).unwrap();
    edit_json(f, ICON_MANIFEST, |manifest| {
        manifest["licenseSha256"] = json!(sha256(&bytes));
    });
}

fn family_index(name: &str) -> usize {
    [
        "Noto Sans",
        "Raleway",
        "Open Sans",
        "Cascadia Code",
        "Fira Code",
    ]
    .iter()
    .position(|n| *n == name)
    .unwrap()
}

fn compose(project: &Project, selection: &Selection) -> Composition {
    project
        .compose(selection)
        .unwrap_or_else(|failure| panic!("composition must pass:\n{failure}"))
}

/// A3 on the default night composition of the fixture, against pristine assets.
fn binding_outcome(f: &Fixture) -> Outcome<()> {
    let project = Project::open(f.root()).unwrap();
    let c = compose(&project, &project.default_selection(Mode::Night));
    check_asset_bindings(&c, &fonts_ok(f), &icons_ok(f))
}

// ---------------------------------------------------------------------------
// Pristine copy

#[test]
fn a1_a4_pristine_copy_passes_with_nine_faces_and_sixteen_icons() {
    let f = fixture();
    let fonts = fonts_ok(&f);
    assert_eq!(fonts.face_count(), 9);
    assert_eq!(fonts.manifest(), &read_json(&f, FONT_MANIFEST));
    let icons = icons_ok(&f);
    assert_eq!(icons.count(), 16);
    assert_eq!(icons.manifest(), &read_json(&f, ICON_MANIFEST));
}

#[test]
fn l5_pristine_faces_are_the_baseline_families_and_styles() {
    let f = fixture();
    let mut faces: Vec<(String, String)> = fonts_ok(&f)
        .faces()
        .iter()
        .map(|face| (face.family.clone(), face.style.clone()))
        .collect();
    faces.sort();
    let mut expected: Vec<(String, String)> = [
        ("Cascadia Code", "italic"),
        ("Cascadia Code", "normal"),
        ("Fira Code", "normal"),
        ("Noto Sans", "italic"),
        ("Noto Sans", "normal"),
        ("Open Sans", "italic"),
        ("Open Sans", "normal"),
        ("Raleway", "italic"),
        ("Raleway", "normal"),
    ]
    .iter()
    .map(|(family, style)| ((*family).to_owned(), (*style).to_owned()))
    .collect();
    expected.sort();
    assert_eq!(faces, expected);
}

#[test]
fn a3_pristine_bindings_pass_in_every_mode() {
    let f = fixture();
    let project = Project::open(f.root()).unwrap();
    let (fonts, icons) = (fonts_ok(&f), icons_ok(&f));
    for mode in Mode::ALL {
        let c = compose(&project, &project.default_selection(mode));
        check_asset_bindings(&c, &fonts, &icons)
            .unwrap_or_else(|failure| panic!("{mode:?}:\n{failure}"));
    }
}

// ---------------------------------------------------------------------------
// A1 fonts: bytes, licences and paths

#[rstest]
#[case::fira_code("fira-code/FiraCode-VF.woff2")]
#[case::cascadia_italic("cascadia-code/CascadiaCodeItalic.woff2")]
#[case::raleway("raleway/Raleway-latin.woff2")]
fn a1_tampered_font_bytes_fail(#[case] file: &str) {
    let f = fixture();
    let path = f.path(&font_path(file));
    let mut bytes = fs::read(&path).unwrap();
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0xff;
    fs::write(&path, bytes).unwrap();
    // Same length and header: only the pinned hash catches it.
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn a1_appended_font_byte_fails_hash_size_and_header_independently() {
    let f = fixture();
    let path = f.path(&font_path("open-sans/OpenSans-latin.woff2"));
    let mut bytes = fs::read(&path).unwrap();
    bytes.push(0);
    fs::write(&path, bytes).unwrap();
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset; 3]);
}

#[test]
fn a1_tampered_licence_bytes_fail() {
    let f = fixture();
    // The markers stay; only the pinned licence hash catches the change.
    edit_text(&f, &font_path("raleway/OFL.txt"), |text| {
        format!("{text}\n")
    });
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn a1_deleted_licence_is_io() {
    let f = fixture();
    fs::remove_file(f.path(&font_path("fira-code/OFL.txt"))).unwrap();
    assert_eq!(failed(check_fonts(f.root())), [Rule::Io]);
}

#[test]
fn a1_deleted_face_is_io() {
    let f = fixture();
    fs::remove_file(f.path(&font_path("cascadia-code/CascadiaCode.woff2"))).unwrap();
    assert_eq!(failed(check_fonts(f.root())), [Rule::Io]);
}

/// Break the WOFF2 header, then re-pin hash and size so only the header check
/// remains to catch it (L4).
#[rstest]
#[case::woff1_magic(|b: &mut Vec<u8>| b[..4].copy_from_slice(b"wOFF"))]
#[case::declared_length_too_long(|b: &mut Vec<u8>| {
    let wrong = u32::try_from(b.len() + 1).unwrap();
    b[8..12].copy_from_slice(&wrong.to_be_bytes());
})]
#[case::declared_length_too_short(|b: &mut Vec<u8>| {
    let wrong = u32::try_from(b.len() - 1).unwrap();
    b[8..12].copy_from_slice(&wrong.to_be_bytes());
})]
#[case::truncated(|b: &mut Vec<u8>| b.truncate(100))]
#[case::shorter_than_48_with_consistent_length(|b: &mut Vec<u8>| {
    b.truncate(47);
    b[8..12].copy_from_slice(&47_u32.to_be_bytes());
})]
fn l4_broken_woff2_header_with_refreshed_hash_fails(#[case] breaks: fn(&mut Vec<u8>)) {
    let f = fixture();
    let file = "raleway/RalewayItalic-latin.woff2";
    let path = f.path(&font_path(file));
    let mut bytes = fs::read(&path).unwrap();
    breaks(&mut bytes);
    fs::write(&path, bytes).unwrap();
    refresh_face(&f, file);
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn l4_refreshed_face_with_valid_header_passes() {
    // Control for the header cases: re-pinning a file whose header is intact
    // and whose bytes are unchanged leaves the set valid.
    let f = fixture();
    refresh_face(&f, "raleway/RalewayItalic-latin.woff2");
    assert_eq!(fonts_ok(&f).face_count(), 9);
}

#[rstest]
#[case::missing_version("Copyright notice.\nSIL OPEN FONT LICENSE\n", false)]
#[case::missing_title("Copyright notice.\nVersion 1.1 - 26 February 2007\n", false)]
#[case::other_licence("Apache License\nVersion 2.0, January 2004\n", false)]
#[case::both_markers("Reworded preamble.\nSIL OPEN FONT LICENSE\nVersion 1.1\n", true)]
fn l4_replaced_font_licence_with_refreshed_hash(#[case] text: &str, #[case] passes: bool) {
    let f = fixture();
    let index = family_index("Open Sans");
    fs::write(f.path(&font_path("open-sans/OFL.txt")), text).unwrap();
    refresh_font_licence(&f, index);
    let outcome = check_fonts(f.root());
    if passes {
        assert_eq!(outcome.unwrap().face_count(), 9);
    } else {
        assert_eq!(failed(outcome), [Rule::FontAsset]);
    }
}

#[rstest]
#[case::raleway("Raleway", "raleway/RalewayItalic-latin.woff2")]
#[case::open_sans("Open Sans", "open-sans/OpenSansItalic-latin.woff2")]
#[case::cascadia_code("Cascadia Code", "cascadia-code/CascadiaCodeItalic.woff2")]
fn l5_missing_italic_face_fails(#[case] family: &str, #[case] file: &str) {
    let f = fixture();
    let index = family_index(family);
    edit_json(&f, FONT_MANIFEST, |manifest| {
        manifest["families"][index]["faces"]
            .as_array_mut()
            .unwrap()
            .retain(|face| face["file"] != file);
    });
    edit_json(&f, INSPECTION, |inspection| {
        inspection["faces"]
            .as_array_mut()
            .unwrap()
            .retain(|face| face["file"] != file);
    });
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn l5_fira_code_italic_is_not_approved() {
    let f = fixture();
    let extra = "fira-code/FiraCode-Italic.woff2";
    fs::copy(
        f.path(&font_path("fira-code/FiraCode-VF.woff2")),
        f.path(&font_path(extra)),
    )
    .unwrap();
    let index = family_index("Fira Code");
    edit_json(&f, FONT_MANIFEST, |manifest| {
        let faces = manifest["families"][index]["faces"].as_array_mut().unwrap();
        let mut face = faces[0].clone();
        face["file"] = json!(extra);
        face["style"] = json!("italic");
        faces.push(face);
    });
    edit_json(&f, INSPECTION, |inspection| {
        let faces = inspection["faces"].as_array_mut().unwrap();
        let mut face = faces
            .iter()
            .find(|face| face["file"] == "fira-code/FiraCode-VF.woff2")
            .unwrap()
            .clone();
        face["file"] = json!(extra);
        face["style"] = json!("italic");
        faces.push(face);
    });
    assert_only(&failed(check_fonts(f.root())), Rule::FontAsset);
}

#[test]
fn l5_missing_family_fails() {
    let f = fixture();
    edit_json(&f, FONT_MANIFEST, |manifest| {
        manifest["families"]
            .as_array_mut()
            .unwrap()
            .retain(|family| family["family"] != "Fira Code");
    });
    edit_json(&f, INSPECTION, |inspection| {
        inspection["faces"]
            .as_array_mut()
            .unwrap()
            .retain(|face| face["family"] != "Fira Code");
    });
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn l5_unknown_family_fails() {
    let f = fixture();
    let index = family_index("Fira Code");
    edit_json(&f, FONT_MANIFEST, |manifest| {
        manifest["families"][index]["family"] = json!("Fira Sans");
    });
    assert_only(&failed(check_fonts(f.root())), Rule::FontAsset);
}

#[test]
fn a1_licence_escape_through_dot_dot_fails() {
    let f = fixture();
    // The escaped file exists and has the pinned bytes: only containment fails.
    fs::copy(
        f.path(&font_path("noto-sans/OFL.txt")),
        f.path("assets/OFL.txt"),
    )
    .unwrap();
    edit_json(&f, FONT_MANIFEST, |manifest| {
        manifest["families"][family_index("Noto Sans")]["licenseFile"] = json!("../OFL.txt");
    });
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn a1_face_with_absolute_path_fails() {
    let f = fixture();
    let file = "noto-sans/NotoSans-latin.woff2";
    let absolute = f.path(&font_path(file)).to_string_lossy().into_owned();
    rename_face(&f, file, &absolute);
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn a1_face_escape_through_dot_dot_fails_alone() {
    let f = fixture();
    let file = "open-sans/OpenSans-latin.woff2";
    fs::copy(f.path(&font_path(file)), f.path("assets/escape.woff2")).unwrap();
    rename_face(&f, file, "../escape.woff2");
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[cfg(unix)]
#[test]
fn a1_licence_symlink_outside_fonts_fails() {
    let f = fixture();
    let licence = f.path(&font_path("raleway/OFL.txt"));
    let outside = f.path("tokens/OFL.txt");
    fs::rename(&licence, &outside).unwrap();
    std::os::unix::fs::symlink(&outside, &licence).unwrap();
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[cfg(unix)]
#[test]
fn a1_symlink_that_stays_inside_fonts_passes() {
    let f = fixture();
    let licence = f.path(&font_path("raleway/OFL.txt"));
    let shared = f.path(&font_path("shared-OFL.txt"));
    fs::rename(&licence, &shared).unwrap();
    std::os::unix::fs::symlink("../shared-OFL.txt", &licence).unwrap();
    assert_eq!(fonts_ok(&f).face_count(), 9);
}

#[rstest]
#[case::schema_version_two(|m: &mut Value| m["schemaVersion"] = json!(2))]
#[case::schema_version_string(|m: &mut Value| m["schemaVersion"] = json!("1"))]
#[case::font_display_block(|m: &mut Value| m["fontDisplay"] = json!("block"))]
#[case::font_display_missing(|m: &mut Value| {
    m.as_object_mut().unwrap().remove("fontDisplay");
})]
#[case::family_licence_mit(|m: &mut Value| m["families"][2]["license"] = json!("MIT"))]
#[case::family_licence_missing(|m: &mut Value| {
    m["families"][4].as_object_mut().unwrap().remove("license");
})]
#[case::licence_hash_drift(|m: &mut Value| m["families"][1]["licenseSha256"] = json!("0".repeat(64)))]
#[case::byte_count_drift(|m: &mut Value| m["families"][0]["faces"][1]["bytes"] = json!(1))]
#[case::not_woff2(|m: &mut Value| m["families"][0]["faces"][0]["file"] = json!("noto-sans/OFL.txt"))]
fn a1_manifest_field_violations_fail(#[case] edit: fn(&mut Value)) {
    let f = fixture();
    edit_json(&f, FONT_MANIFEST, edit);
    assert_only(&failed(check_fonts(f.root())), Rule::FontAsset);
}

#[rstest]
#[case::min_zero(json!([0, 900]))]
#[case::reversed(json!([900, 100]))]
#[case::fractional(json!([100.5, 900]))]
#[case::single(json!([400]))]
fn a1_face_weight_range_violations_fail(#[case] weight: Value) {
    let f = fixture();
    // Keep the inspection evidence in step so only the manifest range fails.
    edit_json(&f, FONT_MANIFEST, |m| {
        m["families"][0]["faces"][0]["weight"] = weight.clone();
    });
    edit_json(&f, INSPECTION, |i| i["faces"][0]["weight"] = weight.clone());
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn a1_duplicate_face_file_fails() {
    let f = fixture();
    let index = family_index("Noto Sans");
    edit_json(&f, FONT_MANIFEST, |manifest| {
        let faces = manifest["families"][index]["faces"].as_array_mut().unwrap();
        let normal = faces[0].clone();
        for key in ["file", "sha256", "bytes"] {
            faces[1][key] = normal[key].clone();
        }
    });
    assert_only(&failed(check_fonts(f.root())), Rule::FontAsset);
}

#[test]
fn a1_duplicate_style_within_family_fails() {
    let f = fixture();
    let index = family_index("Cascadia Code");
    let italic = "cascadia-code/CascadiaCodeItalic.woff2";
    edit_json(&f, FONT_MANIFEST, |manifest| {
        manifest["families"][index]["faces"][1]["style"] = json!("normal");
    });
    edit_json(&f, INSPECTION, |inspection| {
        for face in inspection["faces"].as_array_mut().unwrap() {
            if face["file"] == italic {
                face["style"] = json!("normal");
            }
        }
    });
    assert_only(&failed(check_fonts(f.root())), Rule::FontAsset);
}

// ---------------------------------------------------------------------------
// A2 inspection evidence

#[rstest]
#[case::sha256(8, "sha256", json!("f".repeat(64)))]
#[case::family(1, "family", json!("Noto Serif"))]
#[case::style(2, "style", json!("italic"))]
#[case::weight(3, "weight", json!([100, 800]))]
#[case::not_variable(4, "variable", json!(false))]
#[case::variable_missing(5, "variable", Value::Null)]
#[case::missing_probe_codepoints(6, "missingProbeCodepoints", json!([230]))]
fn a2_inspection_drift_fails(#[case] face: usize, #[case] key: &str, #[case] value: Value) {
    let f = fixture();
    edit_json(&f, INSPECTION, |inspection| {
        let face = inspection["faces"][face].as_object_mut().unwrap();
        if value.is_null() {
            face.remove(key);
        } else {
            face.insert(key.to_owned(), value);
        }
    });
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn a2_inspection_weight_is_compared_numerically() {
    let f = fixture();
    edit_json(&f, INSPECTION, |inspection| {
        inspection["faces"][0]["weight"] = json!([100.0, 900.0]);
    });
    assert_eq!(fonts_ok(&f).face_count(), 9);
}

#[rstest]
#[case::missing_evidence(|faces: &mut Vec<Value>| {
    faces.remove(3);
})]
#[case::undeclared_evidence(|faces: &mut Vec<Value>| {
    let mut extra = faces[0].clone();
    extra["file"] = json!("noto-sans/Extra.woff2");
    faces.push(extra);
})]
#[case::duplicate_evidence(|faces: &mut Vec<Value>| {
    let copy = faces[0].clone();
    faces.push(copy);
})]
fn a2_inspection_must_cover_exactly_the_manifest_files(#[case] edit: fn(&mut Vec<Value>)) {
    let f = fixture();
    edit_json(&f, INSPECTION, |inspection| {
        edit(inspection["faces"].as_array_mut().unwrap());
    });
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[rstest]
#[case::space(' ')]
#[case::tilde('~')]
#[case::capital_a('A')]
#[case::digit('7')]
#[case::ae('æ')]
#[case::capital_aring('Å')]
#[case::capital_e_acute('É')]
fn a2_short_probe_fails(#[case] missing: char) {
    let f = fixture();
    edit_json(&f, INSPECTION, |inspection| {
        let probe = inspection["probe"].as_str().unwrap().replace(missing, "");
        inspection["probe"] = json!(probe);
    });
    assert_eq!(failed(check_fonts(f.root())), [Rule::FontAsset]);
}

#[test]
fn a2_probe_superset_passes() {
    let f = fixture();
    edit_json(&f, INSPECTION, |inspection| {
        let probe = format!("{}ßÜ", inspection["probe"].as_str().unwrap());
        inspection["probe"] = json!(probe);
    });
    assert_eq!(fonts_ok(&f).face_count(), 9);
}

// ---------------------------------------------------------------------------
// A3 bindings between tokens and assets

#[rstest]
#[case::font_manifest_elsewhere("fontAssets", Some(json!("assets/fonts/other.json")), Rule::FontAsset)]
#[case::font_manifest_missing("fontAssets", None, Rule::FontAsset)]
#[case::font_manifest_not_a_string("fontAssets", Some(json!(["assets/fonts/manifest.json"])), Rule::FontAsset)]
#[case::icon_manifest_elsewhere("iconAssets", Some(json!("assets/fonts/manifest.json")), Rule::IconAsset)]
#[case::icon_manifest_missing("iconAssets", None, Rule::IconAsset)]
fn a3_base_asset_pointer_must_name_the_manifest(
    #[case] key: &str,
    #[case] value: Option<Value>,
    #[case] rule: Rule,
) {
    let f = fixture();
    edit_json(&f, BASE, |base| {
        let metadata = base["$extensions"][EXTENSION].as_object_mut().unwrap();
        match value {
            Some(v) => {
                metadata.insert(key.to_owned(), v);
            }
            None => {
                metadata.remove(key);
            }
        }
    });
    assert_eq!(failed(binding_outcome(&f)), [rule]);
}

/// `semantic.typography.code` starts with Cascadia Code (200–700);
/// `semantic.typography.body` starts with Noto Sans (100–900).
#[rstest]
#[case::code_800("code", 800, None, false)]
#[case::code_150("code", 150, None, false)]
#[case::code_700_upper_bound("code", 700, None, true)]
#[case::code_200_lower_bound("code", 200, None, true)]
#[case::body_950("body", 950, None, false)]
#[case::body_900("body", 900, None, true)]
#[case::body_italic("body", 400, Some("italic"), true)]
#[case::code_italic_oblique("code-italic", 400, Some("oblique"), false)]
#[case::code_italic_800("code-italic", 800, Some("italic"), false)]
fn a3_typography_role_needs_a_bundled_face(
    #[case] role: &str,
    #[case] weight: u16,
    #[case] style: Option<&str>,
    #[case] passes: bool,
) {
    let f = fixture();
    edit_json(&f, BASE, |base| {
        let token = &mut base["semantic"]["typography"][role];
        token["$value"]["fontWeight"] = json!(weight);
        if let Some(style) = style {
            token["$extensions"][EXTENSION]["fontStyle"] = json!(style);
        }
    });
    let outcome = binding_outcome(&f);
    if passes {
        outcome.unwrap_or_else(|failure| panic!("{failure}"));
    } else {
        assert_eq!(failed(outcome), [Rule::FontAsset]);
    }
}

#[test]
fn a3_font_style_must_be_a_string() {
    let f = fixture();
    edit_json(&f, BASE, |base| {
        base["semantic"]["typography"]["body"]["$extensions"][EXTENSION]["fontStyle"] = json!(1);
    });
    assert_eq!(failed(binding_outcome(&f)), [Rule::FontAsset]);
}

#[test]
fn a3_unsupported_weight_from_an_override_fails() {
    let f = fixture();
    let mut code = read_json(&f, BASE)["semantic"]["typography"]["code"].clone();
    code["$value"]["fontWeight"] = json!(800);
    let file = layer(
        f.root(),
        "heavy-code.tokens.json",
        &[("semantic.typography.code", code)],
    );
    let project = Project::open(f.root()).unwrap();
    let selection = Selection {
        mode: Mode::Day,
        theme: None,
        overrides: vec![file],
    };
    let c = compose(&project, &selection);
    let outcome = check_asset_bindings(&c, &fonts_ok(&f), &icons_ok(&f));
    assert_eq!(failed(outcome), [Rule::FontAsset]);
}

#[test]
fn a3_manifest_pointer_follows_a_renamed_base_file() {
    let f = fixture();
    fs::rename(f.path(BASE), f.path("tokens/core.tokens.json")).unwrap();
    edit_json(&f, "tokens/contract.json", |contract| {
        contract["base"] = json!("core.tokens.json");
    });
    // The renamed base still carries the pointers: the bindings pass.
    binding_outcome(&f).unwrap_or_else(|failure| panic!("{failure}"));

    // And the check reads the renamed file, not a hard-coded name.
    edit_json(&f, "tokens/core.tokens.json", |base| {
        base["$extensions"][EXTENSION]
            .as_object_mut()
            .unwrap()
            .remove("fontAssets");
    });
    let failure = binding_outcome(&f).unwrap_err();
    assert_eq!(rules(&failure), [Rule::FontAsset]);
    assert!(
        failure.0[0].location.starts_with("tokens/core.tokens.json"),
        "{}",
        failure.0[0].location
    );
}

// ---------------------------------------------------------------------------
// A4 icons

#[test]
fn a4_tampered_svg_bytes_fail() {
    let f = fixture();
    // Still a valid icon, so only the pinned hash catches the edit.
    edit_text(&f, "assets/icons/close.svg", |text| {
        text.replacen('M', "M ", 1)
    });
    assert_eq!(failed(check_icons(f.root())), [Rule::IconAsset]);
}

#[test]
fn a4_tampered_licence_bytes_fail() {
    let f = fixture();
    edit_text(&f, ICON_LICENCE, |text| format!("{text}\n"));
    assert_eq!(failed(check_icons(f.root())), [Rule::IconAsset]);
}

#[cfg(unix)]
#[test]
fn a4_svg_symlink_outside_icons_fails() {
    let f = fixture();
    let icon = f.path("assets/icons/search.svg");
    let outside = f.path("assets/search.svg");
    fs::rename(&icon, &outside).unwrap();
    std::os::unix::fs::symlink(&outside, &icon).unwrap();
    assert_eq!(failed(check_icons(f.root())), [Rule::IconAsset]);
}

const SVG_OPEN: &str =
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor">"#;

fn svg(body: &str) -> String {
    format!("{SVG_OPEN}{body}</svg>")
}

/// Replace `info.svg` with `text`, re-pin it, and check the set.
fn icon_outcome(text: &str) -> Outcome<IconReport> {
    let f = fixture();
    fs::write(f.path("assets/icons/info.svg"), text).unwrap();
    refresh_icon(&f, "info");
    check_icons(f.root())
}

#[rstest]
#[case::script_sibling(svg(r#"<path d="M0 0L1 1Z"/><script>alert(1)</script>"#))]
#[case::script_inside_path(svg(r#"<path d="M0 0L1 1Z"><script>alert(1)</script></path>"#))]
#[case::external_dtd(format!(
    "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">\n{}",
    svg(r#"<path d="M0 0L1 1Z"/>"#)
))]
#[case::circle_child(svg(r#"<path d="M0 0L1 1Z"/><circle r="4"/>"#))]
#[case::group_wrapper(svg(r#"<g><path d="M0 0L1 1Z"/></g>"#))]
#[case::animated_path(svg(r#"<path d="M0 0L1 1Z"><animate attributeName="d"/></path>"#))]
#[case::path_event_handler(svg(r#"<path d="M0 0L1 1Z" onclick="x()"/>"#))]
#[case::path_href(svg(
    r##"<path xmlns:xlink="http://www.w3.org/1999/xlink" d="M0 0L1 1Z" xlink:href="#x"/>"##
))]
#[case::foreign_path(svg(r#"<path xmlns="http://example.com/other" d="M0 0L1 1Z"/>"#))]
#[case::no_path(svg(""))]
#[case::empty_d(svg(r#"<path d=""/>"#))]
#[case::missing_d(svg(r#"<path fill-rule="evenodd"/>"#))]
#[case::d_url(svg(r#"<path d="M0 0 url(#x)"/>"#))]
#[case::d_exponent(svg(r#"<path d="M1e2 0L1 1Z"/>"#))]
#[case::d_semicolon(svg(r#"<path d="M0 0;L1 1Z"/>"#))]
#[case::root_view_box(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M0 0L1 1Z"/></svg>"#.to_owned())]
#[case::root_fill_missing(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><path d="M0 0L1 1Z"/></svg>"#.to_owned())]
#[case::root_fill_red(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="red"><path d="M0 0L1 1Z"/></svg>"#.to_owned())]
#[case::root_extra_width(r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" viewBox="0 0 256 256" fill="currentColor"><path d="M0 0L1 1Z"/></svg>"#.to_owned())]
#[case::root_onload(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" onload="x()"><path d="M0 0L1 1Z"/></svg>"#.to_owned())]
#[case::root_without_namespace(r#"<svg viewBox="0 0 256 256" fill="currentColor"><path d="M0 0L1 1Z"/></svg>"#.to_owned())]
#[case::root_not_svg(r#"<g xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor"><path d="M0 0L1 1Z"/></g>"#.to_owned())]
#[case::not_xml("<svg".to_owned())]
fn a4_svg_structure_violation_with_refreshed_hash_fails(#[case] text: String) {
    assert_only(&failed(icon_outcome(&text)), Rule::IconAsset);
}

#[rstest]
#[case::plain_path(svg(r#"<path d="M0 0L1 1Z"/>"#))]
#[case::fill_and_clip_rules(svg(
    r#"<path d="M0,0 L10.5,-2 l+3 4 Z" fill-rule="evenodd" clip-rule="nonzero"/>"#
))]
#[case::every_command_letter(svg(r#"<path d="M0 0 m1 1 L2 2 l1 1 H3 h1 V4 v1 C1 2 3 4 5 6 c1 2 3 4 5 6 S1 2 3 4 s1 2 3 4 Q1 2 3 4 q1 2 3 4 T1 2 t1 2 A1 1 0 0 1 2 2 a1 1 0 0 1 2 2 Z z"/>"#))]
#[case::two_paths_and_comment(svg("<!-- glyph --><path d=\"M0 0Z\"/>\n<path d=\"M1 1Z\"/>"))]
fn a4_static_filled_paths_with_refreshed_hash_pass(#[case] text: String) {
    assert_eq!(icon_outcome(&text).unwrap().count(), 16);
}

#[rstest]
#[case::missing_name(|icons: &mut Vec<Value>| icons.retain(|i| i["name"] != "arrow-right"))]
#[case::extra_name(|icons: &mut Vec<Value>| {
    let mut extra = icons[0].clone();
    extra["name"] = json!("extra");
    extra["file"] = json!("extra.svg");
    icons.push(extra);
})]
#[case::duplicate_name(|icons: &mut Vec<Value>| {
    let copy = icons[0].clone();
    icons.push(copy);
})]
#[case::renamed_entry(|icons: &mut Vec<Value>| {
    icons[0]["name"] = json!("tick");
    icons[0]["file"] = json!("tick.svg");
})]
#[case::file_not_named_after_icon(|icons: &mut Vec<Value>| {
    let close = icons.iter().find(|i| i["name"] == "close").unwrap().clone();
    icons[0]["file"] = close["file"].clone();
    icons[0]["sha256"] = close["sha256"].clone();
})]
#[case::file_escape(|icons: &mut Vec<Value>| icons[0]["file"] = json!("../icons/check.svg"))]
#[case::hash_not_a_string(|icons: &mut Vec<Value>| icons[1]["sha256"] = json!(0))]
fn a4_icon_list_violations_fail(#[case] edit: fn(&mut Vec<Value>)) {
    let f = fixture();
    // `extra`/`tick` exist on disk with pinned bytes, so only the list check can fail.
    for name in ["extra", "tick"] {
        fs::copy(
            f.path("assets/icons/check.svg"),
            f.path(&format!("assets/icons/{name}.svg")),
        )
        .unwrap();
    }
    edit_json(&f, ICON_MANIFEST, |manifest| {
        edit(manifest["icons"].as_array_mut().unwrap());
    });
    assert_only(&failed(check_icons(f.root())), Rule::IconAsset);
}

#[rstest]
#[case::schema_version(|m: &mut Value| m["schemaVersion"] = json!(2))]
#[case::family(|m: &mut Value| m["family"] = json!("Lucide"))]
#[case::weight(|m: &mut Value| m["weight"] = json!("regular"))]
#[case::licence(|m: &mut Value| m["license"] = json!("ISC"))]
#[case::licence_file_missing(|m: &mut Value| {
    m.as_object_mut().unwrap().remove("licenseFile");
})]
#[case::licence_escape(|m: &mut Value| m["licenseFile"] = json!("../fonts/noto-sans/OFL.txt"))]
fn a4_manifest_field_violations_fail(#[case] edit: fn(&mut Value)) {
    let f = fixture();
    edit_json(&f, ICON_MANIFEST, edit);
    assert_eq!(failed(check_icons(f.root())), [Rule::IconAsset]);
}

#[rstest]
#[case::missing_phosphor(
    "MIT License\n\nPermission is hereby granted, free of charge.\n",
    false
)]
#[case::missing_permission("MIT License\n\nCopyright (c) 2023 Phosphor Icons\n", false)]
#[case::other_licence("ISC License\n\nCopyright (c) Lucide Contributors\n", false)]
#[case::both_markers(
    "Copyright Phosphor Icons.\nPermission is hereby granted to all.\n",
    true
)]
fn l4_replaced_icon_licence_with_refreshed_hash(#[case] text: &str, #[case] passes: bool) {
    let f = fixture();
    fs::write(f.path(ICON_LICENCE), text).unwrap();
    refresh_icon_licence(&f);
    let outcome = check_icons(f.root());
    if passes {
        assert_eq!(outcome.unwrap().count(), 16);
    } else {
        assert_eq!(failed(outcome), [Rule::IconAsset]);
    }
}
