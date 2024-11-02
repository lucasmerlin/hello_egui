use eframe::NativeOptions;
use egui::{CentralPanel, Style, Ui};

pub fn run(name: &str, mut f: impl FnMut(&mut Ui) + 'static) {
    let mut initalized = false;
    eframe::run_simple_native("helloui", NativeOptions::default(), move |ctx, _frame| {
        if !initalized {
            initalized = true;
            return;
        }
        CentralPanel::default().show(ctx, |ui| {
            //ctx.inspection_ui(ui);
            let mut style = (*ctx.style()).clone();
            ui.checkbox(&mut style.debug.debug_on_hover, "Debug on hover");
            ui.checkbox(&mut style.visuals.dark_mode, "Dark mode");
            ctx.set_style(style);

            f(ui);
        });
    })
    .unwrap();
}

#[macro_export]
macro_rules! run {
    ($content:expr) => {
        $crate::example::run(file!(), $content);
    };
}
