//! Sprint s1 application: a file-explorer panel layered over the
//! sprint s0 bootstrap.
//!
//! Layout:
//!  - Top toolbar — `Open folder…` button (native: rfd; WASM: a stub
//!    that surfaces the deferred-feature message).
//!  - Left sidebar — list of olibs discovered under the opened folder.
//!  - Central panel — the selected olib's atoms, plus the s0 probe in
//!    a collapsing section (kept for sprint-over-sprint visual diff).
//!
//! All filesystem walking and parsing is done by the `oovra` library
//! crate; this file is purely a presenter over it (the CLAUDE.md
//! CLI-first rule in practice).

use std::path::{Path, PathBuf};

use oovra::header::{is_kebab_case, slugify};

/// Application state. Persisted across runs via eframe's persistence
/// feature, keyed by [`eframe::APP_KEY`]. Only `kebab_probe` and
/// `roots` are persisted; the discovery / loaded-library state is
/// recomputed each run.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OovraApp {
    /// Live input for the kebab-case / slugify probe (sprint s0).
    kebab_probe: String,
    /// Folders the user has opened in this app — kept for a future
    /// "recent folders" affordance.
    roots: Vec<PathBuf>,
    /// Discovered olibs under the most recently opened root.
    #[serde(skip)]
    discovered: Vec<oovra::DiscoveredOlib>,
    /// Index into `discovered` of the currently selected olib.
    #[serde(skip)]
    selected: Option<usize>,
    /// Parsed library for the selected olib.
    #[serde(skip)]
    loaded: Option<oovra::Library>,
    /// One-line status / error string surfaced under the toolbar.
    #[serde(skip)]
    status: String,
}

impl Default for OovraApp {
    fn default() -> Self {
        Self {
            kebab_probe: "My Draft".to_owned(),
            roots: Vec::new(),
            discovered: Vec::new(),
            selected: None,
            loaded: None,
            status: String::new(),
        }
    }
}

impl OovraApp {
    /// Build the app, optionally rehydrating from eframe's storage.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Self::default()
    }

    /// Discover olibs under `root`, replacing the current view.
    /// Only reachable on native — the WASM branch shows a "deferred"
    /// status instead of calling this — so the wasm32 build flags it
    /// as dead code without this gate.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    fn open_folder(&mut self, root: PathBuf) {
        match oovra::discovery::discover(&root, None) {
            Ok(d) => {
                self.status = format!("Discovered {} olib(s) under {}", d.len(), root.display());
                self.discovered = d;
                self.selected = None;
                self.loaded = None;
                if !self.roots.contains(&root) {
                    self.roots.push(root);
                }
            }
            Err(e) => {
                self.status = format!("Discover failed: {e}");
            }
        }
    }

    /// Load the library at `path` and stash it as `loaded`.
    fn load_selected(&mut self, idx: usize, path: &Path) {
        self.selected = Some(idx);
        match oovra::Library::load_with(path, oovra::ParseOptions::default()) {
            Ok(lib) => {
                self.status = format!(
                    "Loaded {}: {} element(s)",
                    path.display(),
                    lib.elements.len()
                );
                self.loaded = Some(lib);
            }
            Err(e) => {
                self.status = format!("Load failed: {e}");
                self.loaded = None;
            }
        }
    }

    /// Render the sidebar list of discovered olibs. Selecting one calls
    /// [`Self::load_selected`].
    fn render_olib_list(&mut self, ui: &mut egui::Ui) {
        // Snapshot the bits we need so we can mutate self inside the loop.
        let entries: Vec<(PathBuf, usize)> = self
            .discovered
            .iter()
            .map(|d| (d.path.clone(), d.md_count))
            .collect();
        for (i, (path, md_count)) in entries.into_iter().enumerate() {
            let is_sel = self.selected == Some(i);
            let label = format!("{}  ({} .md)", short_path(&path), md_count);
            if ui.selectable_label(is_sel, label).clicked() {
                self.load_selected(i, &path);
            }
        }
    }

    /// Render the selected olib's atoms in the central panel.
    fn render_atoms(&self, ui: &mut egui::Ui) {
        let Some(lib) = &self.loaded else {
            ui.weak("Select an olib in the sidebar to see its contents.");
            return;
        };
        ui.label(format!("{} element(s) in this olib:", lib.elements.len()));
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Stable display order, regardless of HashMap iteration order.
            let mut items: Vec<&oovra::PromptElement> = lib.elements.values().collect();
            items.sort_by(|a, b| a.header.id.cmp(&b.header.id));
            for e in items {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("• {}", e.header.id)).strong());
                    ui.weak(format!("[{:?}]", e.header.kind));
                    ui.weak(format!("v{}", e.header.version));
                    if !e.header.meta.is_empty() {
                        ui.weak(format!("— {}", e.header.meta));
                    }
                });
            }
        });
    }
}

impl eframe::App for OovraApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    // egui 0.34.2 deprecated the TopBottomPanel/SidePanel type aliases
    // and the `default_width` method in favor of a unified `Panel<Side>`
    // type and `default_size`. The migration is a follow-up sprint; for
    // now we use the still-functional aliases and silence the noise.
    #[allow(deprecated)]
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Top toolbar — Open folder + status line.
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

        // Left sidebar — discovered olibs.
        egui::SidePanel::left("olibs")
            .resizable(true)
            .default_width(260.0)
            .show_inside(ui, |ui| {
                ui.heading("olibs");
                ui.separator();
                if self.discovered.is_empty() {
                    ui.weak("(open a folder to discover olibs)");
                } else {
                    self.render_olib_list(ui);
                }
            });

        // Central panel — selected olib's atoms + the s0 probe.
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("oovra-gui — sprint s1");
            ui.label(format!(
                "Linked to oovra v{} via the workspace path-dep.",
                oovra::VERSION
            ));
            ui.separator();
            self.render_atoms(ui);
            ui.separator();
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
        // U-1 / U-2 from s0, still pinning the lib link at test time.
        let app = OovraApp::default();
        assert!(!app.kebab_probe.is_empty());
        assert!(oovra::header::is_kebab_case("my-id"));
        assert_eq!(
            oovra::header::slugify("My Draft").as_deref(),
            Some("my-draft")
        );

        // s1 additions: the new state fields start empty so a fresh
        // app shows the "(open a folder to discover olibs)" hint.
        assert!(app.discovered.is_empty());
        assert!(app.selected.is_none());
        assert!(app.loaded.is_none());
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
