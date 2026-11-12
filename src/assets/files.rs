//! File access shared by the asset checks: contained reads and hashing.

use std::fs;
use std::path::{Component, Path};

use sha2::{Digest, Sha256};

use crate::diagnostic::{Diagnostic, Rule};
use crate::json::display_path;

/// Lower-case hexadecimal SHA-256 of `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Read `name` from `dir`, refusing any path that leaves `dir`.
///
/// Lexical escapes (`..`, absolute paths) and symlinks whose target lies outside `dir`
/// are reported under `rule`. A missing or unreadable file is `Rule::Io`.
///
/// # Errors
/// Returns the diagnostic describing why the file cannot be used.
pub fn read_within(
    dir: &Path,
    name: &str,
    root: &Path,
    location: &str,
    rule: Rule,
) -> Result<Vec<u8>, Diagnostic> {
    let dir_label = display_path(dir, root);
    let lexical_ok = !name.is_empty()
        && Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_)));
    if !lexical_ok {
        return Err(Diagnostic::error(
            rule,
            location,
            format!("{name:?} must be a relative path inside {dir_label}"),
        )
        .with_help("reference files inside the asset directory only"));
    }
    let canonical_dir = fs::canonicalize(dir).map_err(|e| {
        Diagnostic::error(Rule::Io, location, format!("cannot open {dir_label}: {e}"))
    })?;
    let canonical = fs::canonicalize(dir.join(name))
        .map_err(|e| Diagnostic::error(Rule::Io, location, format!("cannot read {name}: {e}")))?;
    if !canonical.starts_with(&canonical_dir) {
        return Err(Diagnostic::error(
            rule,
            location,
            format!("{name} resolves outside {dir_label} (through a symlink)"),
        )
        .with_help("replace the link with the file itself inside the asset directory"));
    }
    fs::read(&canonical)
        .map_err(|e| Diagnostic::error(Rule::Io, location, format!("cannot read {name}: {e}")))
}

#[cfg(test)]
pub mod testing {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::Value;
    use tempfile::TempDir;

    use crate::diagnostic::{Failure, Outcome, Rule};

    /// A temporary project root holding a copy of the repository's `assets/`.
    pub struct Fixture {
        _dir: TempDir,
        pub root: PathBuf,
    }

    impl Fixture {
        pub fn new() -> Self {
            let dir = TempDir::new().unwrap();
            let root = fs::canonicalize(dir.path()).unwrap();
            let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
            copy_tree(&source, &root.join("assets"));
            Self { _dir: dir, root }
        }

        pub fn path(&self, rel: &str) -> PathBuf {
            self.root.join(rel)
        }

        pub fn read_json(&self, rel: &str) -> Value {
            serde_json::from_str(&fs::read_to_string(self.path(rel)).unwrap()).unwrap()
        }

        pub fn write_json(&self, rel: &str, value: &Value) {
            let mut text = serde_json::to_string_pretty(value).unwrap();
            text.push('\n');
            fs::write(self.path(rel), text).unwrap();
        }
    }

    pub fn copy_tree(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let target = dst.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    pub fn rules(failure: &Failure) -> Vec<Rule> {
        failure.0.iter().map(|d| d.rule).collect()
    }

    /// Distinct rules reported by a failed outcome, in first-seen order; empty on success.
    pub fn distinct_rules<T>(outcome: &Outcome<T>) -> Vec<Rule> {
        let mut out = Vec::new();
        if let Err(failure) = outcome {
            for rule in rules(failure) {
                if !out.contains(&rule) {
                    out.push(rule);
                }
            }
        }
        out
    }
}
