//! Guarded atomic output (O4).

use std::ffi::OsString;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Component, Path, PathBuf};

use tempfile::NamedTempFile;

use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule};
use crate::json::display_path;

/// Symbolic links followed while resolving one path, as in Linux `MAXSYMLINKS`.
const MAX_LINKS: usize = 40;

/// Write `bytes` to `output` atomically: a failure keeps the old file.
///
/// The target is resolved through every symbolic link, including dangling
/// ones and the output itself. Inside the project root it must lie under
/// `<root>/dist/`, and it may never be one of `layer_files`.
///
/// # Errors
/// [`Rule::OutputProtected`] for a refused target, [`Rule::Io`] when the
/// path cannot be resolved or the file cannot be written.
pub fn write(root: &Path, output: &Path, bytes: &[u8], layer_files: &[PathBuf]) -> Outcome<()> {
    let location = display_path(output, root);
    let io_error = |what: &str, e: io::Error| {
        Failure::one(Diagnostic::error(
            Rule::Io,
            &location,
            format!("{what}: {e}"),
        ))
    };
    let target = resolve(output).map_err(|e| io_error("cannot resolve the output path", e))?;
    let root = resolve(root).map_err(|e| io_error("cannot resolve the project root", e))?;
    let mut layers = Vec::with_capacity(layer_files.len());
    for file in layer_files {
        layers.push(resolve(file).map_err(|e| io_error("cannot resolve a layer file", e))?);
    }
    if let Some(d) = protection(&target, &root, &layers, &location) {
        return Err(Failure::one(d));
    }
    persist(&target, bytes).map_err(|e| io_error("cannot write the output", e))
}

/// Why writing `target` is refused, if it is.
fn protection(
    target: &Path,
    root: &Path,
    layers: &[PathBuf],
    location: &str,
) -> Option<Diagnostic> {
    let resolved = display_path(target, root);
    if layers.iter().any(|layer| same_file(target, layer)) {
        return Some(
            Diagnostic::error(
                Rule::OutputProtected,
                location,
                format!("output resolves to {resolved}, a layer file of this composition"),
            )
            .with_help("write the export to a different file"),
        );
    }
    let dist = root.join("dist");
    if target.starts_with(root) && !(target.starts_with(&dist) && target != dist) {
        return Some(
            Diagnostic::error(
                Rule::OutputProtected,
                location,
                format!("output resolves to {resolved}, inside the project root but outside dist/"),
            )
            .with_help("write under dist/ or outside the project"),
        );
    }
    None
}

/// Whether two resolved paths name the same file: equal paths, or, on Unix,
/// the same existing inode (hard links, case-insensitive file systems).
fn same_file(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(x), Ok(y)) = (fs::metadata(a), fs::metadata(b)) {
            return x.dev() == y.dev() && x.ino() == y.ino();
        }
    }
    false
}

/// Make `path` absolute and follow every symbolic link in it, the way the
/// kernel would. Components that do not exist yet are appended as written,
/// and `..` removes the previous component of the resolved prefix.
fn resolve(path: &Path) -> io::Result<PathBuf> {
    let mut resolved = PathBuf::new();
    let mut pending = Vec::new();
    queue(&mut pending, &mut resolved, &std::path::absolute(path)?);
    let mut links = 0;
    while let Some(part) = pending.pop() {
        let Part::Name(name) = part else {
            resolved.pop();
            continue;
        };
        let candidate = resolved.join(&name);
        match fs::symlink_metadata(&candidate) {
            Ok(meta) if meta.file_type().is_symlink() => {
                links += 1;
                if links > MAX_LINKS {
                    return Err(io::Error::other(format!(
                        "too many levels of symbolic links at {}",
                        candidate.display()
                    )));
                }
                let link = fs::read_link(&candidate)?;
                queue(&mut pending, &mut resolved, &link);
            }
            _ => resolved = candidate,
        }
    }
    Ok(resolved)
}

enum Part {
    Name(OsString),
    Parent,
}

/// Queue the components of `path` so the first is popped next. A rooted
/// path restarts `resolved` from its root.
fn queue(pending: &mut Vec<Part>, resolved: &mut PathBuf, path: &Path) {
    let start = pending.len();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => *resolved = PathBuf::from(prefix.as_os_str()),
            Component::RootDir => resolved.push(Component::RootDir),
            Component::CurDir => {}
            Component::ParentDir => pending.push(Part::Parent),
            Component::Normal(name) => pending.push(Part::Name(name.to_owned())),
        }
    }
    pending[start..].reverse();
}

fn persist(target: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| io::Error::other("the output has no parent directory"))?;
    fs::create_dir_all(parent)?;
    let mut file = temp_file(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    file.persist(target).map_err(|e| e.error)?;
    Ok(())
}

/// A temporary file beside the target, created with the permissions a
/// plain file would get (`0o666` minus the umask) instead of `0o600`.
fn temp_file(dir: &Path) -> io::Result<NamedTempFile> {
    let mut builder = tempfile::Builder::new();
    builder.prefix(".fjord-");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(fs::Permissions::from_mode(0o666));
    }
    builder.tempfile_in(dir)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use tempfile::TempDir;

    struct Project {
        _dir: TempDir,
        root: PathBuf,
    }

    fn project() -> Project {
        let dir = TempDir::new().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap().join("repo");
        fs::create_dir_all(root.join("tokens")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("tokens/base.tokens.json"), "base").unwrap();
        Project { _dir: dir, root }
    }

    fn rule(outcome: &Outcome<()>) -> Option<Rule> {
        outcome.as_ref().err().map(|f| f.0[0].rule)
    }

    #[test]
    fn o4_dist_is_writable_and_created() {
        let p = project();
        let out = p.root.join("dist/fjord.night.tokens.json");
        write(&p.root, &out, b"{}\n", &[]).unwrap();
        assert_eq!(fs::read(&out).unwrap(), b"{}\n");
    }

    #[test]
    fn o4_inside_root_outside_dist_is_refused() {
        let p = project();
        for rel in [
            "tokens/base.tokens.json",
            "out.json",
            "dist",
            "assets/x.json",
            "distx/a.json",
        ] {
            let out = p.root.join(rel);
            let outcome = write(&p.root, &out, b"x", &[]);
            assert_eq!(rule(&outcome), Some(Rule::OutputProtected), "{rel}");
        }
        assert_eq!(
            fs::read(p.root.join("tokens/base.tokens.json")).unwrap(),
            b"base"
        );
        assert!(!p.root.join("out.json").exists());
    }

    #[test]
    fn o4_parent_components_cannot_escape_dist() {
        let p = project();
        let out = p.root.join("dist/new/../../tokens/base.tokens.json");
        assert_eq!(
            rule(&write(&p.root, &out, b"x", &[])),
            Some(Rule::OutputProtected)
        );
        assert!(!p.root.join("dist/new").exists());
        assert_eq!(
            fs::read(p.root.join("tokens/base.tokens.json")).unwrap(),
            b"base"
        );
    }

    #[test]
    fn o4_layer_files_are_refused_even_outside_root() {
        let p = project();
        let outside = p.root.parent().unwrap().join("override.tokens.json");
        fs::write(&outside, "layer").unwrap();
        let outcome = write(&p.root, &outside, b"x", std::slice::from_ref(&outside));
        assert_eq!(rule(&outcome), Some(Rule::OutputProtected));
        assert_eq!(fs::read(&outside).unwrap(), b"layer");

        let in_dist = p.root.join("dist/layer.tokens.json");
        fs::create_dir_all(p.root.join("dist")).unwrap();
        fs::write(&in_dist, "layer").unwrap();
        let outcome = write(&p.root, &in_dist, b"x", std::slice::from_ref(&in_dist));
        assert_eq!(rule(&outcome), Some(Rule::OutputProtected));
    }

    #[cfg(unix)]
    #[test]
    fn o4_symlinks_are_resolved_before_checking() {
        use std::os::unix::fs::symlink;
        let p = project();
        fs::create_dir_all(p.root.join("dist")).unwrap();
        // An output symlink pointing into assets/.
        symlink(p.root.join("assets/icon.svg"), p.root.join("dist/icon.svg")).unwrap();
        let outcome = write(&p.root, &p.root.join("dist/icon.svg"), b"x", &[]);
        assert_eq!(rule(&outcome), Some(Rule::OutputProtected));
        assert!(!p.root.join("assets/icon.svg").exists());
        // A directory symlink inside dist/ pointing at tokens/.
        symlink(p.root.join("tokens"), p.root.join("dist/tokens")).unwrap();
        let outcome = write(
            &p.root,
            &p.root.join("dist/tokens/base.tokens.json"),
            b"x",
            &[],
        );
        assert_eq!(rule(&outcome), Some(Rule::OutputProtected));
        assert_eq!(
            fs::read(p.root.join("tokens/base.tokens.json")).unwrap(),
            b"base"
        );
        // A symlink outside the root pointing back into tokens/.
        let outside = p.root.parent().unwrap().join("sneaky.json");
        symlink(p.root.join("tokens/base.tokens.json"), &outside).unwrap();
        let outcome = write(&p.root, &outside, b"x", &[]);
        assert_eq!(rule(&outcome), Some(Rule::OutputProtected));
        // A symlink to a layer file is the layer file.
        let link = p.root.join("dist/base.json");
        symlink(p.root.join("tokens/base.tokens.json"), &link).unwrap();
        let outcome = write(
            &p.root,
            &link,
            b"x",
            &[p.root.join("tokens/base.tokens.json")],
        );
        assert_eq!(rule(&outcome), Some(Rule::OutputProtected));
    }

    #[cfg(unix)]
    #[test]
    fn o4_symlink_loops_fail_as_io() {
        use std::os::unix::fs::symlink;
        let p = project();
        fs::create_dir_all(p.root.join("dist")).unwrap();
        symlink(p.root.join("dist/b"), p.root.join("dist/a")).unwrap();
        symlink(p.root.join("dist/a"), p.root.join("dist/b")).unwrap();
        let outcome = write(&p.root, &p.root.join("dist/a"), b"x", &[]);
        assert_eq!(rule(&outcome), Some(Rule::Io));
    }

    #[cfg(unix)]
    #[test]
    fn o4_symlinked_root_still_protects_sources() {
        use std::os::unix::fs::symlink;
        let p = project();
        let alias = p.root.parent().unwrap().join("alias");
        symlink(&p.root, &alias).unwrap();
        let outcome = write(&alias, &p.root.join("tokens/base.tokens.json"), b"x", &[]);
        assert_eq!(rule(&outcome), Some(Rule::OutputProtected));
        write(&alias, &alias.join("dist/ok.json"), b"ok", &[]).unwrap();
        assert_eq!(fs::read(p.root.join("dist/ok.json")).unwrap(), b"ok");
    }

    #[test]
    fn o4_outside_root_is_allowed() {
        let p = project();
        let out = p.root.parent().unwrap().join("elsewhere/out.css");
        write(&p.root, &out, b"css", &[]).unwrap();
        assert_eq!(fs::read(&out).unwrap(), b"css");
    }

    #[test]
    fn o4_replace_is_atomic_and_failure_keeps_the_old_file() {
        let p = project();
        let out = p.root.join("dist/fjord.night.tokens.css");
        write(&p.root, &out, b"old", &[]).unwrap();
        write(&p.root, &out, b"new", &[]).unwrap();
        assert_eq!(fs::read(&out).unwrap(), b"new");
        // A directory in the way makes the write fail; nothing else changes.
        let blocked = p.root.join("dist/blocked");
        fs::create_dir_all(blocked.join("child")).unwrap();
        assert_eq!(rule(&write(&p.root, &blocked, b"x", &[])), Some(Rule::Io));
        assert!(blocked.join("child").is_dir());
        let leftovers: Vec<_> = fs::read_dir(p.root.join("dist"))
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .filter(|n| n.to_string_lossy().starts_with(".fjord-"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    #[cfg(unix)]
    #[test]
    fn o4_written_files_get_plain_file_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let p = project();
        let out = p.root.join("dist/fjord.night.tokens.json");
        write(&p.root, &out, b"{}", &[]).unwrap();
        let plain = p.root.join("dist/plain");
        fs::write(&plain, b"{}").unwrap();
        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&out), mode(&plain));
    }
}
