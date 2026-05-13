//! Decompose: the SPLIT operator.
//!
//! Two modes:
//!
//! 1. **One-level** — split the body of an order-N element into its K
//!    immediate sub-elements (each itself a complete Oovra file). Returns
//!    the parsed sub-elements without writing anything to disk.
//!
//! 2. **Full** — recursively decompose all the way to order-0 leaves and
//!    write the result to disk as a folder tree mirroring the order
//!    hierarchy:
//!
//!    ```text
//!    out/<element-id>/
//!      <element-id>.md          # the element itself
//!      <input-1>/               # subdirectory for an order >= 1 input
//!        <input-1>.md
//!        <leaf-a>.md            # an order-0 leaf
//!        <leaf-b>.md
//!      <leaf-x>.md              # a top-level order-0 input (no subdir)
//!    ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::element::{
    body_delimiter_close, body_delimiter_open, parse_with, serialize, ParseOptions, PromptElement,
};
use crate::error::{OovraError, Result};

/// Split the body of a composed element into its K immediate sub-elements
/// in default (v0.2-only) parse mode. See [`decompose_with`] for legacy-aware
/// decomposition.
pub fn decompose(element: &PromptElement) -> Result<Vec<PromptElement>> {
    decompose_with(element, ParseOptions::default())
}

/// Same as [`decompose`] but each sub-element is parsed with the given
/// [`ParseOptions`]. `oovra migrate` uses this with `legacy: true` to walk
/// v0.1 compounds whose body sub-elements still use the `order` schema.
pub fn decompose_with(element: &PromptElement, opts: ParseOptions) -> Result<Vec<PromptElement>> {
    if element.header.is_atom() {
        return Err(OovraError::CannotDecomposeAtom {
            id: element.header.id.clone(),
        });
    }

    let body_level = element.header.body_level.ok_or_else(|| {
        OovraError::MissingField {
            path: std::path::PathBuf::from(format!("<{}:embedded>", element.header.id)),
            field: "body_level",
        }
    })?;

    let open = body_delimiter_open(body_level);
    let close = body_delimiter_close(body_level);

    let mut chunks: Vec<String> = Vec::new();
    let mut current: Option<Vec<&str>> = None;

    for line in element.body.lines() {
        let trimmed = line.trim_end();
        if trimmed == open {
            if current.is_some() {
                return Err(OovraError::BodyParse {
                    id: element.header.id.clone(),
                    body_level,
                    reason: format!(
                        "encountered '{open}' while still inside an open chunk"
                    ),
                });
            }
            current = Some(Vec::new());
        } else if trimmed == close {
            match current.take() {
                Some(buf) => chunks.push(buf.join("\n")),
                None => {
                    return Err(OovraError::BodyParse {
                        id: element.header.id.clone(),
                        body_level,
                        reason: format!(
                            "encountered '{close}' without a matching '{open}'"
                        ),
                    });
                }
            }
        } else if let Some(buf) = current.as_mut() {
            buf.push(line);
        }
    }

    if current.is_some() {
        return Err(OovraError::BodyParse {
            id: element.header.id.clone(),
            body_level,
            reason: format!("missing '{close}' to close the final chunk"),
        });
    }

    if chunks.is_empty() {
        return Err(OovraError::BodyParse {
            id: element.header.id.clone(),
            body_level,
            reason: format!("no '{open}' delimiters found in body"),
        });
    }

    // Parse each chunk as a complete Oovra file, honoring the caller's
    // parse mode (legacy vs strict v0.2).
    let mut parsed: Vec<PromptElement> = Vec::with_capacity(chunks.len());
    let synthetic = PathBuf::from(format!("<{}:embedded>", element.header.id));
    for chunk in &chunks {
        parsed.push(parse_with(chunk, &synthetic, opts)?);
    }
    Ok(parsed)
}

/// Recursively decompose `element` to order-0 leaves and write the result
/// as a folder tree under `out_root / <element.id>`.
///
/// Returns the path to the root directory of the decomposed tree.
pub fn decompose_full(element: &PromptElement, out_root: &Path) -> Result<PathBuf> {
    let element_dir = out_root.join(&element.header.id);
    fs::create_dir_all(&element_dir).map_err(|source| OovraError::WriteIo {
        path: element_dir.clone(),
        source,
    })?;

    write_recursive(element, &element_dir)?;

    Ok(element_dir)
}

/// Helper: write `element` as `dir/<id>.md`, then for each immediate input,
/// either write it directly (order-0 leaf) or recurse into a subdirectory
/// (order >= 1 sub-composition).
fn write_recursive(element: &PromptElement, dir: &Path) -> Result<()> {
    // Write this element's file to dir/<id>.md.
    let element_path = dir.join(format!("{}.md", element.header.id));
    let content = serialize(element)?;
    fs::write(&element_path, content).map_err(|source| OovraError::WriteIo {
        path: element_path.clone(),
        source,
    })?;

    if element.header.is_atom() {
        return Ok(());
    }

    // For each immediate input, decompose one level and recurse.
    let immediate_inputs = decompose(element)?;
    for input in immediate_inputs {
        if input.header.is_atom() {
            // Order-0 leaf: write it directly into the current directory.
            let path = dir.join(format!("{}.md", input.header.id));
            let content = serialize(&input)?;
            fs::write(&path, content).map_err(|source| OovraError::WriteIo {
                path: path.clone(),
                source,
            })?;
        } else {
            // Order >= 1 sub-composition: create a subdirectory and recurse.
            let sub_dir = dir.join(&input.header.id);
            fs::create_dir_all(&sub_dir).map_err(|source| OovraError::WriteIo {
                path: sub_dir.clone(),
                source,
            })?;
            write_recursive(&input, &sub_dir)?;
        }
    }

    Ok(())
}

/// Produce a human-readable report listing the immediate inputs of a
/// composed element (id, version, order). For the `decompose` command's
/// no-write inspection mode.
pub fn report(element: &PromptElement) -> Result<DecomposeReport> {
    if element.header.is_atom() {
        return Err(OovraError::CannotDecomposeAtom {
            id: element.header.id.clone(),
        });
    }
    let immediate = decompose(element)?;
    let entries = immediate
        .iter()
        .map(|e| ReportEntry {
            id: e.header.id.clone(),
            version: e.header.version.clone(),
            kind: kind_str(&e.header.kind),
            name: e.header.name.clone(),
        })
        .collect();

    Ok(DecomposeReport {
        element_id: element.header.id.clone(),
        element_kind: kind_str(&element.header.kind),
        element_version: element.header.version.clone(),
        body_level: element.header.body_level.unwrap_or(0),
        render_mode: element.header.render_mode.clone(),
        inputs: entries,
    })
}

fn kind_str(k: &crate::header::PromptElementKind) -> &'static str {
    match k {
        crate::header::PromptElementKind::Atom => "atom",
        crate::header::PromptElementKind::Compound => "compound",
    }
}

#[derive(Debug, serde::Serialize)]
pub struct DecomposeReport {
    pub element_id: String,
    pub element_kind: &'static str,
    pub element_version: String,
    pub body_level: u32,
    pub render_mode: Option<String>,
    pub inputs: Vec<ReportEntry>,
}

#[derive(Debug, serde::Serialize)]
pub struct ReportEntry {
    pub id: String,
    pub version: String,
    pub kind: &'static str,
    pub name: String,
}
