use eframe::{egui, NativeOptions};

use egui::{CentralPanel, Frame, Label, ScrollArea, TopBottomPanel, Ui, Vec2, Widget};
use egui_dnd::dnd;

pub fn main() -> eframe::Result<()> {
    let mut items: Vec<_> = (1..1000).collect();

    let mut example = "wrapping";

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Sorted: {:?}",
                        items
                            .iter()
                            .enumerate()
                            .all(|(i, item)| i == *item as usize - 1)
                    ));
                });
            });

            CentralPanel::default().show(ctx, |ui| {
                ui.style_mut().animation_time = 0.15;

                ui.spacing_mut().item_spacing.y = ui.spacing().item_spacing.x;
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut example, "wrapping", "wrapping");
                    ui.selectable_value(&mut example, "scrolling", "scrolling");
                });

                let av_width = ui.available_width() - ui.spacing().item_spacing.x;
                let columns = (av_width / 100.0).ceil() as usize;
                let width = av_width / columns as f32;
                let size = Vec2::new(width, width) - ui.spacing().item_spacing;

                let content = |ui: &mut Ui, items: &mut [i32]| {
                    dnd(ui, "dnd_example").show_vec_sized(
                        items,
                        size,
                        |ui, item, handle, _state| {
                            Frame::NONE
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
            });
        },
    )
}
