//! Error type for Oovra operations.
//!
//! Error messages are part of the agent-facing API: every variant attaches the
//! file path and explains specifically what went wrong. An LLM agent reading
//! "Field 'version' missing in nodes/refusal.md" can act; "TOML parse failed"
//! cannot.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OovraError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write {path}: {source}")]
    WriteIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Empty file: {0}")]
    EmptyFile(PathBuf),

    #[error("Missing opening '+++' delimiter on line 1 of {path}. Expected '+++', got '{actual}'.")]
    MissingOpenDelimiter { path: PathBuf, actual: String },

    #[error("Missing closing '+++' delimiter in {0}. Frontmatter must be terminated by '+++' on its own line.")]
    MissingCloseDelimiter(PathBuf),

    #[error("Invalid TOML in frontmatter of {path}: {source}")]
    InvalidToml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Failed to serialize TOML for element '{id}': {source}")]
    TomlSerialize {
        id: String,
        #[source]
        source: toml::ser::Error,
    },

    #[error("Missing required field '{field}' in {path}")]
    MissingField { path: PathBuf, field: &'static str },

    #[error("Field '{field}' in {path} has invalid value '{value}': {reason}")]
    InvalidField {
        path: PathBuf,
        field: &'static str,
        value: String,
        reason: String,
    },

    #[error("Atom '{id}' in {path} has forbidden field '{field}'. Atoms have no recipe and no composition metadata.")]
    AtomHasForbiddenField {
        path: PathBuf,
        id: String,
        field: &'static str,
    },

    #[error("Compound '{id}' in {path} is missing required field '{field}'.")]
    CompoundMissingField {
        path: PathBuf,
        id: String,
        field: &'static str,
    },

    #[error("Empty body in {0}. The body must be non-empty after stripping whitespace.")]
    EmptyBody(PathBuf),

    // Removed in v0.2: OrderRequiresField, HandAuthoredHigherOrder.
    // Replaced by MissingField (above) plus the kind-aware validator in
    // src/element.rs.

    #[error("Duplicate ID '{id}' in library: '{first}' and '{second}'")]
    DuplicateId {
        id: String,
        first: PathBuf,
        second: PathBuf,
    },

    #[error("Element '{id}' not found in library")]
    ElementNotFound { id: String },

    #[error("Version mismatch for '{id}': pin '{pin}' does not match library version '{actual}'")]
    VersionMismatch {
        id: String,
        pin: String,
        actual: String,
    },

    // Removed in v0.2: OrderMismatch. Order is gone; kind-mismatch is the
    // remaining axis of disagreement, reported via KindMismatch below.

    #[error("Cannot compare an atom with a compound: '{a_id}' is an {a_kind}, '{b_id}' is a {b_kind}.")]
    KindMismatch {
        a_id: String,
        a_kind: &'static str,
        b_id: String,
        b_kind: &'static str,
    },

    #[error("Compose requires at least one input")]
    EmptyCompose,

    #[error("File {0} already has an Oovra header. Use --force to overwrite.")]
    AlreadyLabeled(PathBuf),

    #[error("Cannot decompose atom '{id}'. Atoms have no recipe. Only compounds can be decomposed.")]
    CannotDecomposeAtom { id: String },

    #[error("Body of compound '{id}' at body_level {body_level} could not be split into the expected sub-element chunks: {reason}")]
    BodyParse {
        id: String,
        body_level: u32,
        reason: String,
    },

    #[error("'{0}' is not a directory")]
    NotADirectory(PathBuf),
}

pub type Result<T> = std::result::Result<T, OovraError>;
