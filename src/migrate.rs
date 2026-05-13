//! Migrate: in-place rewrite of a v0.1 library to v0.2 schema.
//!
//! Walks a directory recursively, parses each `.md` file via the legacy path
//! (which accepts both v0.1 and v0.2 shapes), and rewrites it in v0.2 format.
//! Idempotent on files already in v0.2 form — they are re-serialized, which
//! may canonicalize field order but does not change content.
//!
//! `generated_at` on compounds is preserved verbatim. Migration is a
//! representation change, not a regeneration (SPEC §10.2).
//!
//! Each rewrite is validated in memory before disk write (the same paranoia
//! check `element::write` already enforces), so a file that would produce an
//! invalid v0.2 result is left untouched on disk and reported as failed.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::decompose::decompose_with;
use crate::element::{
    looks_like_oovra_file, parse_file_with, serialize, write, ParseOptions, PromptElement,
};
use crate::error::{OovraError, Result};
use crate::render::render_body;

/// Per-file outcome of a migration walk.
#[derive(Debug, Default)]
pub struct MigrationSummary {
    /// Paths that were rewritten in v0.2 format.
    pub migrated: Vec<PathBuf>,
    /// `.md` files skipped, with a reason tag (e.g. "not-oovra").
    pub skipped: Vec<(PathBuf, &'static str)>,
    /// Files that failed to parse or write; the original file is left in place.
    pub failed: Vec<(PathBuf, OovraError)>,
}

/// Walk `root` recursively, migrate every Oovra `.md` file from v0.1 → v0.2
/// schema in place. Non-Oovra `.md` files (no `+++` opener) are skipped.
/// Files that fail to parse or whose rewrite would be invalid are reported
/// as `failed` and left untouched on disk.
pub fn migrate_library(root: &Path) -> Result<MigrationSummary> {
    if !root.exists() {
        return Err(OovraError::FileNotFound(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(OovraError::NotADirectory(root.to_path_buf()));
    }

    let mut summary = MigrationSummary::default();
    let opts = ParseOptions { legacy: true };

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        // Cheap pre-check: does it look like an Oovra file at all?
        let preview = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                summary.skipped.push((path.to_path_buf(), "io-error"));
                continue;
            }
        };
        if !looks_like_oovra_file(&preview) {
            summary.skipped.push((path.to_path_buf(), "not-oovra"));
            continue;
        }

        match parse_file_with(path, opts) {
            Ok(element) => match rewrite_with_body_migration(&element) {
                Ok(migrated) => match write(&migrated, path) {
                    Ok(()) => summary.migrated.push(path.to_path_buf()),
                    Err(e) => summary.failed.push((path.to_path_buf(), e)),
                },
                Err(e) => summary.failed.push((path.to_path_buf(), e)),
            },
            Err(e) => summary.failed.push((path.to_path_buf(), e)),
        }
    }

    Ok(summary)
}

/// Produce a v0.2 version of `element` with its body recursively migrated.
///
/// Compounds carry their full sub-tree as wrapped-frontmatter blocks inside
/// their body. Migrating the outer frontmatter alone leaves the embedded
/// sub-elements in v0.1 form — they would then fail v0.2-strict parsing
/// (which is what `decompose` does after migration completes).
///
/// This function decomposes the body in legacy mode, recursively migrates
/// each sub-element, and re-renders the body from the migrated children.
/// Atoms are returned unchanged (their header is already v0.2 from the
/// outer `parse_file_with(legacy)` call).
fn rewrite_with_body_migration(element: &PromptElement) -> Result<PromptElement> {
    if element.header.is_atom() {
        return Ok(element.clone());
    }
    // Compound: decompose body in legacy mode, recursively migrate each
    // sub-element, re-render the body. body_level is already correct on
    // the outer header (preserved from v0.1).
    let body_level = element
        .header
        .body_level
        .expect("compound validation guarantees body_level");
    let sub_elements = decompose_with(element, ParseOptions { legacy: true })?;
    let mut migrated_chunks: Vec<String> = Vec::with_capacity(sub_elements.len());
    for sub in sub_elements {
        let migrated_sub = rewrite_with_body_migration(&sub)?;
        migrated_chunks.push(serialize(&migrated_sub)?);
    }
    let new_body = render_body(body_level, &migrated_chunks);
    Ok(PromptElement::new(element.header.clone(), new_body))
}
