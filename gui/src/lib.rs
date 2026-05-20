//! oovra-gui — all-Rust GUI front-end for oovra.
//!
//! This library exposes [`OovraApp`], which is what both the native
//! entry (`src/main.rs` under `cfg(not(target_arch = "wasm32"))`) and
//! the WASM entry (`src/main.rs` under `cfg(target_arch = "wasm32")`)
//! instantiate.

#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::OovraApp;
