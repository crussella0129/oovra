//! Discovery: walk a directory tree to find `olib/` libraries.
//!
//! [`crate::library::Library::load_with`] opens one olib by path;
//! `discover` finds them in the first place — so the GUI can say
//! "open a folder" and surface every olib in it, and the CLI offers
//! `oovra discover <root>` to do the same from a script or agent.
//!
//! What counts as an olib here: a directory **literally named
//! `olib`** (case-insensitive on Windows; lowercase by convention).
//! That mirrors what `oovra create --olib` writes to, and keeps
//! discovery from false-positive-ing on any directory that happens
//! to contain `.md` files.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::{OovraError, Result};

/// One olib found by [`discover`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredOlib {
    /// Filesystem path of the olib directory.
    pub path: PathBuf,
    /// Number of `.md` files directly inside the directory
    /// (non-recursive). Counts plain Markdown files too — discovery
    /// does not parse them. To get an atom / compound breakdown, the
    /// caller should `Library::load_with(&path)` after selecting.
    pub md_count: usize,
}

/// Walk `root` looking for directories named `olib`. Returns a list
/// of [`DiscoveredOlib`]s sorted by path for deterministic output.
///
/// - `max_depth`: optional depth bound, measured from `root` (root
///   itself is depth 0). `None` walks the entire tree.
/// - Symlinks are not followed (walkdir's default).
/// - Errors on unreadable subtrees are logged to stderr and the walk
///   continues — a single permission-denied entry doesn't kill the
///   whole scan.
pub fn discover(root: &Path, max_depth: Option<usize>) -> Result<Vec<DiscoveredOlib>> {
    if !root.exists() {
        return Err(OovraError::FileNotFound(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(OovraError::InvalidField {
            path: root.to_path_buf(),
            field: "root",
            value: root.display().to_string(),
            reason: "discover requires a directory root".to_string(),
        });
    }

    let mut walker = WalkDir::new(root);
    if let Some(max) = max_depth {
        walker = walker.max_depth(max);
    }

    let mut out: Vec<DiscoveredOlib> = Vec::new();
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("oovra: discover: skipping unreadable entry: {e}");
                continue;
            }
        };
        if !entry.file_type().is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str() else {
            continue;
        };
        if !name.eq_ignore_ascii_case("olib") {
            continue;
        }
        let path = entry.into_path();
        let md_count = count_md_files(&path);
        out.push(DiscoveredOlib { path, md_count });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Count `.md` files directly inside `dir` (non-recursive). Errors at
/// this level are silently treated as zero — by the time we get here
/// the directory was already confirmed to exist by walkdir.
fn count_md_files(dir: &Path) -> usize {
    match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().map(|t| t.is_file()).unwrap_or(false)
                    && e.path()
                        .extension()
                        .and_then(|x| x.to_str())
                        .map(|x| x.eq_ignore_ascii_case("md"))
                        .unwrap_or(false)
            })
            .count(),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    // Tests get unique temp dirs even when run in parallel.
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn tempdir(name: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let base = std::env::temp_dir().join(format!(
            "oovra-discover-{}-{}-{n}",
            name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "").unwrap();
    }

    #[test]
    fn discover_finds_shallow_olib() {
        let root = tempdir("shallow");
        touch(&root.join("olib/a.md"));
        touch(&root.join("olib/b.md"));
        let r = discover(&root, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].path, root.join("olib"));
        assert_eq!(r[0].md_count, 2);
    }

    #[test]
    fn discover_finds_deep_olib() {
        let root = tempdir("deep");
        touch(&root.join("a/b/olib/x.md"));
        let r = discover(&root, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].path, root.join("a").join("b").join("olib"));
        assert_eq!(r[0].md_count, 1);
    }

    #[test]
    fn discover_respects_max_depth() {
        let root = tempdir("max-depth");
        touch(&root.join("olib/shallow.md"));
        touch(&root.join("a/b/olib/deep.md"));
        // max_depth = 1: root + direct children only.
        // root/olib (depth 1) is found; root/a/b/olib (depth 3) is not.
        let r = discover(&root, Some(1)).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].path, root.join("olib"));
    }

    #[test]
    fn discover_ignores_non_olib_dirs_and_top_level_md() {
        let root = tempdir("non-olib");
        touch(&root.join("notes/a.md")); // wrong dir name
        touch(&root.join("stray.md")); // file at root, not a dir
        let r = discover(&root, None).unwrap();
        assert!(r.is_empty(), "expected no olibs, got {r:?}");
    }

    #[test]
    fn discover_errors_on_missing_root() {
        let r = discover(Path::new("definitely/does/not/exist"), None);
        assert!(matches!(r, Err(OovraError::FileNotFound(_))));
    }

    #[test]
    fn discover_returns_deterministic_order() {
        let root = tempdir("order");
        // touched in non-alphabetical sequence; result must be sorted.
        touch(&root.join("z/olib/.keep"));
        touch(&root.join("a/olib/.keep"));
        touch(&root.join("m/olib/.keep"));
        let r = discover(&root, None).unwrap();
        let names: Vec<String> = r
            .iter()
            .map(|d| {
                d.path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(names, vec!["a", "m", "z"]);
        // `.keep` isn't `.md`, so md_count is zero for each.
        for d in &r {
            assert_eq!(d.md_count, 0);
        }
    }
}
