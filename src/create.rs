//! Create: scaffold a new atom from scratch, or label an existing Markdown
//! file by prepending a generated header.
//!
//! Both modes produce atoms only — by construction Create cannot produce a
//! compound. Use `compose` for that.
//!
//! Both modes verify their work by re-parsing the output through the Stage 1
//! parser. If the post-write parse fails, Create reports the error rather
//! than leaving a broken file in the library.

use std::fs;
use std::path::{Path, PathBuf};

use crate::element::{looks_like_oovra_file, write, PromptElement};
use crate::error::{OovraError, Result};
use crate::header::{PromptElementHeader, PromptElementKind};

pub struct ScaffoldArgs {
    pub library_dir: PathBuf,
    pub id: String,
    pub name: Option<String>,
    pub version: String,
    pub meta: String,
}

/// Scaffold a new order-0 element from scratch into `library_dir/<id>.md`.
pub fn scaffold(args: ScaffoldArgs) -> Result<PathBuf> {
    let path = args.library_dir.join(format!("{}.md", args.id));

    let header = PromptElementHeader {
        name: args.name.unwrap_or_else(|| args.id.clone()),
        kind: PromptElementKind::Atom,
        id: args.id.clone(),
        version: args.version,
        meta: args.meta,
        generated_at: None,
        render_mode: None,
        body_level: None,
        depth: None,
        composed_of: None,
    };

    let body = format!(
        "<!-- TODO: write the prompt body for `{}` here. \
         This element is an atom — hand-authored, internally consistent, portable across compositions. -->",
        args.id
    );

    let element = PromptElement::new(header, body);
    write(&element, &path)?;
    Ok(path)
}

pub struct LabelArgs {
    pub source_path: PathBuf,
    pub id: String,
    pub name: Option<String>,
    pub version: String,
    pub meta: String,
    pub force: bool,
}

/// Take an existing Markdown file (without an Oovra header) and prepend a
/// generated header in place. Refuses to overwrite a file that already has
/// an Oovra header unless `force` is true.
pub fn label(args: LabelArgs) -> Result<PathBuf> {
    if !args.source_path.exists() {
        return Err(OovraError::FileNotFound(args.source_path.clone()));
    }

    let original = fs::read_to_string(&args.source_path).map_err(|source| OovraError::Io {
        path: args.source_path.clone(),
        source,
    })?;

    if looks_like_oovra_file(&original) && !args.force {
        return Err(OovraError::AlreadyLabeled(args.source_path.clone()));
    }

    let body = if looks_like_oovra_file(&original) {
        // Force-relabel: try to peel off the existing header by splitting at
        // the second `+++`. If that fails, just reuse the original content
        // verbatim — better to keep the body than lose it.
        peel_existing_frontmatter(&original).unwrap_or_else(|| original.clone())
    } else if original.trim().is_empty() {
        format!(
            "<!-- TODO: body for `{}` was empty when labeled -->",
            args.id
        )
    } else {
        original.clone()
    };

    let header = PromptElementHeader {
        name: args.name.unwrap_or_else(|| args.id.clone()),
        kind: PromptElementKind::Atom,
        id: args.id.clone(),
        version: args.version,
        meta: args.meta,
        generated_at: None,
        render_mode: None,
        body_level: None,
        depth: None,
        composed_of: None,
    };

    let element = PromptElement::new(header, body);
    write(&element, &args.source_path)?;
    Ok(args.source_path)
}

/// Best-effort: strip the first `+++ ... +++` frontmatter block. Returns
/// None if the format is non-standard.
fn peel_existing_frontmatter(content: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()?.trim_end() != "+++" {
        return None;
    }
    let mut body_lines: Vec<&str> = Vec::new();
    let mut found_close = false;
    for line in lines {
        if !found_close {
            if line.trim_end() == "+++" {
                found_close = true;
            }
        } else {
            body_lines.push(line);
        }
    }
    if !found_close {
        return None;
    }
    // Skip exactly one blank line after the closing delimiter.
    let start = if body_lines.first().map(|l| l.trim().is_empty()).unwrap_or(false) {
        1
    } else {
        0
    };
    Some(body_lines[start..].join("\n"))
}

/// Convenience: ensure the directory exists.
pub fn ensure_dir(p: &Path) -> Result<()> {
    fs::create_dir_all(p).map_err(|source| OovraError::WriteIo {
        path: p.to_path_buf(),
        source,
    })
}
