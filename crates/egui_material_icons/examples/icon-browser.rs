use eframe::egui;
use egui::{Label, Widget};
use egui_material_icons::{
    icon_button,
    icons::{ICON_ADD, ICON_FAVORITE, ICON_IMAGE, ICON_REMOVE},
};

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
}

#[derive(Default)]
struct MyEguiApp {}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_material_icons::initialize(&cc.egui_ctx);

        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                icon_button(ui, ICON_ADD);
                icon_button(ui, ICON_REMOVE);
                icon_button(ui, ICON_IMAGE);
                ui.label("Ayyy")
            });

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    Label::new(ICON_FAVORITE.rich_text().size(16.0)).ui(ui);
                    Label::new("2").ui(ui);
                });
            });
        });
    }
}
