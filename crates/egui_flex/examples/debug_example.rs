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
                    ui.set_width(500.0);
                    ui.set_height(200.0);
                    ui.spacing_mut().item_spacing = Vec2::splat(10.0);

                    let frame = Frame::group(ui.style()).inner_margin(5.0).outer_margin(5.0);
                    Flex::horizontal()
                        .align_items(egui_flex::FlexAlign::Stretch)
                        .align_items_content(Align2::RIGHT_CENTER)
                        .show(ui, |flex| {
                            flex.add_flex(
                                FlexItem::default().grow(1.0),
                                Flex::vertical()
                                    .align_content(egui_flex::FlexAlignContent::Stretch)
                                    .grow_items(1.0),
                                |flex| {
                                    flex.add(
                                        FlexItem::default().grow(1.0),
                                        egui::Label::new("Hello"),
                                    );
                                    flex.add(
                                        FlexItem::default().grow(1.0),
                                        egui::Label::new("World"),
                                    );
                                },
                            );

                            flex.add_flex(
                                FlexItem::default().grow(1.0).frame(frame),
                                Flex::vertical()
                                    .align_content(egui_flex::FlexAlignContent::Stretch)
                                    .grow_items(1.0),
                                |flex| {
                                    flex.add(
                                        FlexItem::default().grow(1.0),
                                        egui::Label::new("Hello"),
                                    );
                                    flex.add(
                                        FlexItem::default().grow(1.0),
                                        egui::Label::new("World"),
                                    );

                                    flex.add_flex(
                                        FlexItem::default().grow(1.0),
                                        Flex::horizontal()
                                            .align_content(egui_flex::FlexAlignContent::Stretch)
                                            .grow_items(1.0),
                                        |flex| {
                                            flex.add(
                                                FlexItem::default().grow(1.0),
                                                egui::Label::new("Hello"),
                                            );
                                            flex.add(
                                                FlexItem::default().grow(1.0),
                                                egui::Label::new("World"),
                                            );
                                        },
                                    );
                                },
                            );

                            flex.add_flex(
                                FlexItem::default().grow(1.0),
                                Flex::vertical()
                                    .align_content(egui_flex::FlexAlignContent::Stretch)
                                    .grow_items(1.0),
                                |flex| {
                                    flex.add(
                                        FlexItem::default().grow(1.0),
                                        egui::Label::new("Hello"),
                                    );
                                    flex.add(
                                        FlexItem::default().grow(1.0),
                                        egui::Label::new("World"),
                                    );
                                },
                            );
                        });
                });
        },
    )
}
