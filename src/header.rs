//! The TOML frontmatter schema for every Oovra prompt element.
//!
//! Every Oovra file is a `PromptElement`. Its `kind` field tells you which of
//! two shapes the file has:
//!
//! - `kind = "atom"` is a hand-authored element. Its body is freeform
//!   Markdown. It has no recipe.
//! - `kind = "compound"` is a composition produced by `oovra compose`. Its
//!   body is a sequence of wrapped sub-prompt-element files (see the chiral
//!   delimiter scheme in [`crate::element`]). It carries a `composed_of`
//!   recipe plus composition metadata (`generated_at`, `render_mode`,
//!   `body_level`, and optional `depth`).
//!
//! Required fields for ALL elements:
//! - `name`, `kind`, `id`, `version`, `meta` (may be empty string)
//!
//! Required when `kind = "compound"`:
//! - `generated_at` (RFC 3339), `render_mode` (e.g. `"markdown-h2"`),
//!   `body_level` (>= 1, used by the body parser), and `composed_of`
//!   (non-empty array of immediate-input `{id, version}` records).
//!
//! Optional on compounds, derived from the recipe: `depth`.

use serde::{Deserialize, Serialize};

/// The two kinds of prompt element. An **atom** is hand-authored and
/// indivisible; a **compound** is produced by `oovra compose` and carries a
/// `composed_of` recipe.
///
/// v0.2 introduces this as the canonical kind discriminator, replacing the
/// numeric `order` field. In v0.2's commit-2 transition state the enum exists
/// alongside `order`; commit 3 removes `order` and makes `kind` the sole
/// discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PromptElementKind {
    #[default]
    Atom,
    Compound,
}

/// One immediate input to a composed element. Recorded in the `composed_of`
/// array of order >= 1 elements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRef {
    pub id: String,
    pub version: String,
}

impl InputRef {
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
        }
    }
}

/// The full TOML frontmatter of a prompt element. One struct for both kinds
/// of element (`atom` and `compound`); fields that are only meaningful for
/// compounds are `Option<T>` and validated jointly against `kind`.
///
/// `body_level` is the **physical delimiter level** used inside the body of a
/// compound — always `max(input.body_level, default = 0) + 1` at compose
/// time. This satisfies the strict-monotonicity escalation rule the body
/// parser depends on, regardless of how the inputs are bracketed. `depth` is
/// the same number minus one, exposed as a human-friendly compositional-depth
/// metric; the body parser does not consume it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptElementHeader {
    pub name: String,
    /// The kind of element. Canonical v0.2 discriminator — `"atom"` for
    /// hand-authored, `"compound"` for composed. Required on every file.
    pub kind: PromptElementKind,
    pub id: String,
    pub version: String,
    /// Free-form description; may be empty string.
    #[serde(default)]
    pub meta: String,

    /// RFC 3339 timestamp of composition. Required when `composed_of` is
    /// present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,

    /// Renderer identifier. Required when `composed_of` is present. v0.1
    /// supports `"markdown-h2"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_mode: Option<String>,

    /// Physical body delimiter level: the number of tildes minus one, used
    /// in the body's open/close delimiters. Required when `composed_of` is
    /// present; forbidden otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_level: Option<u32>,

    /// Compositional depth of the recipe tree: 0 for atoms, `1 + max(child.depth)`
    /// for compounds. Optional, derived. Compose writes it in v0.2 commit 4;
    /// the validator checks the per-file constraint `depth >= 1` when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,

    /// The immediate inputs (one level down) that produced this element.
    /// Required for any composed element; absent for hand-authored atomics.
    /// Recursion is recorded in the body, not here — see [`crate::element`]
    /// for the body delimiter scheme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composed_of: Option<Vec<InputRef>>,
}

impl PromptElementHeader {
    /// True if this element is an atom (hand-authored, no recipe).
    pub fn is_atom(&self) -> bool {
        matches!(self.kind, PromptElementKind::Atom)
    }

    /// True if this element is a compound (produced by Compose, has a recipe).
    pub fn is_compound(&self) -> bool {
        matches!(self.kind, PromptElementKind::Compound)
    }
}

/// Validate kebab-case: lowercase letters, digits, and hyphens only;
/// no leading/trailing hyphen; no double-hyphen.
pub fn is_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.starts_with('-') || s.ends_with('-') {
        return false;
    }
    if s.contains("--") {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Validate semver per the `semver` crate.
pub fn is_valid_semver(s: &str) -> bool {
    semver::Version::parse(s).is_ok()
}

/// Validate RFC 3339 timestamp (a strict subset of ISO 8601 sufficient for
/// timestamps with timezone).
pub fn is_valid_rfc3339(s: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(s).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kebab_case_accepts_valid_ids() {
        assert!(is_kebab_case("role-declaration"));
        assert!(is_kebab_case("a"));
        assert!(is_kebab_case("a-b-c"));
        assert!(is_kebab_case("v2-beta"));
        assert!(is_kebab_case("policy-007"));
    }

    #[test]
    fn kebab_case_rejects_invalid_ids() {
        assert!(!is_kebab_case(""));
        assert!(!is_kebab_case("Role"));
        assert!(!is_kebab_case("role_declaration"));
        assert!(!is_kebab_case("role declaration"));
        assert!(!is_kebab_case("-leading"));
        assert!(!is_kebab_case("trailing-"));
        assert!(!is_kebab_case("double--dash"));
    }

    #[test]
    fn semver_accepts_valid_versions() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("0.1.0"));
        assert!(is_valid_semver("2.3.1-rc1"));
    }

    #[test]
    fn semver_rejects_invalid_versions() {
        assert!(!is_valid_semver("v1.0"));
        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver(""));
        assert!(!is_valid_semver("1"));
    }

    #[test]
    fn rfc3339_accepts_valid_timestamps() {
        assert!(is_valid_rfc3339("2026-05-09T14:23:15Z"));
        assert!(is_valid_rfc3339("2026-05-09T14:23:15+00:00"));
        assert!(is_valid_rfc3339("2026-05-09T14:23:15.123Z"));
    }

    #[test]
    fn rfc3339_rejects_invalid_timestamps() {
        assert!(!is_valid_rfc3339("2026-05-09"));
        assert!(!is_valid_rfc3339("2026-05-09 14:23:15"));
        assert!(!is_valid_rfc3339(""));
    }
}
