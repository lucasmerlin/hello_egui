use egui::Button;
use egui_styled_widgets::PrimaryStyleExt;

fn main() {
    
    eframe::run_simple_native(
        "Styled Widgets Example",
        eframe::NativeOptions::default(),
        |ctx, _| {
            egui::CentralPanel::default().show(ctx, |ui| {
                
                ui.add(Button::new("Primary Button").primary());
                
            });
        },
    );
    
}