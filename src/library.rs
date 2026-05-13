//! Library: a collection of prompt elements loaded from a directory.
//!
//! v0.1 fails loud — one bad file aborts the load, duplicate IDs are an
//! error. The HashMap is keyed by ID; ordering of files in the library is
//! irrelevant (only ordering specified in a composition matters).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::element::{parse_file_with, ParseOptions, PromptElement};
use crate::error::{OovraError, Result};

#[derive(Debug)]
pub struct Library {
    pub root: PathBuf,
    pub elements: HashMap<String, PromptElement>,
}

impl Library {
    /// Walk `root` recursively in default (v0.2-only) mode, parse every `.md`
    /// file, return a populated library. Errors on duplicate IDs or any
    /// unparseable file.
    pub fn load(root: &Path) -> Result<Self> {
        Self::load_with(root, ParseOptions::default())
    }

    /// Same as [`Library::load`] but with explicit parse options. Use
    /// `ParseOptions { legacy: true }` to accept v0.1 schema files.
    pub fn load_with(root: &Path, opts: ParseOptions) -> Result<Self> {
        if !root.exists() {
            return Err(OovraError::FileNotFound(root.to_path_buf()));
        }
        if !root.is_dir() {
            return Err(OovraError::NotADirectory(root.to_path_buf()));
        }

        let mut elements: HashMap<String, PromptElement> = HashMap::new();
        let mut id_to_path: HashMap<String, PathBuf> = HashMap::new();

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !entry.file_type().is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let element = parse_file_with(path, opts)?;
            let id = element.header.id.clone();

            if let Some(existing) = id_to_path.get(&id) {
                return Err(OovraError::DuplicateId {
                    id: id.clone(),
                    first: existing.clone(),
                    second: path.to_path_buf(),
                });
            }

            id_to_path.insert(id.clone(), path.to_path_buf());
            elements.insert(id, element);
        }

        Ok(Library {
            root: root.to_path_buf(),
            elements,
        })
    }

    pub fn get(&self, id: &str) -> Option<&PromptElement> {
        self.elements.get(id)
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}
