//! Shared helpers for the integration test crates.
//!
//! Each test crate uses a subset of these helpers.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

use fjord::diagnostic::{Failure, Rule};
use serde_json::{Map, Value, json};
use tempfile::TempDir;

/// The repository root.
#[must_use]
pub fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A temporary copy of selected repository directories.
pub struct Fixture {
    _dir: TempDir,
    root: PathBuf,
}

impl Fixture {
    /// Copies the named repository directories (e.g. `["tokens", "assets"]`)
    /// into a fresh temporary directory.
    #[must_use]
    pub fn project(dirs: &[&str]) -> Self {
        let dir = TempDir::new().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        let repo = repo();
        for name in dirs {
            copy_dir(&repo.join(name), &root.join(name));
        }
        Self { _dir: dir, root }
    }

    /// Canonical root of the copy (macOS `/var` resolves to `/private/var`).
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `root().join(rel)`.
    #[must_use]
    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }
}

/// Recursively copies `src` into `dst`, creating `dst`.
pub fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let target = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// The layer metadata block every token file carries.
#[must_use]
pub fn layer_metadata() -> Value {
    json!({
        "org.xerrion.fjord": {
            "schemaVersion": 2,
            "contractVersion": "0.5.0",
            "dtcgVersion": "2025.10"
        }
    })
}

/// Builds an IN2-valid layer document from dotted token paths.
///
/// Each token object gets a `$description` when it has none.
#[must_use]
pub fn layer_document(tokens: &[(&str, Value)]) -> Value {
    let mut doc = Map::new();
    doc.insert("$extensions".into(), layer_metadata());
    let mut doc = Value::Object(doc);
    for (path, token) in tokens {
        let mut token = token.clone();
        if let Value::Object(map) = &mut token
            && !map.contains_key("$description")
        {
            map.insert("$description".into(), json!(format!("Test token {path}.")));
        }
        let mut node = &mut doc;
        let segments: Vec<&str> = path.split('.').collect();
        let (last, groups) = segments.split_last().unwrap();
        for segment in groups {
            node = node
                .as_object_mut()
                .unwrap()
                .entry(*segment)
                .or_insert_with(|| Value::Object(Map::new()));
        }
        node.as_object_mut()
            .unwrap()
            .insert((*last).to_owned(), token);
    }
    doc
}

/// Writes [`layer_document`] to `root.join(name)` and returns the path.
pub fn layer(root: &Path, name: &str, tokens: &[(&str, Value)]) -> PathBuf {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let text = serde_json::to_string_pretty(&layer_document(tokens)).unwrap();
    fs::write(&path, text).unwrap();
    path
}

/// The rule of every diagnostic in a failure, in order.
#[must_use]
pub fn rules(failure: &Failure) -> Vec<Rule> {
    failure.0.iter().map(|d| d.rule).collect()
}

/// The `fjord` binary under test.
#[must_use]
pub fn fjord() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("fjord").unwrap()
}
