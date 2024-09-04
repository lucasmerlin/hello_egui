use eframe::emath::Align2;
use eframe::NativeOptions;
use egui::{Area, Frame, Id, Vec2};
use egui_flex::{Flex, FlexItem};

fn main() -> eframe::Result {
    eframe::run_simple_native(
        "flex debug example",
        NativeOptions::default(),
        |ctx, _frame| {
            Area::new(Id::new("area"))
                .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.set_width(500.0);
                        ui.set_height(200.0);
                        ui.spacing_mut().item_spacing = Vec2::splat(10.0);

                        let frame = Frame::group(ui.style()).inner_margin(5.0).outer_margin(5.0);
                        Flex::new()
                            .align_items(egui_flex::FlexAlign::Stretch)
                            .align_item_content(Align2::RIGHT_CENTER)
                            .show(ui, |flex| {
                                [100.0, 200.0, 100.0].iter().for_each(|&width| {
                                    flex.add_container(
                                        FlexItem::default().grow(1.0),
                                        |ui, content| {
                                            frame
                                                .show(ui, |ui| {
                                                    content.content(ui, |ui| {
                                                        Frame::none()
                                                            .stroke(egui::Stroke::new(
                                                                1.0,
                                                                egui::Color32::RED,
                                                            ))
                                                            .show(ui, |ui| {
                                                                ui.label(format!("{}", width));
                                                                ui.set_width(width);
                                                                ui.set_height(width);
                                                            });
                                                    })
                                                })
                                                .inner
                                        },
                                    );
                                });
                            });
                    });
                });
        },
    )
}
