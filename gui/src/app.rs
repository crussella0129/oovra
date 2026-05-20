//! Sprint s0 bootstrap App: a single panel that proves the gui crate
//! compiles, links the `oovra` library crate, and uses its real API
//! at runtime. Subsequent sprints add the file explorer, the
//! syntax-highlighted editor, and the live autocompose canvas.

use oovra::header::{is_kebab_case, slugify};

/// Application state. Persisted across runs via eframe's persistence
/// feature, keyed by [`eframe::APP_KEY`].
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OovraApp {
    /// Live input for the `is_kebab_case` / `slugify` probe. Seeded
    /// with a deliberately non-kebab value so the bootstrap demo
    /// shows the slug path immediately.
    kebab_probe: String,
}

impl Default for OovraApp {
    fn default() -> Self {
        Self {
            kebab_probe: "My Draft".to_owned(),
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
}

impl eframe::App for OovraApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    // eframe 0.34.2 replaced the per-frame `update(ctx, frame)` entry
    // point with `ui(ui, frame)`: the runtime now hands you a `Ui`
    // already attached to the central area, so we no longer set up a
    // CentralPanel ourselves.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("oovra-gui — sprint s0 bootstrap");
        ui.label(format!(
            "Linked to oovra v{} via the workspace path-dep.",
            oovra::VERSION
        ));
        ui.separator();

        ui.label("Live probe — exercising oovra::header at runtime:");
        ui.horizontal(|ui| {
            ui.label("filename stem:");
            ui.text_edit_singleline(&mut self.kebab_probe);
        });

        let kebab = is_kebab_case(&self.kebab_probe);
        let slug = slugify(&self.kebab_probe);
        ui.label(format!("  is_kebab_case = {kebab}"));
        ui.label(format!("  slugify       = {slug:?}"));

        ui.separator();
        ui.weak(
            "This shell is the s0 deliverable. File explorer, syntax-highlighted \
             editor, and the autocompose canvas come in later sprints.",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_app_is_constructible_and_oovra_lib_is_reachable() {
        // U-1: OovraApp::default() builds.
        // U-2: a real call into the oovra library succeeds from the gui crate
        //      — proving the path-dep + lib surface are wired together.
        let app = OovraApp::default();
        assert!(!app.kebab_probe.is_empty());
        assert!(oovra::header::is_kebab_case("my-id"));
        assert_eq!(
            oovra::header::slugify("My Draft").as_deref(),
            Some("my-draft")
        );
    }
}
