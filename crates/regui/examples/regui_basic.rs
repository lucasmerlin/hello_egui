//! A child ui that is a real, interactive egui viewport.
//!
//! Everything in the box below runs in its own viewport: it has its own input, its own
//! hit-testing and its own focus, but it shares the app's memory, style and fonts.

use egui::{Slider, TextEdit, Ui, vec2};
use regui::Regui;

fn main() {
    let mut name = "World".to_owned();
    let mut volume = 50.0_f32;
    let mut clicks = 0_u32;
    let mut parent_text = "type here too".to_owned();

    hello_egui_utils_dev::run!(move |ui: &mut Ui, _frame: &mut eframe::Frame| {
        ui.horizontal(|ui| {
            ui.label("Parent text field:");
            ui.add(TextEdit::singleline(&mut parent_text).desired_width(150.0));
        });
        ui.label("Focus should move between the two text fields by clicking.");

        ui.add_space(8.0);

        let frame = egui::Frame::group(ui.style());
        frame.show(ui, |ui| {
            Regui::new("child").size(vec2(300.0, 160.0)).show(ui, |ui| {
                ui.heading("I am a child viewport");

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.add(TextEdit::singleline(&mut name).desired_width(120.0));
                });
                ui.label(format!("Hello {name}!"));

                ui.add(Slider::new(&mut volume, 0.0..=100.0).text("volume"));

                if ui.button("Click me").clicked() {
                    clicks += 1;
                }
                ui.label(format!("Clicked {clicks} times"));

                // Proves that repaint requests get bridged to the parent: without
                // that, this spinner would freeze.
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("still animating");
                });
            });
        });

        ui.add_space(8.0);
        ui.label(format!("The parent can see the state too: {clicks} clicks"));
    });
}
