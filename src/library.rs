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

    /// Build a deduplicated forest for the GUI's "schema menu" tree
    /// view. Each id appears AT MOST ONCE in the entire returned set
    /// of trees. Traversal is depth-first pre-order starting from
    /// [`Library::roots`]; a child that's already been visited
    /// earlier in the walk is omitted from its parent's `children`
    /// list (this is the fix for the "correlated list items" bug
    /// where an atom referenced by two compounds in the same recipe
    /// DAG was being rendered twice with synchronized checkboxes).
    ///
    /// This does NOT change the underlying recipe — each compound's
    /// `composed_of` is unchanged on disk; this is purely a display
    /// projection.
    pub fn component_tree(&self) -> Vec<ComponentNode> {
        let mut seen: HashSet<String> = HashSet::new();
        self.roots()
            .into_iter()
            .filter_map(|id| self.build_component_node(&id, &mut seen))
            .collect()
    }

    fn build_component_node(&self, id: &str, seen: &mut HashSet<String>) -> Option<ComponentNode> {
        if !seen.insert(id.to_string()) {
            return None;
        }
        let elem = self.elements.get(id)?;
        let children: Vec<ComponentNode> = elem
            .header
            .composed_of
            .as_ref()
            .map(|inputs| {
                inputs
                    .iter()
                    .filter_map(|i| self.build_component_node(&i.id, seen))
                    .collect()
            })
            .unwrap_or_default();
        Some(ComponentNode {
            id: id.to_string(),
            kind: elem.header.kind,
            children,
        })
    }
}

/// One node of the deduplicated [`Library::component_tree`] forest.
/// `children` lists the deduped sub-nodes in declared `composed_of`
/// order; nodes whose ids appeared earlier in the walk are absent.
#[derive(Debug, Clone)]
pub struct ComponentNode {
    pub id: String,
    pub kind: PromptElementKind,
    pub children: Vec<ComponentNode>,
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

    /// Recursive count following raw `composed_of` (no dedup). This is
    /// the SHAPE of the old GUI tree renderer's walk. Used to
    /// document the bug.
    fn naive_walk_count_id(lib: &Library, id: &str, target: &str) -> usize {
        let Some(elem) = lib.elements.get(id) else {
            return 0;
        };
        let mut count = if id == target { 1 } else { 0 };
        if let Some(inputs) = &elem.header.composed_of {
            for input in inputs {
                count += naive_walk_count_id(lib, &input.id, target);
            }
        }
        count
    }

    /// Count occurrences of `target` in the deduped `component_tree`.
    fn deduped_tree_count_id(forest: &[ComponentNode], target: &str) -> usize {
        let mut count = 0;
        for node in forest {
            if node.id == target {
                count += 1;
            }
            count += deduped_tree_count_id(&node.children, target);
        }
        count
    }

    /// Build the mock library's diamond shape: an atom A, a compound
    /// `c-inner` containing `[a]`, and a compound `c-outer`
    /// containing `[c-inner, a]`. `a` appears at two depths.
    fn diamond_lib(dir: &Path) -> Library {
        let olib = dir.join("olib");
        fs::create_dir_all(&olib).unwrap();
        crate::create::label_into_olib(&olib, "a body", "a", "1.0.0", "").unwrap();
        let lib = Library::load(&olib).unwrap();
        let c_inner = crate::render::compose(crate::render::ComposeRequest {
            library: &lib,
            inputs: vec![("a".to_owned(), Some("1.0.0".to_owned()))],
            output_id: "c-inner".to_owned(),
            output_name: "c-inner".to_owned(),
            output_version: "1.0.0".to_owned(),
            output_meta: String::new(),
        })
        .unwrap();
        crate::element::write(&c_inner, &olib.join("c-inner.md")).unwrap();
        let lib = Library::load(&olib).unwrap();
        let c_outer = crate::render::compose(crate::render::ComposeRequest {
            library: &lib,
            inputs: vec![
                ("c-inner".to_owned(), Some("1.0.0".to_owned())),
                ("a".to_owned(), Some("1.0.0".to_owned())),
            ],
            output_id: "c-outer".to_owned(),
            output_name: "c-outer".to_owned(),
            output_version: "1.0.0".to_owned(),
            output_meta: String::new(),
        })
        .unwrap();
        crate::element::write(&c_outer, &olib.join("c-outer.md")).unwrap();
        Library::load(&olib).unwrap()
    }

    #[test]
    fn diagnostic_naive_walk_double_counts_diamond_leaf() {
        // BUG, captured: walking `composed_of` raw (the shape of the
        // pre-fix GUI tree renderer) makes the diamond leaf appear
        // TWICE under c-outer — once via c-outer.composed_of[1]
        // (the direct `a`), and once via c-inner.composed_of[0]
        // (the nested `a`). Synchronized checkboxes follow.
        let dir = tempdir("diagnostic-naive");
        let lib = diamond_lib(&dir);
        assert_eq!(naive_walk_count_id(&lib, "c-outer", "a"), 2);
    }

    #[test]
    fn component_tree_dedupes_diamond_leaf() {
        // FIX: the deduped tree shows `a` exactly once, at its
        // first-encountered position (depth 2, under c-inner, which
        // is c-outer's first child).
        let dir = tempdir("component-tree-diamond");
        let lib = diamond_lib(&dir);
        let forest = lib.component_tree();
        // Forest root count: c-outer is the only root (c-inner and a
        // are both inputs of something).
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].id, "c-outer");
        // `a` appears exactly once in the entire forest.
        assert_eq!(deduped_tree_count_id(&forest, "a"), 1);
        // c-outer has exactly one child rendered (c-inner). The
        // duplicate `a` at c-outer's depth is suppressed.
        assert_eq!(forest[0].children.len(), 1);
        assert_eq!(forest[0].children[0].id, "c-inner");
        // `a` lives inside c-inner.
        assert_eq!(forest[0].children[0].children.len(), 1);
        assert_eq!(forest[0].children[0].children[0].id, "a");
    }

    #[test]
    fn component_tree_keeps_non_diamond_children_in_order() {
        // Regression: for the simple "c contains a, b" case (no
        // diamond), both children render in their declared order.
        let dir = tempdir("component-tree-simple");
        let lib = small_lib(&dir);
        let forest = lib.component_tree();
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].id, "c");
        let child_ids: Vec<&str> = forest[0].children.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(child_ids, vec!["a", "b"]);
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
