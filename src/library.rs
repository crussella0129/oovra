//! Library: a collection of prompt elements loaded from a directory.
//!
//! v0.1 fails loud — one bad file aborts the load, duplicate IDs are an
//! error. The HashMap is keyed by ID; ordering of files in the library is
//! irrelevant (only ordering specified in a composition matters).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::element::{parse_file_with, ParseOptions, PromptElement};
use crate::error::{OovraError, Result};
use crate::header::PromptElementKind;

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

    /// Top-level component ids: every element in this library whose id
    /// is NOT referenced in the `composed_of` of any other element.
    /// These are the natural "roots" of the recipe DAG and the right
    /// set to show at the top of a schema-style tree (sub-components
    /// appear only inside their parent compound's dropdown).
    /// Returned sorted by id for deterministic display.
    pub fn roots(&self) -> Vec<String> {
        let mut input_ids: HashSet<&str> = HashSet::new();
        for elem in self.elements.values() {
            if let Some(inputs) = &elem.header.composed_of {
                for input in inputs {
                    input_ids.insert(input.id.as_str());
                }
            }
        }
        let mut roots: Vec<String> = self
            .elements
            .keys()
            .filter(|id| !input_ids.contains(id.as_str()))
            .cloned()
            .collect();
        roots.sort();
        roots
    }

    /// Transitive descendants of `id` in pre-order, deduplicated.
    /// Walks the `composed_of` graph from `id` down. Returns an empty
    /// vec if `id` is an atom, missing, or has no inputs.
    pub fn descendants(&self, id: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        self.collect_descendants(id, &mut out, &mut seen);
        out
    }

    fn collect_descendants(&self, id: &str, out: &mut Vec<String>, seen: &mut HashSet<String>) {
        let Some(elem) = self.elements.get(id) else {
            return;
        };
        let Some(inputs) = &elem.header.composed_of else {
            return;
        };
        for input in inputs {
            if seen.insert(input.id.clone()) {
                out.push(input.id.clone());
                self.collect_descendants(&input.id, out, seen);
            }
        }
    }

    /// Atom-only leaves reachable from `id` (recursive). If `id` is an
    /// atom, returns `[id]`. If `id` is a compound, returns the
    /// flat set of atom ids under its recipe tree, deduplicated and
    /// sorted. This is the set used by the GUI's "select all sub
    /// items" cascade: the canvas ends up holding only atoms, so
    /// composing it produces a flat compound and rendering doesn't
    /// duplicate content via nested compounds.
    pub fn leaf_atoms(&self, id: &str) -> Vec<String> {
        let mut out: HashSet<String> = HashSet::new();
        self.collect_leaf_atoms(id, &mut out);
        let mut v: Vec<String> = out.into_iter().collect();
        v.sort();
        v
    }

    fn collect_leaf_atoms(&self, id: &str, out: &mut HashSet<String>) {
        let Some(elem) = self.elements.get(id) else {
            return;
        };
        match elem.header.kind {
            PromptElementKind::Atom => {
                out.insert(id.to_string());
            }
            PromptElementKind::Compound => {
                if let Some(inputs) = &elem.header.composed_of {
                    for input in inputs {
                        self.collect_leaf_atoms(&input.id, out);
                    }
                }
            }
        }
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
            std::env::temp_dir().join(format!("oovra-library-{}-{}-{n}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Build an olib with two atoms (A, B) and one compound C(A, B);
    /// load and return the Library.
    fn small_lib(dir: &Path) -> Library {
        let olib = dir.join("olib");
        fs::create_dir_all(&olib).unwrap();
        crate::create::label_into_olib(&olib, "a body", "a", "1.0.0", "").unwrap();
        crate::create::label_into_olib(&olib, "b body", "b", "1.0.0", "").unwrap();
        let lib = Library::load(&olib).unwrap();
        let c = crate::render::compose(crate::render::ComposeRequest {
            library: &lib,
            inputs: vec![
                ("a".to_owned(), Some("1.0.0".to_owned())),
                ("b".to_owned(), Some("1.0.0".to_owned())),
            ],
            output_id: "c".to_owned(),
            output_name: "c".to_owned(),
            output_version: "1.0.0".to_owned(),
            output_meta: String::new(),
        })
        .unwrap();
        crate::element::write(&c, &olib.join("c.md")).unwrap();
        Library::load(&olib).unwrap()
    }

    #[test]
    fn roots_excludes_components_that_are_inputs() {
        let dir = tempdir("roots");
        let lib = small_lib(&dir);
        // C contains A and B; only C is a root.
        assert_eq!(lib.roots(), vec!["c"]);
    }

    #[test]
    fn descendants_walks_recipe_graph() {
        let dir = tempdir("descendants");
        let lib = small_lib(&dir);
        let mut d = lib.descendants("c");
        d.sort();
        assert_eq!(d, vec!["a", "b"]);
        assert!(lib.descendants("a").is_empty()); // atom has none
    }

    #[test]
    fn leaf_atoms_returns_only_atoms() {
        let dir = tempdir("leaf-atoms");
        let lib = small_lib(&dir);
        assert_eq!(lib.leaf_atoms("c"), vec!["a", "b"]);
        // For an atom, the function returns itself.
        assert_eq!(lib.leaf_atoms("a"), vec!["a"]);
    }

    #[test]
    fn leaf_atoms_dedupes_across_a_diamond() {
        // C1 = (A); C2 = (A, C1); leaf_atoms("C2") = [a] (not [a, a]).
        let dir = tempdir("diamond");
        let olib = dir.join("olib");
        fs::create_dir_all(&olib).unwrap();
        crate::create::label_into_olib(&olib, "a body", "a", "1.0.0", "").unwrap();
        let lib = Library::load(&olib).unwrap();
        let c1 = crate::render::compose(crate::render::ComposeRequest {
            library: &lib,
            inputs: vec![("a".to_owned(), Some("1.0.0".to_owned()))],
            output_id: "c1".to_owned(),
            output_name: "c1".to_owned(),
            output_version: "1.0.0".to_owned(),
            output_meta: String::new(),
        })
        .unwrap();
        crate::element::write(&c1, &olib.join("c1.md")).unwrap();
        let lib = Library::load(&olib).unwrap();
        let c2 = crate::render::compose(crate::render::ComposeRequest {
            library: &lib,
            inputs: vec![
                ("a".to_owned(), Some("1.0.0".to_owned())),
                ("c1".to_owned(), Some("1.0.0".to_owned())),
            ],
            output_id: "c2".to_owned(),
            output_name: "c2".to_owned(),
            output_version: "1.0.0".to_owned(),
            output_meta: String::new(),
        })
        .unwrap();
        crate::element::write(&c2, &olib.join("c2.md")).unwrap();
        let lib = Library::load(&olib).unwrap();
        assert_eq!(lib.leaf_atoms("c2"), vec!["a"]); // deduped
        assert_eq!(lib.roots(), vec!["c2"]); // c1, a are inputs to c2
    }
}
