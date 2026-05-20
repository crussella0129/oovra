//! oovra-gui — entry points for native and WASM, gated by cfg.
//!
//! The native path opens an eframe window. The WASM path is driven
//! by Trunk: it compiles this bin to `wasm32-unknown-unknown`, wraps
//! it with `wasm-bindgen`, and serves it from `index.html`.

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// ----- Native --------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // RUST_LOG=info to see eframe/winit/wgpu logs.

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 480.0])
            .with_min_inner_size([480.0, 320.0])
            .with_title("oovra-gui"),
        ..Default::default()
    };

    eframe::run_native(
        "oovra-gui",
        native_options,
        Box::new(|cc| Ok(Box::new(oovra_gui::OovraApp::new(cc)))),
    )
}

// ----- WASM ----------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Pipe `log` to the browser console.
    eframe::WebLogger::init(log::LevelFilter::Info).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("missing #the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#the_canvas_id is not a <canvas>");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(oovra_gui::OovraApp::new(cc)))),
            )
            .await;

        // Replace / remove the loading text the page shows before WASM starts.
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => loading_text.remove(),
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p>oovra-gui failed to start. See the dev console for details.</p>",
                    );
                    panic!("eframe WebRunner failed: {e:?}");
                }
            }
        }
    });
}
