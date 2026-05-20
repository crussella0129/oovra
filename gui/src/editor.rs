//! Editor: in-memory edit state for one prompt-element file.
//!
//! Sprint s2 only edits **atoms**; compounds open in a read-only
//! shown-but-not-editable state. The flow:
//!
//! 1. `Editor::open(path)` parses the file. If the element is an
//!    atom, returns `OpenResult::Loaded(Editor)`; if it's a
//!    compound, returns `OpenResult::CompoundReadOnly`.
//! 2. The host (`OovraApp`) renders the editor's fields and writes
//!    user edits back through `&mut` references; flipping `dirty`
//!    on `response.changed()`.
//! 3. `Editor::save()` validates and writes the in-memory state
//!    back via `oovra::write`, which itself re-validates the
//!    serialized form before any byte hits disk.

use std::path::{Path, PathBuf};

use oovra::{PromptElement, PromptElementHeader, PromptElementKind};

/// In-memory edit state for one atom.
#[derive(Debug)]
pub struct Editor {
    pub path: PathBuf,
    /// Identifier — read-only this sprint. Renames are a future
    /// "version / rename" sprint because they ripple into compose
    /// recipes that reference the id.
    pub id: String,
    pub name: String,
    pub version: String,
    pub meta: String,
    pub body: String,
    /// `true` while in-memory edits have not yet been saved.
    pub dirty: bool,
    /// One-line status / error string surfaced under the buttons.
    pub status: String,
}

/// Outcome of `Editor::open`.
#[derive(Debug)]
pub enum OpenResult {
    /// Editor loaded — atom is editable.
    Loaded(Editor),
    /// Element is a compound; the gui surfaces a "not editable yet"
    /// message instead of opening an editor.
    CompoundReadOnly { path: PathBuf, summary: String },
    /// Parse or IO error.
    Failed(String),
}

impl Editor {
    /// Try to open `path` as an editable atom.
    pub fn open(path: &Path) -> OpenResult {
        let element = match oovra::parse_file(path) {
            Ok(e) => e,
            Err(e) => return OpenResult::Failed(format!("parse failed: {e}")),
        };
        match element.header.kind {
            PromptElementKind::Atom => OpenResult::Loaded(Editor {
                path: path.to_path_buf(),
                id: element.header.id,
                name: element.header.name,
                version: element.header.version,
                meta: element.header.meta,
                body: element.body,
                dirty: false,
                status: format!("Loaded {}", path.display()),
            }),
            PromptElementKind::Compound => OpenResult::CompoundReadOnly {
                path: path.to_path_buf(),
                summary: format!(
                    "{} — compound with {} input(s); editing compounds is a later sprint",
                    element.header.id,
                    element
                        .header
                        .composed_of
                        .as_ref()
                        .map(|v| v.len())
                        .unwrap_or(0)
                ),
            },
        }
    }

    /// Save the editor's current state back to `self.path`.
    pub fn save(&mut self) -> Result<(), String> {
        let path = self.path.clone();
        self.save_to(&path)
    }

    /// Save to an arbitrary path. Public for tests; the GUI calls
    /// `save()` which targets `self.path`.
    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        if !oovra::header::is_valid_semver(&self.version) {
            let msg = format!("version {:?} is not valid semver", self.version);
            self.status = format!("Save rejected: {msg}");
            return Err(msg);
        }
        if self.name.trim().is_empty() {
            let msg = "name must not be empty".to_string();
            self.status = format!("Save rejected: {msg}");
            return Err(msg);
        }
        let header = PromptElementHeader {
            name: self.name.clone(),
            kind: PromptElementKind::Atom,
            id: self.id.clone(),
            version: self.version.clone(),
            meta: self.meta.clone(),
            generated_at: None,
            render_mode: None,
            body_level: None,
            depth: None,
            composed_of: None,
        };
        let element = PromptElement::new(header, self.body.clone());
        oovra::write(&element, path).map_err(|e| {
            let msg = format!("write failed: {e}");
            self.status = format!("Save failed: {msg}");
            msg
        })?;
        self.dirty = false;
        self.status = format!("Saved {}", path.display());
        Ok(())
    }

    /// Reload from disk, discarding in-memory edits.
    pub fn reload(&mut self) -> Result<(), String> {
        let element = oovra::parse_file(&self.path).map_err(|e| {
            let msg = format!("parse failed: {e}");
            self.status = format!("Reload failed: {msg}");
            msg
        })?;
        if !matches!(element.header.kind, PromptElementKind::Atom) {
            let msg = "element on disk is no longer an atom".to_string();
            self.status = format!("Reload aborted: {msg}");
            return Err(msg);
        }
        self.name = element.header.name;
        self.version = element.header.version;
        self.meta = element.header.meta;
        self.body = element.body;
        self.dirty = false;
        self.status = format!("Reloaded {}", self.path.display());
        Ok(())
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
            std::env::temp_dir().join(format!("oovra-editor-{}-{}-{n}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_atom(olib: &Path, id: &str, body: &str) -> PathBuf {
        fs::create_dir_all(olib).unwrap();
        oovra::create::label_into_olib(olib, body, id, "1.0.0", "test-atom").unwrap()
    }

    #[test]
    fn editor_round_trips_an_atom() {
        // U2-1: open -> edit -> save -> re-read produces the edited body.
        let dir = tempdir("round-trip");
        let olib = dir.join("olib");
        let atom_path = write_atom(&olib, "my-atom", "original body");

        let mut ed = match Editor::open(&atom_path) {
            OpenResult::Loaded(e) => e,
            OpenResult::CompoundReadOnly { .. } => panic!("expected Loaded, got CompoundReadOnly"),
            OpenResult::Failed(msg) => panic!("expected Loaded, got Failed: {msg}"),
        };
        assert_eq!(ed.id, "my-atom");
        assert_eq!(ed.body, "original body");
        assert!(!ed.dirty);

        ed.body = "edited body".to_string();
        ed.dirty = true;
        ed.save().expect("save should succeed");
        assert!(!ed.dirty, "save clears the dirty flag");

        let reread = oovra::parse_file(&atom_path).unwrap();
        assert_eq!(reread.body, "edited body");
        assert_eq!(reread.header.id, "my-atom"); // id unchanged
    }

    #[test]
    fn editor_refuses_to_open_a_compound() {
        // U2-2: a compound surfaces as CompoundReadOnly, not Loaded.
        let dir = tempdir("refuse-compound");
        let path = dir.join("composed.md");
        let content = concat!(
            "+++\n",
            "name = \"composed\"\n",
            "kind = \"compound\"\n",
            "id = \"composed\"\n",
            "version = \"1.0.0\"\n",
            "meta = \"\"\n",
            "generated_at = \"2026-05-20T00:00:00Z\"\n",
            "render_mode = \"markdown-h2\"\n",
            "body_level = 1\n",
            "composed_of = [{id = \"a\", version = \"1.0.0\"}]\n",
            "+++\n",
            "\n",
            "body\n",
        );
        fs::write(&path, content).unwrap();

        match Editor::open(&path) {
            OpenResult::CompoundReadOnly { .. } => {}
            OpenResult::Loaded(_) => panic!("editor should not load a compound"),
            OpenResult::Failed(msg) => panic!("unexpected parse failure: {msg}"),
        }
    }

    #[test]
    fn editor_rejects_invalid_semver_on_save() {
        // U2-3: save with bad semver errors; on-disk file is untouched.
        let dir = tempdir("invalid-semver");
        let olib = dir.join("olib");
        let atom_path = write_atom(&olib, "a", "body");

        let mut ed = match Editor::open(&atom_path) {
            OpenResult::Loaded(e) => e,
            _ => panic!("setup: expected Loaded"),
        };
        ed.version = "not-a-version".to_string();
        ed.dirty = true;
        let r = ed.save();
        assert!(r.is_err(), "save must reject invalid semver");

        let reread = oovra::parse_file(&atom_path).unwrap();
        assert_eq!(
            reread.header.version, "1.0.0",
            "on-disk file must be intact after a rejected save"
        );
    }
}
