//! Compose: the JOIN operator.
//!
//! Compose takes an ordered list of input prompt elements and produces a
//! single compound. The output's `body_level` is computed by
//! [`compute_body_level`]: always `max(input.body_level, default = 0) + 1`,
//! which guarantees the chiral-delimiter escalation rule the body parser
//! depends on.
//!
//! The output's body is the concatenation of each input's *full file content*
//! (frontmatter + body), wrapped in chiral `body_level`-aware delimiters. This
//! makes every compound lossless: `decompose --full` recovers every element at
//! every level by recursively splitting bodies.

use chrono::Utc;

use crate::element::{body_delimiter_close, body_delimiter_open, serialize, PromptElement};
use crate::error::{OovraError, Result};
use crate::header::{InputRef, PromptElementHeader, PromptElementKind};
use crate::library::Library;

/// Inputs to a Compose operation.
pub struct ComposeRequest<'a> {
    pub library: &'a Library,
    /// Each entry: (id, optional version pin). When the pin is `Some`, the
    /// library's version of the element must match exactly.
    pub inputs: Vec<(String, Option<String>)>,
    pub output_id: String,
    pub output_name: String,
    pub output_version: String,
    pub output_meta: String,
}

/// Compute the **physical body delimiter level** for a Compose output.
///
/// Always `max(input.body_level, default = 0) + 1` where atoms contribute
/// `body_level = 0` and compounds contribute their stored `body_level`. The
/// outer delimiter therefore has strictly more tildes than any inner
/// element's delimiter, so an outer scan for `(level + 1)` tildes never
/// collides with any nested level-`k` (k < level) delimiter.
pub fn compute_body_level(input_body_levels: &[u32]) -> u32 {
    input_body_levels
        .iter()
        .copied()
        .max()
        .map(|m| m + 1)
        .unwrap_or(1)
}

/// Compute the **recipe depth** of a Compose output from the depths of its
/// inputs.
///
/// Atoms contribute `depth = 0` by convention; compounds contribute their
/// stored `depth`. The output is `1 + max(child.depth, default = 0)`, mirroring
/// [`compute_body_level`].
///
/// Numerically, `depth` and `body_level` coincide for every valid compound —
/// each output increments by one from the max input value, and atoms
/// contribute 0 to both. The two fields exist for different audiences:
/// `body_level` is what the body parser scans tildes for; `depth` is a
/// human-readable "how deeply composed is this?" metric exposed for library
/// filtering and tooling.
pub fn compute_depth(input_depths: &[u32]) -> u32 {
    input_depths
        .iter()
        .copied()
        .max()
        .map(|m| m + 1)
        .unwrap_or(1)
}

/// Wrap one input's full file content in level-`body_level` open/close
/// delimiters. Each delimiter sits on its own line.
fn wrap_chunk(body_level: u32, full_file_content: &str) -> String {
    let open = body_delimiter_open(body_level);
    let close = body_delimiter_close(body_level);
    let trimmed = full_file_content.trim_end_matches('\n');
    format!("{open}\n{trimmed}\n{close}")
}

/// Render the body of a composed element by concatenating the full-file
/// content of each input separated by level-`body_level` delimiters.
pub fn render_body(body_level: u32, input_files: &[String]) -> String {
    input_files
        .iter()
        .map(|f| wrap_chunk(body_level, f))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Resolve and render a Compose request into a [`PromptElement`].
pub fn compose(req: ComposeRequest<'_>) -> Result<PromptElement> {
    if req.inputs.is_empty() {
        return Err(OovraError::EmptyCompose);
    }

    // Resolve inputs against the library, enforcing version pins.
    let mut resolved: Vec<&PromptElement> = Vec::with_capacity(req.inputs.len());
    let mut input_refs: Vec<InputRef> = Vec::with_capacity(req.inputs.len());

    for (id, pin) in &req.inputs {
        let element = req
            .library
            .get(id)
            .ok_or_else(|| OovraError::ElementNotFound { id: id.clone() })?;

        if let Some(pin) = pin {
            if &element.header.version != pin {
                return Err(OovraError::VersionMismatch {
                    id: id.clone(),
                    pin: pin.clone(),
                    actual: element.header.version.clone(),
                });
            }
        }

        resolved.push(element);
        input_refs.push(InputRef::new(id.clone(), element.header.version.clone()));
    }

    // Compute the physical body delimiter level. Atoms contribute 0 (they
    // have no body_level); compounds contribute their stored body_level.
    // The output level escalates by 1 to satisfy strict monotonicity.
    let input_body_levels: Vec<u32> = resolved
        .iter()
        .map(|e| e.header.body_level.unwrap_or(0))
        .collect();
    let body_level = compute_body_level(&input_body_levels);

    // Compute the recipe depth in parallel. Atoms contribute 0; compounds
    // contribute their stored depth (or fall back to body_level on legacy
    // compounds that pre-date the depth field — they're guaranteed equal).
    let input_depths: Vec<u32> = resolved
        .iter()
        .map(|e| e.header.depth.or(e.header.body_level).unwrap_or(0))
        .collect();
    let depth = compute_depth(&input_depths);

    // Render each input as a complete file string (frontmatter + body),
    // wrap each in level-`body_level` delimiters, and concatenate.
    let mut input_files: Vec<String> = Vec::with_capacity(resolved.len());
    for input in &resolved {
        input_files.push(serialize(input)?);
    }
    let body = render_body(body_level, &input_files);

    let header = PromptElementHeader {
        name: req.output_name,
        kind: PromptElementKind::Compound,
        id: req.output_id,
        version: req.output_version,
        meta: req.output_meta,
        generated_at: Some(Utc::now().to_rfc3339()),
        render_mode: Some("markdown-h2".to_string()),
        body_level: Some(body_level),
        depth: Some(depth),
        composed_of: Some(input_refs),
    };

    Ok(PromptElement::new(header, body))
}

/// Render a clean human-readable prompt from a list of inputs, suitable for
/// the `compose --text` flag. Each input's body is wrapped in a Markdown H2
/// containing its ID; sub-element headers are stripped.
///
/// This is intentionally NOT a valid Oovra file — it's the "give me a prompt
/// to paste into a model" output. Use `compose` (without `--text`) when you
/// want a self-describing on-disk artifact.
pub fn render_text(inputs: &[&PromptElement]) -> Result<String> {
    let parts: Vec<String> = inputs
        .iter()
        .map(|e| render_for_paste(e))
        .collect::<Result<Vec<_>>>()?;
    Ok(parts.join("\n\n"))
}

/// Recursively render a prompt element to its prose form: an H2 header per
/// "leaf" (order-0) element, with no Oovra metadata leaking into the output.
/// For order >= 1 elements, the body is parsed for embedded sub-elements and
/// the recursion descends.
fn render_for_paste(element: &PromptElement) -> Result<String> {
    if element.header.is_atom() {
        return Ok(format!(
            "## {}\n\n{}",
            element.header.id,
            element.body.trim()
        ));
    }
    // Composed element: split its body into immediate sub-elements and
    // render each. This collapses arbitrarily nested compositions into a
    // flat list of H2-wrapped order-0 bodies.
    let sub_elements = crate::decompose::decompose(element)?;
    let parts: Vec<String> = sub_elements
        .iter()
        .map(render_for_paste)
        .collect::<Result<Vec<_>>>()?;
    Ok(parts.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_body_wraps_each_input_in_correct_delimiters() {
        let chunks = vec!["FILE_A".to_string(), "FILE_B".to_string()];
        let body = render_body(1, &chunks);
        assert!(body.contains("~~>>\nFILE_A\n~~<<"));
        assert!(body.contains("~~>>\nFILE_B\n~~<<"));

        let body2 = render_body(2, &chunks);
        assert!(body2.contains("~~~>>\nFILE_A\n~~~<<"));
        assert!(body2.contains("~~~>>\nFILE_B\n~~~<<"));
    }

    #[test]
    fn body_level_always_strictly_greater_than_max_input_body_level() {
        // Atoms contribute 0; compounds contribute their stored body_level.
        // The output is always max+1, guaranteeing strict monotonicity of the
        // chiral-delimiter scheme.
        assert_eq!(compute_body_level(&[0, 0, 0]), 1);
        assert_eq!(compute_body_level(&[1, 1]), 2);
        assert_eq!(compute_body_level(&[1, 0, 0, 0]), 2);
        assert_eq!(compute_body_level(&[2, 1, 0, 0, 0]), 3);
        assert_eq!(compute_body_level(&[2, 2, 1]), 3);
        assert_eq!(compute_body_level(&[5]), 6);
        assert_eq!(compute_body_level(&[]), 1);
    }

    #[test]
    fn compute_depth_mirrors_compute_body_level() {
        // SPEC §1.3: depth = 1 + max(child.depth, atoms=0). Numerically
        // identical to compute_body_level — the two fields exist for
        // different audiences (parser vs. humans/tools) but always agree.
        for inputs in [
            &[0u32, 0, 0][..],
            &[1, 1][..],
            &[1, 0, 0, 0][..],
            &[2, 1, 0, 0, 0][..],
            &[2, 2, 1][..],
            &[5][..],
            &[][..],
        ] {
            assert_eq!(
                compute_depth(inputs),
                compute_body_level(inputs),
                "depth and body_level diverged on input {:?}",
                inputs
            );
        }
    }
}
