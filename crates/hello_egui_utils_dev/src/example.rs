use eframe::NativeOptions;
use egui::{CentralPanel, Ui};

/// Run an example with the given name and content.
pub fn run(name: &str, mut f: impl FnMut(&mut Ui) + 'static) {
    let mut initialized = false;
    eframe::run_ui_native(name, NativeOptions::default(), move |ui, _frame| {
        if !initialized {
            initialized = true;
            return;
        }
        CentralPanel::default().show_inside(ui, |ui| {
            let mut style = (*ui.ctx().global_style()).clone();
            ui.checkbox(&mut style.debug.debug_on_hover, "Debug on hover");
            ui.checkbox(&mut style.visuals.dark_mode, "Dark mode");
            ui.ctx().set_global_style(style);

            f(ui);
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
