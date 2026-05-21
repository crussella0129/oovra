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

use oovra::diff::DiffReport;
use oovra::header::{is_kebab_case, slugify};

use crate::canvas::CanvasState;
use crate::compare::CompareState;
use crate::editor::{Editor, OpenResult};

/// Which view is active in the central panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum CentralView {
    #[default]
    Editor,
    Canvas,
    Compare,
}

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

    /// Sorted id snapshot of the loaded library. Used to map
    /// `selected_atom: Option<usize>` to an id (the editor's
    /// click-highlight tracking) and to know whether the library is
    /// empty without re-reading `self.loaded`. The tree renderer
    /// resolves paths and kinds directly via `self.loaded` per row,
    /// so this list doesn't need to carry the path anymore.
    #[serde(skip)]
    atom_index: Vec<String>,
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

    /// Working set for the autocompose canvas (sprint s3).
    #[serde(skip)]
    canvas: CanvasState,
    /// Working set for the Compare view (sprint s4).
    #[serde(skip)]
    compare: CompareState,
    /// Which tab is showing in the central panel.
    #[serde(skip)]
    view: CentralView,

    /// True while a save-confirm dialog is up (sprint s5).
    #[serde(skip)]
    save_confirm_pending: bool,
    /// Force every CollapsingHeader in the components tree to this
    /// state on the next frame, then clear (sprint s5). Drives the
    /// Expand all / Collapse all buttons.
    #[serde(skip)]
    pending_open: Option<bool>,
    /// Currently selected bump kind for "Save As New Version"
    /// (sprint s5).
    #[serde(skip)]
    new_version_bump: oovra::header::BumpKind,

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
            selected_olib: None,
            loaded: None,
            atom_index: Vec::new(),
            selected_atom: None,
            editor: None,
            compound_msg: None,
            canvas: CanvasState::new(),
            compare: CompareState::new(),
            view: CentralView::Editor,
            save_confirm_pending: false,
            pending_open: None,
            new_version_bump: oovra::header::BumpKind::default(),
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
        // The Compare picks reference ids in the previous olib — clear
        // them so the picker doesn't show stale selections that no
        // longer resolve.
        self.compare = CompareState::new();
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
                let mut ids: Vec<String> = lib.elements.keys().cloned().collect();
                ids.sort();
                self.status = format!("Loaded {}: {} element(s)", path.display(), ids.len());
                self.atom_index = ids;
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
    /// Open a library component in the editor by id. Works for any
    /// id, including atoms nested inside compounds; the tree-view
    /// recursive renderer calls this so it doesn't need to know
    /// which atom_index position a deep atom is at.
    fn select_component_by_id(&mut self, id: &str) {
        self.editor = None;
        self.compound_msg = None;
        let Some(elem) = self.loaded.as_ref().and_then(|l| l.elements.get(id)) else {
            return;
        };
        let Some(path) = elem.source_path.clone() else {
            return;
        };
        self.selected_atom = self.atom_index.iter().position(|a| a == id);
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

    /// Render the Library Components as a recursive tree, schema-style.
    ///
    /// Backing data is [`oovra::Library::component_tree`], which gives
    /// a forest with each id appearing AT MOST ONCE — top-level is
    /// just the recipe-DAG roots, and within the tree a component
    /// that shows up at multiple paths is rendered at its first
    /// encounter (depth-first pre-order). This fixes the
    /// "correlated list items" issue: two visual instances of the
    /// same id with synchronized checkboxes.
    ///
    /// Atoms render as leaf rows. Compounds render as
    /// `CollapsingHeader`s containing their (deduped) children.
    /// Each row keeps the two-click-target pattern: the left
    /// checkbox controls canvas inclusion (with leaf-cascade for
    /// compounds); the row body opens the component in the editor.
    fn render_component_tree(&mut self, ui: &mut egui::Ui) {
        if self.atom_index.is_empty() {
            ui.weak("(no olib selected — pick one on the left)");
            return;
        }
        let forest: Vec<oovra::ComponentNode> = self
            .loaded
            .as_ref()
            .map(|lib| lib.component_tree())
            .unwrap_or_default();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for node in &forest {
                self.render_component_node(ui, node, 0);
            }
        });
        // Whichever expand/collapse-all the user requested was applied
        // this frame; reset so manual toggles work going forward.
        self.pending_open = None;
    }

    /// Render one node of the component tree. Recurses into a
    /// compound's `composed_of` inputs.
    /// Render a single deduplicated tree node. The recursive walk
    /// is over `node.children` — already deduped by the library —
    /// instead of raw `composed_of`.
    fn render_component_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &oovra::ComponentNode,
        depth: u32,
    ) {
        // Defensive depth cap; real recipes are nowhere near this.
        if depth > 16 {
            ui.weak("(depth limit reached)");
            return;
        }
        // Pull cascade leaves + the header tags (version / meta) for
        // this node in one scoped borrow of self.loaded. Cascade
        // leaves derive from the *underlying* library (not the
        // deduped display tree) so a compound's checkbox reflects
        // every leaf in its real recipe.
        let (leaves, version, meta): (Vec<String>, String, String) = {
            let lib = self.loaded.as_ref();
            let leaves = match node.kind {
                oovra::PromptElementKind::Atom => vec![node.id.clone()],
                oovra::PromptElementKind::Compound => {
                    lib.map(|l| l.leaf_atoms(&node.id)).unwrap_or_default()
                }
            };
            let (version, meta) = lib
                .and_then(|l| l.get(&node.id))
                .map(|e| (e.header.version.clone(), e.header.meta.clone()))
                .unwrap_or_default();
            (leaves, version, meta)
        };

        // Editor-selection highlight: which row matches the open
        // editor's id (if any).
        let is_sel = self
            .editor
            .as_ref()
            .map(|ed| ed.id == node.id)
            .unwrap_or(false);
        let id = node.id.clone();

        match node.kind {
            oovra::PromptElementKind::Atom => {
                ui.horizontal(|ui| {
                    let mut included = self.canvas.contains(&id);
                    if ui.checkbox(&mut included, "").changed() {
                        self.canvas.toggle(&id);
                    }
                    let label = format!("·  {id}");
                    if ui.selectable_label(is_sel, label).clicked() {
                        self.select_component_by_id(&id);
                    }
                    tag_labels(ui, &version, &meta);
                });
            }
            oovra::PromptElementKind::Compound => {
                ui.horizontal(|ui| {
                    let any_leaf_in_canvas = leaves.iter().any(|d| self.canvas.contains(d));
                    let mut shown = any_leaf_in_canvas;
                    if ui.checkbox(&mut shown, "").changed() {
                        if shown {
                            for leaf in &leaves {
                                self.canvas.add(leaf);
                            }
                        } else {
                            for leaf in &leaves {
                                self.canvas.remove(leaf);
                            }
                        }
                    }
                    let pending = self.pending_open;
                    let mut header = egui::CollapsingHeader::new(format!("▣  {id}"))
                        .id_salt(format!("tree-{depth}-{id}"));
                    if let Some(open) = pending {
                        header = header.open(Some(open));
                    }
                    let resp = header.show(ui, |ui| {
                        for child in &node.children {
                            self.render_component_node(ui, child, depth + 1);
                        }
                    });
                    // Tags trail the collapsing header on the same row.
                    let _ = resp;
                    tag_labels(ui, &version, &meta);
                });
            }
        }
    }

    /// Render the autocompose canvas: ordered (drag-reorderable) list
    /// of selected components, live render_text preview, and a
    /// save-as-compound form.
    fn render_canvas(&mut self, ui: &mut egui::Ui) {
        ui.label(format!(
            "Canvas — {} component(s) selected",
            self.canvas.order.len()
        ));
        ui.add_space(4.0);

        if self.canvas.order.is_empty() {
            ui.weak(
                "Check Library Components in the middle column to add \
                 them to the canvas.",
            );
            return;
        }

        ui.label("Order (drag ≡ to rearrange):");
        egui_dnd::dnd(ui, "canvas_list").show_vec(
            &mut self.canvas.order,
            |ui, id, handle, _state| {
                ui.horizontal(|ui| {
                    handle.ui(ui, |ui| {
                        ui.label("≡");
                    });
                    ui.label(format!("· {id}"));
                });
            },
        );

        ui.separator();
        ui.label("Live preview:");
        let preview = match &self.loaded {
            Some(lib) => self
                .canvas
                .live_preview(lib)
                .unwrap_or_else(|e| format!("preview error: {e}")),
            None => "(no olib loaded — select one to preview)".to_owned(),
        };
        // Render the rendered prompt as ordinary wrapped text — same
        // font as the rest of the UI — rather than a bordered
        // monospace TextEdit (which read as a terminal panel and
        // visually broke from the surrounding layout). Selectable so
        // the user can still copy the rendered prompt out.
        egui::ScrollArea::vertical()
            .max_height(260.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(egui::Label::new(&preview).selectable(true));
            });

        ui.separator();
        ui.label("Save as compound into the active olib:");
        ui.horizontal(|ui| {
            ui.label("output id:");
            ui.text_edit_singleline(&mut self.canvas.output_id);
        });

        let can_save = self.loaded.is_some()
            && !self.canvas.order.is_empty()
            && !self.canvas.output_id.trim().is_empty();
        let mut save_clicked = false;
        if ui
            .add_enabled(can_save, egui::Button::new("Save into active olib"))
            .clicked()
        {
            save_clicked = true;
        }
        if !self.canvas.status.is_empty() {
            ui.weak(&self.canvas.status);
        }

        // Handle the save outside the borrow-heavy reads above so the
        // post-save library refresh has clean mutable access to self.
        if save_clicked {
            self.handle_canvas_save();
        }
    }

    /// Fork the editor's current state into a sibling file at
    /// `<olib>/<canonical>-v<dashed-new-version>.md` per the s5
    /// versioning convention. Refresh the loaded library on success
    /// so the new sibling appears in the Library Components tree.
    fn save_as_new_version_now(&mut self) {
        let Some(ed) = self.editor.as_ref() else {
            return;
        };
        let Some(idx) = self.selected_olib else {
            return;
        };
        let Some(olib_dir) = self.discovered.get(idx).map(|d| d.path.clone()) else {
            return;
        };

        let new_ver = match oovra::header::bump_version(&ed.version, self.new_version_bump) {
            Ok(v) => v,
            Err(e) => {
                self.status = format!("Save As New Version failed: bump: {e}");
                return;
            }
        };

        let (canonical, _) = oovra::header::parse_filename_version(&ed.id);
        let new_stem = match oovra::header::compose_versioned_filename(&canonical, &new_ver) {
            Ok(s) => s,
            Err(e) => {
                self.status = format!("Save As New Version failed: filename: {e}");
                return;
            }
        };
        let new_path = olib_dir.join(format!("{new_stem}.md"));

        let header = oovra::PromptElementHeader {
            name: canonical.clone(),
            kind: oovra::PromptElementKind::Atom,
            id: new_stem.clone(),
            version: new_ver.clone(),
            meta: ed.meta.clone(),
            generated_at: None,
            render_mode: None,
            body_level: None,
            depth: None,
            composed_of: None,
        };
        let element = oovra::PromptElement::new(header, ed.body.clone());

        if let Err(e) = oovra::write(&element, &new_path) {
            self.status = format!("Save As New Version write failed: {e}");
            return;
        }
        self.status = format!("Saved new version v{new_ver} at {}", new_path.display());
        // Refresh the active library so the new sibling appears.
        self.load_selected_olib(idx, &olib_dir);
    }

    /// Render the modal-ish "Are you sure?" save confirm window.
    /// Called from `ui()` once per frame; only draws when
    /// `save_confirm_pending` is true.
    fn render_save_confirm(&mut self, ctx: &egui::Context) {
        if !self.save_confirm_pending {
            return;
        }
        let mut decide: Option<bool> = None;
        let editor_label = self
            .editor
            .as_ref()
            .map(|ed| format!("'{}' v{}", ed.id, ed.version))
            .unwrap_or_else(|| "this component".to_owned());
        egui::Window::new("Confirm save in place")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Save changes to {editor_label} in place?"));
                ui.add_space(4.0);
                ui.weak(
                    "Anyone composing with this id @ version will see the new \
                     content under an existing pin — this is a release-after-\
                     publish edit. If you want to preserve the existing \
                     version, cancel and use Save As New Version instead.",
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Yes, save in place").clicked() {
                        decide = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        decide = Some(false);
                    }
                });
            });
        match decide {
            Some(true) => {
                if let Some(ed) = self.editor.as_mut() {
                    let _ = ed.save();
                }
                self.save_confirm_pending = false;
            }
            Some(false) => {
                self.save_confirm_pending = false;
            }
            None => {}
        }
    }

    /// Compose the current canvas into a compound and write it into
    /// the active olib. Refresh the loaded library on success so the
    /// new compound appears in the Library Components column.
    fn handle_canvas_save(&mut self) {
        let Some(idx) = self.selected_olib else {
            return;
        };
        let Some(olib_dir) = self.discovered.get(idx).map(|d| d.path.clone()) else {
            return;
        };
        let save_result = match &self.loaded {
            Some(lib) => self.canvas.save_as_compound(lib, &olib_dir),
            None => return,
        };
        match save_result {
            Ok(path) => {
                self.canvas.status = format!("Saved {}", path.display());
                self.status = self.canvas.status.clone();
                self.load_selected_olib(idx, &olib_dir);
            }
            Err(e) => {
                self.canvas.status = format!("Save failed: {e}");
            }
        }
    }

    /// Render the Compare view: two ComboBox pickers driving a
    /// live-recomputed DiffReport.
    fn render_compare(&mut self, ui: &mut egui::Ui) {
        if self.atom_index.is_empty() {
            ui.weak("(no olib loaded — pick one on the left to compare its components)");
            return;
        }

        // Owned snapshot for the closure captures.
        let ids: Vec<String> = self.atom_index.clone();

        // A picker
        let mut a_changed: Option<Option<String>> = None;
        let a_text = self
            .compare
            .a
            .as_deref()
            .unwrap_or("(pick a component)")
            .to_owned();
        ui.horizontal(|ui| {
            ui.label("A:");
            egui::ComboBox::from_id_salt("compare_a")
                .selected_text(a_text)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(self.compare.a.is_none(), "(none)")
                        .clicked()
                    {
                        a_changed = Some(None);
                    }
                    for id in &ids {
                        let is_sel = self.compare.a.as_deref() == Some(id);
                        if ui.selectable_label(is_sel, id).clicked() {
                            a_changed = Some(Some(id.clone()));
                        }
                    }
                });
        });
        if let Some(new) = a_changed {
            self.compare.set_a(new, self.loaded.as_ref());
        }

        // B picker
        let mut b_changed: Option<Option<String>> = None;
        let b_text = self
            .compare
            .b
            .as_deref()
            .unwrap_or("(pick a component)")
            .to_owned();
        ui.horizontal(|ui| {
            ui.label("B:");
            egui::ComboBox::from_id_salt("compare_b")
                .selected_text(b_text)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(self.compare.b.is_none(), "(none)")
                        .clicked()
                    {
                        b_changed = Some(None);
                    }
                    for id in &ids {
                        let is_sel = self.compare.b.as_deref() == Some(id);
                        if ui.selectable_label(is_sel, id).clicked() {
                            b_changed = Some(Some(id.clone()));
                        }
                    }
                });
        });
        if let Some(new) = b_changed {
            self.compare.set_b(new, self.loaded.as_ref());
        }

        ui.separator();

        match &self.compare.report {
            None => {
                let hint = match (&self.compare.a, &self.compare.b) {
                    (Some(a), Some(b)) if a == b => "Pick a different second component.",
                    _ => "Pick two components above to compare.",
                };
                ui.weak(hint);
            }
            Some(Err(msg)) => {
                ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(220, 80, 80)));
            }
            Some(Ok(report)) => {
                render_diff_report(ui, report);
            }
        }
    }

    fn render_editor(&mut self, ui: &mut egui::Ui) {
        // Local action flags collected during the editor render and
        // applied after the ed borrow releases.
        let mut request_save_confirm = false;
        let mut request_save_as_new_version = false;

        // Three states: editor loaded; compound (read-only msg); nothing.
        if let Some(ed) = &mut self.editor {
            ui.add_space(2.0);
            egui::Grid::new("editor_fields")
                .num_columns(2)
                .min_col_width(110.0)
                .show(ui, |ui| {
                    ui.weak("Filesystem Name");
                    // Read-only: tied to the on-disk file. Use Save As
                    // New Version (or a future explicit rename) to
                    // change it.
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut ed.id.clone()));
                    ui.end_row();

                    ui.weak("Component-ID");
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
                    request_save_confirm = true;
                }
                if ui.button("Save As New Version").clicked() {
                    request_save_as_new_version = true;
                }
                egui::ComboBox::from_id_salt("new_version_bump")
                    .selected_text(format!("{:?}", self.new_version_bump))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.new_version_bump,
                            oovra::header::BumpKind::Patch,
                            "Patch",
                        );
                        ui.selectable_value(
                            &mut self.new_version_bump,
                            oovra::header::BumpKind::Minor,
                            "Minor",
                        );
                        ui.selectable_value(
                            &mut self.new_version_bump,
                            oovra::header::BumpKind::Major,
                            "Major",
                        );
                    });
                if ui.button("Reload").clicked() {
                    let _ = ed.reload();
                }
                if ui.button("Bump (no-fork)").clicked() {
                    match oovra::header::bump_version(&ed.version, self.new_version_bump) {
                        Ok(new) => {
                            ed.version = new;
                            ed.dirty = true;
                            ed.status = "version bumped in-memory — Save to persist".to_owned();
                        }
                        Err(e) => {
                            ed.status = format!("Bump failed: {e}");
                        }
                    }
                }
                ui.separator();
                ui.weak(&ed.status);
            });
            // Fall through past the ed borrow so we can apply the
            // button-collected requests with full mutable access to
            // self.
        }
        if request_save_confirm {
            self.save_confirm_pending = true;
        }
        if request_save_as_new_version {
            self.save_as_new_version_now();
        }
        if self.editor.is_some() {
            return;
        }
        if let Some(msg) = &self.compound_msg {
            ui.label(egui::RichText::new("compound — read-only").strong());
            ui.add_space(4.0);
            ui.weak(msg);
            ui.add_space(8.0);
            ui.weak(
                "Editing compounds is a future sprint; for now, edit \
                 the recipe by re-composing from the CLI.",
            );
            return;
        }
        ui.weak(
            "Select a library component in the middle column to open it in the Component Editor.",
        );
    }
}

impl eframe::App for OovraApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Top toolbar
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
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
        egui::Panel::bottom("footer").show_inside(ui, |ui| {
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

        // Left sidebar: discovered olibs under the opened folder.
        // Label is the user-facing canonical: an Olib is a folder
        // literally named `olib`, and discovery recurses into the
        // opened folder.
        egui::Panel::left("olibs")
            .resizable(true)
            .default_size(280.0)
            .show_inside(ui, |ui| {
                ui.heading("Oovra Library Directories (Olibs)");
                ui.separator();
                if self.discovered.is_empty() {
                    ui.weak("(open a folder to discover olibs)");
                } else {
                    self.render_olib_list(ui);
                }
            });

        // Second left sidebar: the contents of the selected olib —
        // both atoms and compounds are "library components" (anything
        // in the oovra format that `compose` operates on).
        egui::Panel::left("components")
            .resizable(true)
            .default_size(260.0)
            .show_inside(ui, |ui| {
                ui.heading("Library Components");
                ui.horizontal(|ui| {
                    if ui.small_button("Expand all").clicked() {
                        self.pending_open = Some(true);
                    }
                    if ui.small_button("Collapse all").clicked() {
                        self.pending_open = Some(false);
                    }
                });
                ui.separator();
                self.render_component_tree(ui);
            });

        // Central: the Component Editor (with Editor / Canvas tabs).
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Component Editor");
            ui.weak(format!("oovra-gui  ·  linked to oovra v{}", oovra::VERSION));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.view, CentralView::Editor, "Editor");
                ui.selectable_value(&mut self.view, CentralView::Canvas, "Canvas");
                ui.selectable_value(&mut self.view, CentralView::Compare, "Compare");
            });
            ui.separator();
            match self.view {
                CentralView::Editor => self.render_editor(ui),
                CentralView::Canvas => self.render_canvas(ui),
                CentralView::Compare => self.render_compare(ui),
            }
        });

        // The save-confirm modal renders on top of the panels, last,
        // so it's always visible when pending.
        self.render_save_confirm(ui.ctx());
    }
}

/// Render a `DiffReport` into the given Ui. Atom vs atom: field
/// changes table + colored body unified diff. Compound vs compound:
/// added / removed / version_changed / moved lists.
fn render_diff_report(ui: &mut egui::Ui, report: &DiffReport) {
    const GREEN: egui::Color32 = egui::Color32::from_rgb(80, 180, 80);
    const RED: egui::Color32 = egui::Color32::from_rgb(220, 80, 80);
    const HUNK: egui::Color32 = egui::Color32::from_rgb(120, 160, 220);
    const DIM: egui::Color32 = egui::Color32::from_rgb(160, 160, 160);

    match report {
        DiffReport::Content(c) => {
            ui.label(egui::RichText::new(format!("{} → {}  (atoms)", c.a_id, c.b_id)).strong());
            ui.add_space(2.0);

            if c.field_changes.is_empty() {
                ui.label(egui::RichText::new("metadata: unchanged").color(DIM));
            } else {
                ui.label("metadata changes:");
                egui::Grid::new("compare_field_changes")
                    .num_columns(3)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("field").color(DIM));
                        ui.label(egui::RichText::new("before").color(DIM));
                        ui.label(egui::RichText::new("after").color(DIM));
                        ui.end_row();
                        for fc in &c.field_changes {
                            ui.label(&fc.field);
                            ui.label(egui::RichText::new(&fc.before).color(RED));
                            ui.label(egui::RichText::new(&fc.after).color(GREEN));
                            ui.end_row();
                        }
                    });
            }

            ui.add_space(6.0);
            if c.bodies_equal {
                ui.label(egui::RichText::new("body: unchanged").color(DIM));
            } else {
                ui.label("body diff:");
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for line in c.body_unified_diff.lines() {
                            let color = if line.starts_with("+++") || line.starts_with("---") {
                                Some(DIM)
                            } else if line.starts_with('+') {
                                Some(GREEN)
                            } else if line.starts_with('-') {
                                Some(RED)
                            } else if line.starts_with("@@") {
                                Some(HUNK)
                            } else {
                                None
                            };
                            let mut rt = egui::RichText::new(line).monospace();
                            if let Some(c) = color {
                                rt = rt.color(c);
                            }
                            ui.add(egui::Label::new(rt).selectable(true));
                        }
                    });
            }
        }
        DiffReport::Structural(s) => {
            ui.label(egui::RichText::new(format!("{} → {}  (compounds)", s.a_id, s.b_id)).strong());
            ui.add_space(2.0);
            if s.recipes_equal {
                ui.label(egui::RichText::new("recipes: identical").color(DIM));
                return;
            }
            if !s.added.is_empty() {
                ui.label("added inputs:");
                for pi in &s.added {
                    ui.label(
                        egui::RichText::new(format!(
                            "  + [{}] {} @ {}",
                            pi.position, pi.input.id, pi.input.version
                        ))
                        .color(GREEN),
                    );
                }
            }
            if !s.removed.is_empty() {
                ui.label("removed inputs:");
                for pi in &s.removed {
                    ui.label(
                        egui::RichText::new(format!(
                            "  - [{}] {} @ {}",
                            pi.position, pi.input.id, pi.input.version
                        ))
                        .color(RED),
                    );
                }
            }
            if !s.version_changed.is_empty() {
                ui.label("version-changed inputs:");
                for v in &s.version_changed {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("  ~").strong());
                        ui.label(&v.id);
                        ui.label(":");
                        ui.label(egui::RichText::new(&v.before_version).color(RED));
                        ui.label("→");
                        ui.label(egui::RichText::new(&v.after_version).color(GREEN));
                    });
                }
            }
            if !s.moved.is_empty() {
                ui.label("moved inputs:");
                for m in &s.moved {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("  ↔").color(HUNK));
                        ui.label(format!("{} @ {}", m.id, m.version));
                        ui.label(":");
                        ui.label(egui::RichText::new(format!("pos {}", m.before_pos)).color(RED));
                        ui.label("→");
                        ui.label(egui::RichText::new(format!("pos {}", m.after_pos)).color(GREEN));
                    });
                }
            }
        }
    }
}

/// Render a component's header tags inline after its tree-row label
/// — the version, and a truncated meta when present — both dimmed so
/// they read as metadata rather than primary content. This is the
/// "see the tags without opening" affordance.
fn tag_labels(ui: &mut egui::Ui, version: &str, meta: &str) {
    if !version.is_empty() {
        ui.weak(format!("v{version}"));
    }
    if !meta.is_empty() {
        let short = if meta.chars().count() > 40 {
            let head: String = meta.chars().take(40).collect();
            format!("— {head}…")
        } else {
            format!("— {meta}")
        };
        ui.weak(short);
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
