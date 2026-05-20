//! oovra-gui — all-Rust GUI front-end for oovra.
//!
//! This library exposes [`OovraApp`], which is what both the native
//! entry (`src/main.rs` under `cfg(not(target_arch = "wasm32"))`) and
//! the WASM entry (`src/main.rs` under `cfg(target_arch = "wasm32")`)
//! instantiate.

#![warn(clippy::all, rust_2018_idioms)]
// OovraError carries a boxed-ish toml::de::Error in one variant,
// making the enum large enough that any `Result<_, OovraError>` here
// trips clippy::result_large_err. The root `oovra` crate suppresses
// it for the same reason; mirror that decision here.
#![allow(clippy::result_large_err)]

mod app;
mod canvas;
mod editor;
pub use app::OovraApp;
pub use canvas::CanvasState;
pub use editor::{Editor, OpenResult};
