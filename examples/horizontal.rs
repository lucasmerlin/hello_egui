use eframe::egui;
use eframe::emath::Align;
use egui::{CentralPanel, Frame, Label, Layout, ScrollArea, Sense, Ui, Vec2, Widget};
use egui_dnd::dnd;

pub fn main() -> eframe::Result<()> {
    let mut items: Vec<_> = (1..1000).collect();

    let mut example = "wrapping";

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = ui.spacing().item_spacing.x;
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut example, "wrapping", "wrapping");
                    ui.selectable_value(&mut example, "scrolling", "scrolling");
                });

                let av_width = ui.available_width() - ui.spacing().item_spacing.x;
                let columns = (av_width / 100.0).ceil() as usize;
                let width = av_width / columns as f32;
                let size = Vec2::new(width, width) - ui.spacing().item_spacing;

                let mut content = |ui: &mut Ui, items: &mut [i32]| {
                    dnd(ui, "dnd_example").show_vec_sized(
                        items,
                        size,
                        |ui, item, handle, state| {
                            Frame::none()
                                .fill(ui.visuals().faint_bg_color)
                                .show(ui, |ui| {
                                    handle.ui_sized(ui, size, |ui| {
                                        // ui.label("drag");
                                        ui.set_width(ui.available_width());
                                        ui.set_height(ui.available_height());
                                        ui.centered_and_justified(|ui| {
                                            Label::new(item.to_string()).ui(ui);
                                        });
                                    });
                                });
                        },
                    );
                };

                if example == "wrapping" {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            content(ui, &mut items);
                        });
                    });
                } else {
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.set_height(ui.available_height());
                        ui.horizontal(|ui| {
                            content(ui, &mut items);
                        });
                    });
                }

                ctx.style_ui(ui);
            });
        },
    )
}
