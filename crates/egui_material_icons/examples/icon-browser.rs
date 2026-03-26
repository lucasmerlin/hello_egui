use eframe::egui;
use egui::{Label, Widget};
use egui_material_icons::icons::ICON_RECTANGLE;
use egui_material_icons::{
    icon_button,
    icons::{ICON_FAVORITE, ICON_IMAGE},
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
                ui.label("Filled:");
                icon_button(ui, ICON_IMAGE);
                icon_button(ui, ICON_RECTANGLE);
                #[cfg(feature = "outline")]
                {
                    ui.label("Outlined:");
                    icon_button(ui, ICON_IMAGE.outlined());
                    icon_button(ui, ICON_RECTANGLE.outlined());
                }
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
