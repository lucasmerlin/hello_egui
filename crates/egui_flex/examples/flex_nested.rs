use egui::{CentralPanel, Frame, Vec2};
use egui_flex::flex_button::FlexButton;
use egui_flex::{Flex, FlexAlignContent, FlexItem};

fn main() -> eframe::Result {
    eframe::run_simple_native("flex nested", Default::default(), |ctx, _frame| {
        CentralPanel::default().show(&ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(10.0);
            Flex::horizontal()
                .align_content(FlexAlignContent::Normal)
                .grow_items()
                .show(ui, |flex| {
                    flex.add_flex(
                        FlexItem::new(),
                        Flex::vertical()
                            .align_content(FlexAlignContent::Stretch)
                            .grow_items(),
                        |flex| {
                            flex.add(FlexItem::new(), FlexButton::new("btn"));
                            flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                        },
                    );

                    flex.add(FlexItem::new().grow(1.0), FlexButton::new("Single Button"));

                    flex.add_flex(
                        FlexItem::new().grow(1.0),
                        Flex::vertical()
                            .align_content(FlexAlignContent::Stretch)
                            .grow_items(),
                        |flex| {
                            flex.add(FlexItem::new().grow(1.0), FlexButton::new("btn"));
                            flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                        },
                    );

                    flex.add_flex(
                        FlexItem::new().grow(1.0),
                        Flex::vertical()
                            .align_content(FlexAlignContent::Stretch)
                            .grow_items(),
                        |flex| {
                            flex.add_flex(
                                FlexItem::new().grow(1.0),
                                Flex::horizontal()
                                    .align_content(FlexAlignContent::Stretch)
                                    .grow_items(),
                                |flex| {
                                    flex.add(FlexItem::new().grow(1.0), FlexButton::new("btn"));
                                    flex.add(FlexItem::new(), FlexButton::new("Very long button"));

                                    flex.add_flex_frame(
                                        FlexItem::new().grow(1.0),
                                        Flex::vertical()
                                            .align_content(FlexAlignContent::Stretch)
                                            .grow_items(),
                                        Frame::group(flex.ui().style()),
                                        |flex| {
                                            flex.add(
                                                FlexItem::new().grow(1.0),
                                                FlexButton::new("btn"),
                                            );
                                            flex.add(
                                                FlexItem::new(),
                                                FlexButton::new("Very long button"),
                                            );
                                        },
                                    );
                                },
                            );

                            flex.add(FlexItem::new().grow(1.0), FlexButton::new("btn"));
                            flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                            flex.add(FlexItem::new(), FlexButton::new("btn"));
                        },
                    );
                });

            Flex::vertical().grow_items().show(ui, |flex| {
                flex.add_flex_frame(
                    FlexItem::new(),
                    Flex::horizontal()
                        .align_content(FlexAlignContent::Normal)
                        .grow_items(),
                    Frame::group(flex.ui().style()),
                    |flex| {
                        flex.add(FlexItem::new().grow(1.0), FlexButton::new("btn"));
                        flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                    },
                );
            })
        });
    })
}
