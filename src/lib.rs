//! Oovra: a tool for the composition and comparison of agentic system
//! prompts from Markdown+TOML "prompt elements".
//!
//! The library exposes the parser, library loader, and the four operators —
//! Create, Compose, Decompose, Compare — for use by the `oovra` binary or
//! any other Rust consumer.

// `OovraError::InvalidToml` wraps `toml::de::Error` which is ~128 bytes, making
// the enum large. Boxing the source on every error path is a runtime cost paid
// for a stack-size lint with no real consequence in this codebase. Suppressed
// at the crate root; revisit if/when the error type grows or is benchmarked.
#![allow(clippy::result_large_err)]

pub mod create;
pub mod decompose;
pub mod diff;
pub mod element;
pub mod error;
pub mod header;
pub mod library;
pub mod render;

pub use element::{
    parse, parse_file, parse_file_with, parse_with, serialize, write, ParseOptions, PromptElement,
};
pub use error::{OovraError, Result};
pub use header::{InputRef, LegacyHeader, PromptElementHeader, PromptElementKind};
pub use library::Library;
