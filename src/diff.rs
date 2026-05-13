//! Compare: the FORWARD-DIFF operator with kind-aware, sequence-aware dispatch.
//!
//! - **Both atoms** → content diff (body unified diff + metadata changes).
//! - **Both compounds** → structural diff over `composed_of`, four axes
//!   (`added`, `removed`, `version_changed`, `moved`). The four axes are not
//!   mutually exclusive: a version-changed input that also moved positions
//!   surfaces on both `version_changed` and `moved`. `recipes_equal` is true
//!   iff all four lists are empty.
//! - **Mixed (atom vs compound)** → refused with a clear error.
//!
//! Sequence-awareness matters because reordering inputs changes the rendered
//! output. v0.1 was order-blind for `composed_of`; v0.2 fixes that.

use std::collections::HashMap;

use serde::Serialize;
use similar::TextDiff;

use crate::element::PromptElement;
use crate::error::{OovraError, Result};
use crate::header::{InputRef, PromptElementKind};

/// Top-level result of [`compare`].
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiffReport {
    Content(ContentDiff),
    Structural(StructuralDiff),
}

/// Diff of two atomic (order-0) elements.
#[derive(Debug, Serialize)]
pub struct ContentDiff {
    pub a_id: String,
    pub b_id: String,
    pub field_changes: Vec<FieldChange>,
    pub body_unified_diff: String,
    pub bodies_equal: bool,
}

#[derive(Debug, Serialize)]
pub struct FieldChange {
    pub field: String,
    pub before: String,
    pub after: String,
}

/// Diff of two compounds. v0.2 is sequence-aware: position changes are
/// surfaced via the `moved` list.
#[derive(Debug, Serialize)]
pub struct StructuralDiff {
    pub a_id: String,
    pub b_id: String,
    /// Inputs present in `b` but not in `a`. Position is `b`-side.
    pub added: Vec<PositionedInput>,
    /// Inputs present in `a` but not in `b`. Position is `a`-side.
    pub removed: Vec<PositionedInput>,
    /// Inputs present in both with the same id but a different version pin.
    pub version_changed: Vec<VersionChange>,
    /// Inputs present in both with the same id and version but at a different
    /// position in the `composed_of` array. Reordering changes the rendered
    /// output, so this is a real diff.
    pub moved: Vec<Move>,
    /// True iff added, removed, version_changed, and moved are all empty.
    pub recipes_equal: bool,
}

/// One input together with its position in the input list.
#[derive(Debug, Clone, Serialize)]
pub struct PositionedInput {
    pub position: usize,
    pub input: InputRef,
}

/// An input that exists in both compounds with the same `id@version` but at
/// a different position in `composed_of`.
#[derive(Debug, Clone, Serialize)]
pub struct Move {
    pub id: String,
    pub version: String,
    pub before_pos: usize,
    pub after_pos: usize,
}

#[derive(Debug, Serialize)]
pub struct VersionChange {
    pub id: String,
    pub before_version: String,
    pub after_version: String,
}

/// Dispatch a comparison between two prompt elements based on their kinds.
/// Atom + atom = content diff; compound + compound = structural diff;
/// mixed = error.
pub fn compare(a: &PromptElement, b: &PromptElement) -> Result<DiffReport> {
    match (a.header.kind, b.header.kind) {
        (PromptElementKind::Atom, PromptElementKind::Atom) => {
            Ok(DiffReport::Content(content_diff(a, b)))
        }
        (PromptElementKind::Compound, PromptElementKind::Compound) => {
            Ok(DiffReport::Structural(structural_diff(a, b)?))
        }
        (a_kind, b_kind) => Err(OovraError::KindMismatch {
            a_id: a.header.id.clone(),
            a_kind: kind_label(a_kind),
            b_id: b.header.id.clone(),
            b_kind: kind_label(b_kind),
        }),
    }
}

fn kind_label(k: PromptElementKind) -> &'static str {
    match k {
        PromptElementKind::Atom => "atom",
        PromptElementKind::Compound => "compound",
    }
}

fn content_diff(a: &PromptElement, b: &PromptElement) -> ContentDiff {
    let mut field_changes = Vec::new();

    track_field(&mut field_changes, "name", &a.header.name, &b.header.name);
    track_field(
        &mut field_changes,
        "version",
        &a.header.version,
        &b.header.version,
    );
    track_field(&mut field_changes, "meta", &a.header.meta, &b.header.meta);

    let diff = TextDiff::from_lines(&a.body, &b.body);
    let body_unified_diff = diff
        .unified_diff()
        .header(&a.header.id, &b.header.id)
        .to_string();
    let bodies_equal = a.body == b.body;

    ContentDiff {
        a_id: a.header.id.clone(),
        b_id: b.header.id.clone(),
        field_changes,
        body_unified_diff,
        bodies_equal,
    }
}

fn track_field(out: &mut Vec<FieldChange>, name: &str, before: &str, after: &str) {
    if before != after {
        out.push(FieldChange {
            field: name.to_string(),
            before: before.to_string(),
            after: after.to_string(),
        });
    }
}

fn structural_diff(a: &PromptElement, b: &PromptElement) -> Result<StructuralDiff> {
    // A valid Compound has composed_of by definition; the validator ensures
    // this before `compare` is reachable.
    let a_inputs = a.header.composed_of.as_ref().expect("compound has composed_of");
    let b_inputs = b.header.composed_of.as_ref().expect("compound has composed_of");

    // Index each side by id → (position, version). Duplicate ids within a
    // single composed_of are rare but possible; the second occurrence wins,
    // which means duplicate-id moves will not be detected. Documented in
    // the module docstring.
    let a_by_id: HashMap<&str, (usize, &str)> = a_inputs
        .iter()
        .enumerate()
        .map(|(pos, i)| (i.id.as_str(), (pos, i.version.as_str())))
        .collect();
    let b_by_id: HashMap<&str, (usize, &str)> = b_inputs
        .iter()
        .enumerate()
        .map(|(pos, i)| (i.id.as_str(), (pos, i.version.as_str())))
        .collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut version_changed = Vec::new();
    let mut moved = Vec::new();

    // Walk b: classify each b-input as added (id not in a), version_changed
    // (id in a, version differs), and/or moved (id in a, position differs).
    // version_changed and moved are not mutually exclusive — a single input
    // can both have its pin bumped and shift position; both axes report it.
    for (b_pos, input) in b_inputs.iter().enumerate() {
        match a_by_id.get(input.id.as_str()) {
            Some(&(a_pos, a_ver)) => {
                if a_ver != input.version.as_str() {
                    version_changed.push(VersionChange {
                        id: input.id.clone(),
                        before_version: a_ver.to_string(),
                        after_version: input.version.clone(),
                    });
                }
                if a_pos != b_pos {
                    moved.push(Move {
                        id: input.id.clone(),
                        version: input.version.clone(),
                        before_pos: a_pos,
                        after_pos: b_pos,
                    });
                }
            }
            None => added.push(PositionedInput {
                position: b_pos,
                input: input.clone(),
            }),
        }
    }

    // Walk a: classify each a-input absent from b as removed.
    for (a_pos, input) in a_inputs.iter().enumerate() {
        if !b_by_id.contains_key(input.id.as_str()) {
            removed.push(PositionedInput {
                position: a_pos,
                input: input.clone(),
            });
        }
    }

    let recipes_equal = added.is_empty()
        && removed.is_empty()
        && version_changed.is_empty()
        && moved.is_empty();

    Ok(StructuralDiff {
        a_id: a.header.id.clone(),
        b_id: b.header.id.clone(),
        added,
        removed,
        version_changed,
        moved,
        recipes_equal,
    })
}
