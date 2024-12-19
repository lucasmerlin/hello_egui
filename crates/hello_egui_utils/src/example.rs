use eframe::NativeOptions;
use egui::{CentralPanel, Ui};

/// Run an example with the given name and content.
pub fn run(name: &str, mut f: impl FnMut(&mut Ui) + 'static) {
    let mut initialized = false;
    eframe::run_simple_native(name, NativeOptions::default(), move |ctx, _frame| {
        if !initialized {
            initialized = true;
            return;
        }
        CentralPanel::default().show(ctx, |ui| {
            let mut style = (*ctx.style()).clone();
            ui.checkbox(&mut style.debug.debug_on_hover, "Debug on hover");
            ui.checkbox(&mut style.visuals.dark_mode, "Dark mode");
            ctx.set_style(style);

            f(ui);
        });
    })
    .unwrap();
}

/// Run an example with the given content.
#[macro_export]
macro_rules! run {
    ($content:expr) => {
        $crate::example::run(file!(), $content);
    };
}
