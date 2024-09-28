use eframe::NativeOptions;
use egui::{Button, CentralPanel, Checkbox, DragValue, Frame, TextEdit, Vec2};
use egui_flex::{Flex, FlexAlignContent, FlexItem};

fn main() -> eframe::Result {
    let mut flt = 0.0;

    let mut txt = String::new();

    eframe::run_simple_native(
        "flex nested",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(10.0);
                let frame = Frame::group(ui.style());
                Flex::horizontal()
                    .align_content(FlexAlignContent::Normal)
                    .grow_items(1.0)
                    .show(ui, |flex| {
                        flex.add_flex_frame(
                            FlexItem::new(),
                            Flex::vertical()
                                .align_content(FlexAlignContent::Stretch)
                                .grow_items(1.0),
                            Frame::group(flex.ui().style()),
                            |flex| {
                                flex.add(FlexItem::new(), Button::new("btn"));
                                // flex.add(
                                //     FlexItem::new(),
                                //     Slider::new(&mut flt, 0.0..=1000.0).show_value(false),
                                // );
                                flex.add(
                                    FlexItem::new().grow(0.0),
                                    TextEdit::singleline(&mut txt).desired_width(100.0),
                                );
                                flex.add(FlexItem::new(), DragValue::new(&mut flt));
                                flex.add(FlexItem::new(), Checkbox::new(&mut false, "Checkbox"));
                            },
                        );

                        flex.add(FlexItem::new().grow(1.0), Button::new("Single Button"));

                        flex.add_flex_frame(
                            FlexItem::new().grow(1.0),
                            Flex::vertical()
                                .align_content(FlexAlignContent::Stretch)
                                .grow_items(1.0),
                            frame,
                            |flex| {
                                flex.add(FlexItem::new().grow(1.0), Button::new("btn"));
                                flex.add(FlexItem::new(), Button::new("Very long button"));
                            },
                        );

                        flex.add_flex_frame(
                            FlexItem::new().grow(1.0),
                            Flex::vertical()
                                .align_content(FlexAlignContent::Stretch)
                                .grow_items(1.0),
                            frame,
                            |flex| {
                                flex.add_flex_frame(
                                    FlexItem::new().grow(1.0),
                                    Flex::horizontal()
                                        .align_content(FlexAlignContent::Stretch)
                                        .grow_items(1.0),
                                    frame,
                                    |flex| {
                                        flex.add(FlexItem::new().grow(1.0), Button::new("btn"));
                                        flex.add(FlexItem::new(), Button::new("Very long button"));

                                        flex.add_flex_frame(
                                            FlexItem::new().grow(1.0),
                                            Flex::vertical()
                                                .align_content(FlexAlignContent::Stretch)
                                                .grow_items(1.0),
                                            Frame::group(flex.ui().style()),
                                            |flex| {
                                                flex.add(
                                                    FlexItem::new().grow(1.0),
                                                    Button::new("btn"),
                                                );
                                                flex.add(
                                                    FlexItem::new(),
                                                    Button::new("Very long button"),
                                                );
                                            },
                                        );
                                    },
                                );

                                flex.add(FlexItem::new().grow(1.0), Button::new("btn"));
                                flex.add(FlexItem::new(), Button::new("Very long button"));
                                flex.add(FlexItem::new(), Button::new("btn"));
                            },
                        );
                    });

                Flex::vertical().show(ui, |flex| {
                    flex.add_flex_frame(
                        FlexItem::new(),
                        Flex::horizontal()
                            .align_content(FlexAlignContent::Normal)
                            .grow_items(1.0),
                        Frame::group(flex.ui().style()),
                        |flex| {
                            flex.add(FlexItem::new().grow(1.0), Button::new("btn"));
                            flex.add(FlexItem::new(), Button::new("Very long button"));
                        },
                    );
                })
            });
        },
    )
}
