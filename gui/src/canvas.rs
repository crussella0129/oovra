//! Autocompose canvas: the working set of library components the
//! user has selected, the order they should compose in, and the
//! plumbing to live-preview / save as a compound.
//!
//! Sprint s3. Pure presenter over `oovra::render` — no new library
//! code; the canvas just remembers an ordered `Vec<String>` of ids
//! and resolves them against the active `Library` on demand.

use std::path::{Path, PathBuf};

use oovra::render::{ComposeRequest, compose, render_text};
use oovra::{Library, OovraError, PromptElement};

/// Working set of canvas state. Lives on `OovraApp`; not persisted
/// across runs (the canvas is a workbench, not a project document).
#[derive(Debug, Default, Clone)]
pub struct CanvasState {
    /// Ids included in the canvas, in display order.
    pub order: Vec<String>,
    /// Output id for save-as-compound.
    pub output_id: String,
    /// Last save status / error string.
    pub status: String,
}

impl CanvasState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.order.iter().any(|x| x == id)
    }

    /// Add an id if absent, remove it (preserving the order of the
    /// others) if present.
    pub fn toggle(&mut self, id: &str) {
        if let Some(pos) = self.order.iter().position(|x| x == id) {
            self.order.remove(pos);
        } else {
            self.order.push(id.to_string());
        }
    }

    /// Add an id if absent; no-op if already present. Returns true
    /// if the canvas grew. Idempotent.
    pub fn add(&mut self, id: &str) -> bool {
        if !self.contains(id) {
            self.order.push(id.to_string());
            true
        } else {
            false
        }
    }

    /// Remove an id if present; no-op if absent. Returns true if the
    /// canvas shrank. Idempotent.
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.order.iter().position(|x| x == id) {
            self.order.remove(pos);
            true
        } else {
            false
        }
    }

    /// Render the live prose preview from the current order.
    /// Returns an error if any id in `order` is missing from `lib`.
    pub fn live_preview(&self, lib: &Library) -> Result<String, OovraError> {
        let resolved = self.resolve(lib)?;
        render_text(&resolved)
    }

    /// Compose the current order into a compound and write it to
    /// `<olib_dir>/<output_id>.md`. Returns the path on success.
    pub fn save_as_compound(&self, lib: &Library, olib_dir: &Path) -> Result<PathBuf, OovraError> {
        if self.output_id.trim().is_empty() {
            return Err(OovraError::InvalidField {
                path: olib_dir.to_path_buf(),
                field: "output_id",
                value: self.output_id.clone(),
                reason: "output id must not be empty".to_string(),
            });
        }
        if self.order.is_empty() {
            return Err(OovraError::EmptyCompose);
        }
        // Build (id, version-pin) tuples. The pin captures the
        // library's current version at compose time so the recipe
        // is reproducible across future library version bumps.
        let mut inputs: Vec<(String, Option<String>)> = Vec::with_capacity(self.order.len());
        for id in &self.order {
            let element = lib
                .get(id)
                .ok_or_else(|| OovraError::ElementNotFound { id: id.clone() })?;
            inputs.push((id.clone(), Some(element.header.version.clone())));
        }
        let req = ComposeRequest {
            library: lib,
            inputs,
            output_id: self.output_id.clone(),
            output_name: self.output_id.clone(),
            output_version: "1.0.0".to_string(),
            output_meta: String::new(),
        };
        let composed = compose(req)?;
        let dest = olib_dir.join(format!("{}.md", self.output_id));
        oovra::write(&composed, &dest)?;
        Ok(dest)
    }

    /// Resolve the canvas's ids against `lib`. Errors with the
    /// first missing id, matching `compose`'s own behavior.
    fn resolve<'a>(&self, lib: &'a Library) -> Result<Vec<&'a PromptElement>, OovraError> {
        let mut out = Vec::with_capacity(self.order.len());
        for id in &self.order {
            let element = lib
                .get(id)
                .ok_or_else(|| OovraError::ElementNotFound { id: id.clone() })?;
            out.push(element);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn tempdir(name: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("oovra-canvas-{}-{}-{n}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Build a small olib with two atoms and return (olib_dir, library).
    fn two_atom_olib(dir: &Path) -> (PathBuf, Library) {
        let olib = dir.join("olib");
        fs::create_dir_all(&olib).unwrap();
        oovra::create::label_into_olib(&olib, "the role body", "role", "1.0.0", "").unwrap();
        oovra::create::label_into_olib(&olib, "the tone body", "tone", "1.0.0", "").unwrap();
        let lib = Library::load_with(&olib, oovra::ParseOptions::default()).unwrap();
        (olib, lib)
    }

    #[test]
    fn canvas_toggle_is_idempotent() {
        // U3-1
        let mut c = CanvasState::new();
        assert!(!c.contains("a"));
        c.toggle("a");
        assert_eq!(c.order, vec!["a"]);
        c.toggle("a");
        assert!(c.order.is_empty());

        c.toggle("a");
        c.toggle("b");
        c.toggle("c");
        assert_eq!(c.order, vec!["a", "b", "c"]);

        // Removing middle preserves the others' order.
        c.toggle("b");
        assert_eq!(c.order, vec!["a", "c"]);
    }

    #[test]
    fn canvas_add_and_remove_are_idempotent() {
        let mut c = CanvasState::new();
        assert!(c.add("a"));
        assert!(!c.add("a")); // second add is no-op
        assert_eq!(c.order, vec!["a"]);
        c.add("b");
        c.add("c");
        assert!(c.remove("b"));
        assert!(!c.remove("b")); // second remove is no-op
        assert_eq!(c.order, vec!["a", "c"]);
    }

    #[test]
    fn canvas_live_preview_matches_render_text() {
        // U3-2: live_preview equals the direct render_text call.
        let dir = tempdir("preview");
        let (_olib, lib) = two_atom_olib(&dir);

        let mut c = CanvasState::new();
        c.toggle("tone");
        c.toggle("role");
        let preview = c.live_preview(&lib).expect("preview should succeed");

        let direct =
            oovra::render::render_text(&[lib.get("tone").unwrap(), lib.get("role").unwrap()])
                .unwrap();
        assert_eq!(preview, direct);
    }

    #[test]
    fn canvas_save_as_compound_round_trips() {
        // U3-3: save produces a valid Compound file on disk.
        let dir = tempdir("save");
        let (olib, lib) = two_atom_olib(&dir);

        let mut c = CanvasState::new();
        c.toggle("role");
        c.toggle("tone");
        c.output_id = "coding-agent".to_string();

        let path = c
            .save_as_compound(&lib, &olib)
            .expect("save should succeed");
        assert_eq!(path, olib.join("coding-agent.md"));

        let parsed = oovra::parse_file(&path).unwrap();
        assert_eq!(parsed.header.id, "coding-agent");
        assert_eq!(parsed.header.kind, oovra::PromptElementKind::Compound);
        let inputs = parsed
            .header
            .composed_of
            .as_ref()
            .expect("compound has inputs");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].id, "role");
        assert_eq!(inputs[1].id, "tone");
        assert_eq!(parsed.header.body_level, Some(1));
    }

    #[test]
    fn canvas_live_preview_errors_on_missing_id() {
        // U3-4: canvas referencing an id absent from the library
        // surfaces an ElementNotFound error rather than silently
        // dropping or panicking.
        let dir = tempdir("missing");
        let (_olib, lib) = two_atom_olib(&dir);

        let mut c = CanvasState::new();
        c.toggle("role");
        c.toggle("ghost"); // doesn't exist in lib
        let err = c.live_preview(&lib).expect_err("preview must error");
        match err {
            OovraError::ElementNotFound { id } => assert_eq!(id, "ghost"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
