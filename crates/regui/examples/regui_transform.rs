//! Scale and rotate a child ui, and check that it stays usable.
//!
//! Drag the slider inside the child while the child is rotated: the pointer is mapped
//! through the inverse transform, so the slider follows the pointer along its own axis
//! rather than the screen's.

use egui::{Slider, Ui, vec2};
use regui::Regui;

fn main() {
    let mut scale = 1.0_f32;
    let mut rotation = 0.0_f32;
    let mut crisp = false;
    let mut child_value = 0.5_f32;
    let mut child_checked = false;

    hello_egui_utils_dev::run!(move |ui: &mut Ui, _frame: &mut eframe::Frame| {
        ui.add(Slider::new(&mut scale, 0.25..=3.0).text("scale"));
        ui.add(
            Slider::new(&mut rotation, -std::f32::consts::PI..=std::f32::consts::PI)
                .text("rotation"),
        );
        ui.checkbox(&mut crisp, "crisp text (re-rasterize glyphs at this scale)");

        ui.separator();

        Regui::new("child")
            .size(vec2(220.0, 120.0))
            .scale(scale)
            .rotation(rotation)
            .crisp(crisp)
            .show(ui, |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label("Rotate me, then drag the slider.");
                    ui.add(Slider::new(&mut child_value, 0.0..=1.0).text("value"));
                    ui.checkbox(&mut child_checked, "and check a box");
                });
            });
    });
}
