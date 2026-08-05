//! Scale, rotate and blur a child ui, and check that it stays usable.
//!
//! Drag the slider inside the child while the child is rotated: the pointer is mapped
//! through the inverse transform, so the slider follows the pointer along its own axis
//! rather than the screen's.
//!
//! Run with `--features wgpu` for the off-screen renderer, which clips a rotated child
//! exactly, keeps text crisp at any scale, and can blur the child's own content.

use egui::{ScrollArea, Slider, Ui, vec2};
use regui::Regui;

fn main() {
    let mut scale = 1.0_f32;
    let mut rotation = 0.0_f32;
    let mut crisp = false;
    let mut offscreen = false;
    let mut blur = 0.0_f32;
    let mut child_value = 0.5_f32;
    let mut child_checked = false;
    let mut chosen = "nothing".to_owned();

    hello_egui_utils_dev::run!(move |ui: &mut Ui, frame: &mut eframe::Frame| {
        #[cfg(feature = "wgpu")]
        if let Some(render_state) = frame.wgpu_render_state() {
            regui::install_wgpu(ui.ctx(), render_state.clone());
        }
        #[cfg(not(feature = "wgpu"))]
        let _ = frame;

        ui.add(Slider::new(&mut scale, 0.25..=3.0).text("scale"));
        ui.add(
            Slider::new(&mut rotation, -std::f32::consts::PI..=std::f32::consts::PI)
                .text("rotation"),
        );
        ui.checkbox(&mut crisp, "crisp text (re-rasterize glyphs at this scale)");

        if cfg!(feature = "wgpu") {
            ui.checkbox(
                &mut offscreen,
                "render through a texture (exact clipping when rotated)",
            );
            ui.add(Slider::new(&mut blur, 0.0..=30.0).text("blur the child itself"));
        } else {
            ui.label("Build with --features wgpu for the off-screen renderer and blur.");
        }

        ui.separator();

        let child = Regui::new("child")
            // Tall enough that the menu popup has somewhere to open.
            .size(vec2(240.0, 220.0))
            .scale(scale)
            .rotation(rotation)
            .crisp(crisp);

        #[cfg(feature = "wgpu")]
        let child = child.offscreen(offscreen).blur(blur);

        child.show(ui, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                // A menu is the hardest thing to put in a child: it opens a popup, and all
                // of the child's shapes go into one layer of the parent, so the popup cannot
                // escape the child's rect. Give the child room below the button, or the menu
                // has nowhere to go.
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Menu", |ui| {
                        for option in ["First", "Second", "Third"] {
                            if ui.button(option).clicked() {
                                option.clone_into(&mut chosen);
                                ui.close();
                            }
                        }
                    });
                });
                ui.label(format!("Picked: {chosen}"));

                ui.add(Slider::new(&mut child_value, 0.0..=1.0).text("value"));
                ui.checkbox(&mut child_checked, "and check a box");

                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for i in 0..100 {
                        ui.group(|ui| {
                            for i in 0..5 {
                                ui.label("Hello");
                            }
                        });
                    }
                });
            });
        });
    });
}
