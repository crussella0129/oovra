//! Sprint s2 application: file-explorer panel from s1, plus an
//! embedded oovra-particle editor.
//!
//! Layout:
//!  - Top toolbar — `Open folder…` (native rfd; WASM: deferred stub).
//!  - Left sidebar — olib list (discovered under the opened folder).
//!  - Second-left sidebar — atom list (contents of the selected olib).
//!  - Central panel — the editor, or a hint when no atom is selected.
//!  - Bottom panel — s0 kebab/slugify probe in a collapsing section.
//!
//! All filesystem / parsing / writing work goes through the `oovra`
//! library; this file is a presenter (CLI-first per CLAUDE.md).

use std::path::{Path, PathBuf};

use oovra::header::{is_kebab_case, slugify};

use crate::editor::{Editor, OpenResult};

/// Application state. Persisted across runs via eframe's persistence
/// feature, keyed by [`eframe::APP_KEY`]. Only the lightweight
/// fields persist; discovery + editor state is recomputed each run.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OovraApp {
    /// Live input for the kebab-case / slugify probe (sprint s0).
    kebab_probe: String,
    /// Folders the user has opened in this app.
    roots: Vec<PathBuf>,

    /// Discovered olibs under the most recently opened root.
    #[serde(skip)]
    discovered: Vec<oovra::DiscoveredOlib>,
    /// Index into `discovered` of the currently selected olib.
    #[serde(skip)]
    selected_olib: Option<usize>,
    /// Parsed library for the selected olib.
    #[serde(skip)]
    loaded: Option<oovra::Library>,

    /// Snapshot of (id, source_path) tuples from the loaded library,
    /// in stable sorted order — drives the atoms-list rendering and
    /// "click atom -> open editor" lookup without re-querying the
    /// library each frame.
    #[serde(skip)]
    atom_index: Vec<AtomEntry>,
    /// Index into `atom_index` of the currently selected atom.
    #[serde(skip)]
    selected_atom: Option<usize>,
    /// The active editor, if the selection landed on an atom.
    #[serde(skip)]
    editor: Option<Editor>,
    /// Message shown when the selection is a compound (not editable
    /// yet) — distinct from `editor` so the central panel knows what
    /// to render.
    #[serde(skip)]
    compound_msg: Option<String>,

    /// One-line status / error string surfaced under the toolbar.
    #[serde(skip)]
    status: String,
}

/// Lightweight per-frame entry for the atoms list. The full
/// [`oovra::PromptElement`] lives in `loaded`; this snapshot keeps
/// the list rendering simple and decoupled from the library map's
/// iteration order.
#[derive(Debug, Clone)]
struct AtomEntry {
    id: String,
    kind: oovra::PromptElementKind,
    path: PathBuf,
}

impl Default for OovraApp {
    fn default() -> Self {
        Self {
            kebab_probe: "My Draft".to_owned(),
            roots: Vec::new(),
            discovered: Vec::new(),
            selected_olib: None,
            loaded: None,
            atom_index: Vec::new(),
            selected_atom: None,
            editor: None,
            compound_msg: None,
            status: String::new(),
        }
    }
}

impl OovraApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Self::default()
    }

    /// Discover olibs under `root`, replacing the current view. Also
    /// clears any active editor / atom selection — opening a new
    /// folder starts a fresh session.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    fn open_folder(&mut self, root: PathBuf) {
        match oovra::discovery::discover(&root, None) {
            Ok(d) => {
                self.status = format!("Discovered {} olib(s) under {}", d.len(), root.display());
                self.discovered = d;
                self.clear_olib_selection();
                if !self.roots.contains(&root) {
                    self.roots.push(root);
                }
            }
            Err(e) => {
                self.status = format!("Discover failed: {e}");
            }
        }
    }

    fn clear_olib_selection(&mut self) {
        self.selected_olib = None;
        self.loaded = None;
        self.atom_index.clear();
        self.clear_atom_selection();
    }

    fn clear_atom_selection(&mut self) {
        self.selected_atom = None;
        self.editor = None;
        self.compound_msg = None;
    }

    /// Load the library at `path`; rebuild the atom index.
    fn load_selected_olib(&mut self, idx: usize, path: &Path) {
        self.selected_olib = Some(idx);
        self.clear_atom_selection();
        match oovra::Library::load_with(path, oovra::ParseOptions::default()) {
            Ok(lib) => {
                let mut idx: Vec<AtomEntry> = lib
                    .elements
                    .values()
                    .map(|e| AtomEntry {
                        id: e.header.id.clone(),
                        kind: e.header.kind,
                        path: e.source_path.clone().unwrap_or_default(),
                    })
                    .collect();
                idx.sort_by(|a, b| a.id.cmp(&b.id));
                self.status = format!("Loaded {}: {} element(s)", path.display(), idx.len());
                self.atom_index = idx;
                self.loaded = Some(lib);
            }
            Err(e) => {
                self.loaded = None;
                self.atom_index.clear();
                self.status = format!("Load failed: {e}");
            }
        }
    }

    /// React to a click on an atom list entry: try to open it in the
    /// editor; show a message if it's a compound.
    fn select_atom(&mut self, idx: usize) {
        self.selected_atom = Some(idx);
        self.editor = None;
        self.compound_msg = None;
        let Some(entry) = self.atom_index.get(idx) else {
            return;
        };
        let path = entry.path.clone();
        match Editor::open(&path) {
            OpenResult::Loaded(ed) => {
                self.status = ed.status.clone();
                self.editor = Some(ed);
            }
            OpenResult::CompoundReadOnly { summary, .. } => {
                self.status = summary.clone();
                self.compound_msg = Some(summary);
            }
            OpenResult::Failed(msg) => {
                self.status = format!("Open failed: {msg}");
            }
        }
    }

    fn render_olib_list(&mut self, ui: &mut egui::Ui) {
        let entries: Vec<(PathBuf, usize)> = self
            .discovered
            .iter()
            .map(|d| (d.path.clone(), d.md_count))
            .collect();
        for (i, (path, md_count)) in entries.into_iter().enumerate() {
            let is_sel = self.selected_olib == Some(i);
            let label = format!("{}  ({} .md)", short_path(&path), md_count);
            if ui.selectable_label(is_sel, label).clicked() {
                self.load_selected_olib(i, &path);
            }
        }
    }

    fn render_atom_list(&mut self, ui: &mut egui::Ui) {
        if self.atom_index.is_empty() {
            ui.weak("(no library loaded — select an olib in the sidebar)");
            return;
        }
        // Snapshot for the borrow.
        let entries: Vec<(String, oovra::PromptElementKind)> = self
            .atom_index
            .iter()
            .map(|a| (a.id.clone(), a.kind))
            .collect();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, (id, kind)) in entries.into_iter().enumerate() {
                let is_sel = self.selected_atom == Some(i);
                let glyph = match kind {
                    oovra::PromptElementKind::Atom => "·",
                    oovra::PromptElementKind::Compound => "▣",
                };
                let label = format!("{glyph}  {id}");
                if ui.selectable_label(is_sel, label).clicked() {
                    self.select_atom(i);
                }
            }
        });
    }

    fn render_editor(&mut self, ui: &mut egui::Ui) {
        // Three states: editor loaded; compound (read-only msg); nothing.
        if let Some(ed) = &mut self.editor {
            ui.heading("editor");
            ui.add_space(2.0);
            egui::Grid::new("editor_fields")
                .num_columns(2)
                .min_col_width(80.0)
                .show(ui, |ui| {
                    ui.weak("id");
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut ed.id.clone()));
                    ui.end_row();

                    ui.weak("name");
                    if ui.text_edit_singleline(&mut ed.name).changed() {
                        ed.dirty = true;
                    }
                    ui.end_row();

                    ui.weak("version");
                    if ui.text_edit_singleline(&mut ed.version).changed() {
                        ed.dirty = true;
                    }
                    ui.end_row();

                    ui.weak("meta");
                    if ui.text_edit_singleline(&mut ed.meta).changed() {
                        ed.dirty = true;
                    }
                    ui.end_row();
                });

            ui.separator();
            ui.label("body:");
            let body_resp = ui.add(
                egui::TextEdit::multiline(&mut ed.body)
                    .desired_width(f32::INFINITY)
                    .desired_rows(18)
                    .font(egui::TextStyle::Monospace),
            );
            if body_resp.changed() {
                ed.dirty = true;
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let save_label = if ed.dirty { "Save *" } else { "Save" };
                if ui
                    .add_enabled(ed.dirty, egui::Button::new(save_label))
                    .clicked()
                {
                    let _ = ed.save();
                }
                if ui.button("Reload").clicked() {
                    let _ = ed.reload();
                }
                ui.separator();
                ui.weak(&ed.status);
            });
            return;
        }
        if let Some(msg) = &self.compound_msg {
            ui.heading("compound — read-only");
            ui.add_space(4.0);
            ui.weak(msg);
            ui.add_space(8.0);
            ui.weak("Editing compounds is a future sprint; for now, edit the recipe by re-composing from the CLI.");
            return;
        }
        ui.weak("Select an atom in the middle column to open it in the editor.");
    }
}

impl eframe::App for OovraApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    // egui 0.34.2 panel-alias deprecations are tracked for a follow-up
    // migration sprint; carry the local allow forward unchanged.
    #[allow(deprecated)]
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Top toolbar
        egui::TopBottomPanel::top("toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open folder…").clicked() {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.open_folder(folder);
                        }
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        self.status =
                            "Folder picking via WASM is wired in a later sprint.".to_owned();
                    }
                }
                if !self.status.is_empty() {
                    ui.separator();
                    ui.weak(&self.status);
                }
            });
        });

        // Bottom: s0 probe in a collapsing section (kept for sprint-over-sprint diff)
        egui::TopBottomPanel::bottom("footer").show_inside(ui, |ui| {
            ui.collapsing("s0 probe (kebab / slugify)", |ui| {
                ui.label("Live oovra::header probe — exercising the lib at runtime:");
                ui.horizontal(|ui| {
                    ui.label("filename stem:");
                    ui.text_edit_singleline(&mut self.kebab_probe);
                });
                ui.label(format!(
                    "  is_kebab_case = {}",
                    is_kebab_case(&self.kebab_probe)
                ));
                ui.label(format!(
                    "  slugify       = {:?}",
                    slugify(&self.kebab_probe)
                ));
            });
        });

        // Left sidebar: discovered olibs
        egui::SidePanel::left("olibs")
            .resizable(true)
            .default_width(220.0)
            .show_inside(ui, |ui| {
                ui.heading("olibs");
                ui.separator();
                if self.discovered.is_empty() {
                    ui.weak("(open a folder to discover olibs)");
                } else {
                    self.render_olib_list(ui);
                }
            });

        // Second left sidebar: atoms in the selected olib
        egui::SidePanel::left("atoms")
            .resizable(true)
            .default_width(220.0)
            .show_inside(ui, |ui| {
                ui.heading("atoms");
                ui.separator();
                self.render_atom_list(ui);
            });

        // Central: the editor (or hint)
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label(format!(
                "oovra-gui — sprint s2  ·  linked to oovra v{}",
                oovra::VERSION
            ));
            ui.separator();
            self.render_editor(ui);
        });
    }
}

/// Short, human-readable label for a path: the last two-or-three
/// components joined with `/`, prefixed with `…` when truncated.
fn short_path(p: &Path) -> String {
    let comps: Vec<_> = p.components().rev().take(3).collect();
    let mut parts: Vec<String> = comps
        .iter()
        .rev()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if comps.len() == 3 {
        parts.insert(0, "…".to_owned());
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_app_is_constructible_and_oovra_lib_is_reachable() {
        let app = OovraApp::default();
        assert!(!app.kebab_probe.is_empty());
        assert!(oovra::header::is_kebab_case("my-id"));
        assert_eq!(
            oovra::header::slugify("My Draft").as_deref(),
            Some("my-draft")
        );
        assert!(app.discovered.is_empty());
        assert!(app.selected_olib.is_none());
        assert!(app.loaded.is_none());
        assert!(app.atom_index.is_empty());
        assert!(app.selected_atom.is_none());
        assert!(app.editor.is_none());
        assert!(app.compound_msg.is_none());
        assert!(app.status.is_empty());
    }

    #[test]
    fn short_path_truncates_with_ellipsis() {
        let p = std::path::Path::new("/a/b/c/d/olib");
        let s = short_path(p);
        assert!(s.starts_with("…/"), "expected ellipsis prefix, got {s}");
        assert!(s.ends_with("olib"));
    }

    #[test]
    fn short_path_short_paths_are_not_prefixed() {
        let p = std::path::Path::new("a/olib");
        let s = short_path(p);
        assert!(
            !s.starts_with("…"),
            "short path should not be prefixed: {s}"
        );
        assert!(s.ends_with("olib"));
    }
}
