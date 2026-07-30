use eframe::{Frame, NativeOptions};
use egui::{CentralPanel, Panel, Theme, Ui};

/// Run an example with the given name and content.
///
/// The content gets the [`Frame`] as well as the [`Ui`], since some examples need what is
/// on it, such as the wgpu render state.
pub fn run(name: &str, mut f: impl FnMut(&mut Ui, &mut Frame) + 'static) {
    let mut initialized = false;
    eframe::run_ui_native(name, NativeOptions::default(), move |ui, frame| {
        if !initialized {
            initialized = true;
            return;
        }
        // These live in their own panel so that an example is free to fill its whole
        // `max_rect` without painting over them.
        Panel::top("hello_egui_utils_dev_controls").show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut style = (*ui.ctx().global_style()).clone();
                if ui
                    .checkbox(&mut style.debug.debug_on_hover, "Debug on hover")
                    .changed()
                {
                    ui.ctx().set_global_style(style);
                }

                // `Visuals::dark_mode` only reports which theme is in use; setting it does
                // not switch themes, since the style gets rebuilt from the theme.
                let mut dark_mode = ui.ctx().theme() == Theme::Dark;
                if ui.checkbox(&mut dark_mode, "Dark mode").changed() {
                    ui.ctx()
                        .set_theme(if dark_mode { Theme::Dark } else { Theme::Light });
                }
            });
        });

        CentralPanel::default().show(ui, |ui| {
            f(ui, frame);
        });
    })
    .unwrap();
}

/// Run an example with the given content.
#[macro_export]
macro_rules! run {
    ($content:expr) => {
        $crate::run(file!(), $content);
    };
}
