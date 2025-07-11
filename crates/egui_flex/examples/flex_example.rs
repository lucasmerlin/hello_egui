use eframe::NativeOptions;
use egui::{Button, CentralPanel, Checkbox, Frame, Label, Slider, TextEdit, Widget};
use egui_flex::{item, Flex, FlexAlign, FlexItem};
use std::num::NonZeroUsize;

#[allow(clippy::too_many_lines)] // It's an example
fn main() -> eframe::Result {
    let mut text = "Hello, world!".to_owned();

    let mut toggle = false;

    eframe::run_simple_native(
        "flex example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ctx.options_mut(|opts| {
                    opts.max_passes = NonZeroUsize::new(3).unwrap();
                });

                ui.horizontal_top(|ui| {
                    let items = vec![
                        "I",
                        "can have veeeeeeeeeeeeery long",
                        "and",
                        "very",
                        "short",
                        "and",
                        "multi\nline",
                        "or\neven\nmore\nlines",
                        "and",
                        "even",
                        "some middle length",
                        "items",
                    ];

                    Flex::new()
                        .w_full()
                        .align_items(egui_flex::FlexAlign::Stretch)
                        .align_items_content(egui::Align2::CENTER_CENTER)
                        .wrap(true)
                        .show(ui, |flex| {
                            flex.add_ui(
                                FlexItem::default()
                                    .grow(1.0)
                                    .frame(Frame::group(flex.ui().style())),
                                |ui| {
                                    ui.label("Hello");
                                },
                            );

                            for item in items {
                                flex.add_ui(
                                    FlexItem::default()
                                        .grow(1.0)
                                        .frame(Frame::group(flex.ui().style())),
                                    |ui| {
                                        Label::new(item).wrap().ui(ui);
                                    },
                                );
                            }

                            let response = flex.add_widget(
                                item().grow(1.0),
                                Button::new(if toggle {
                                    "short"
                                } else {
                                    "You can toggle my size"
                                }),
                            );
                            // let response = flex.add_ui(item().grow(1.0), |ui| {
                            //     Button::new(if toggle {
                            //         "short"
                            //     } else {
                            //         "You can toggle my size"
                            //     })
                            //     .ui(ui)
                            // });
                            if response.inner.clicked() {
                                toggle = !toggle;
                            }

                            flex.add_ui(
                                FlexItem::default()
                                    .grow(1.0)
                                    .basis(200.0)
                                    .frame(Frame::group(flex.ui().style())),
                                |ui| {
                                    TextEdit::singleline(&mut text)
                                        .desired_width(ui.available_width())
                                        .ui(ui);
                                },
                            );

                            flex.add_ui(
                                FlexItem::default()
                                    .grow(1.0)
                                    .basis(80.0)
                                    .frame(Frame::group(flex.ui().style())),
                                |ui| {
                                    ui.add(Label::new("I have flex basis 80").wrap());
                                },
                            );

                            for align in &[
                                FlexAlign::Start,
                                FlexAlign::End,
                                FlexAlign::Center,
                                FlexAlign::Stretch,
                            ] {
                                flex.add_ui(
                                    FlexItem::default()
                                        .grow(1.0)
                                        .align_self(*align)
                                        .frame(Frame::group(flex.ui().style())),
                                    |ui| {
                                        ui.add(
                                            Label::new(format!("I have align-self: {align:?}"))
                                                .wrap(),
                                        );
                                    },
                                );
                            }

                            flex.add_ui(FlexItem::new().grow(1.0).basis(150.0), |ui| {
                                ui.style_mut().spacing.slider_width = ui.available_width() - 50.0;
                                Slider::new(&mut 0.0, 0.0..=1000.0).ui(ui);
                            });

                            flex.add_flex(
                                FlexItem::default()
                                    .grow(1.0)
                                    .frame(egui::Frame::group(flex.ui().style())),
                                Flex::vertical()
                                    .align_content(egui_flex::FlexAlignContent::Stretch)
                                    .grow_items(1.0),
                                |flex| {
                                    flex.add(FlexItem::default().grow(1.0), Button::new("btn"));
                                    flex.add(FlexItem::default(), Button::new("Very long button"));
                                    flex.add_flex(
                                        FlexItem::default().grow(1.0),
                                        Flex::horizontal()
                                            .align_content(egui_flex::FlexAlignContent::Stretch)
                                            .w_full()
                                            .grow_items(1.0),
                                        |flex| {
                                            flex.add(
                                                FlexItem::default().grow(1.0),
                                                Button::new("btn"),
                                            );
                                            flex.add(
                                                FlexItem::default(),
                                                Button::new("Very long button"),
                                            );
                                        },
                                    );
                                },
                            );

                            flex.add(FlexItem::new().grow(1.0), Button::new("Very long button"));

                            flex.add(FlexItem::new().grow(1.0), Button::new("Button"));
                            flex.add(
                                FlexItem::new().grow(1.0),
                                Button::new("Button wefoijfgiweopjg"),
                            );
                            flex.add(FlexItem::new().grow(1.0), Button::new("Button"));
                            flex.add(FlexItem::new(), Button::new("Non-grow Button"));

                            flex.add(FlexItem::new(), Checkbox::new(&mut false, "Checkbox"));

                            // flex.add_container(
                            //     FlexItem::default().grow(1.0).basis(100.0),
                            //     |ui, content| {
                            //         ui.group(|ui| {
                            //             content.content(ui, |ui| {
                            //                 ui.vertical(|ui| {
                            //                     Flex::new().show(ui, |flex| {
                            //                         flex.add(
                            //                             FlexItem::new(),
                            //                             Button::new("Button"),
                            //                         );
                            //
                            //                         flex.add(
                            //                             FlexItem::new(),
                            //                             Button::new("Longer Button"),
                            //                         );
                            //
                            //                         flex.add(
                            //                             FlexItem::new(),
                            //                             Button::new(
                            //                                 "Button\nwith\nmultiple\nlines",
                            //                             ),
                            //                         );
                            //                     });
                            //                 });
                            //             })
                            //         })
                            //         .inner
                            //     },
                            // );
                        });
                });

                // ui.horizontal_top(|ui| {
                //     Flex::new().show(ui, |flex| {
                //         for i in 0..1000 {
                //             flex.add_ui(FlexItem::default().grow(1.0), |ui| {
                //                 ui.label(i.to_string());
                //             });
                //         }
                //         flex.add_ui(FlexItem::default().grow(1000000.0), |ui| {});
                //     });
                // });

                ui.horizontal_wrapped(|ui| {
                    let _ = ui.button("Normal Button");
                    Flex::horizontal().show(ui, |flex| {
                        flex.add(FlexItem::new(), Button::new("Hello"));
                    });
                    let _ = ui.button("Normal Button");
                });

                let _ = ui.button("Button");

                // Window::new("Window").show(ui.ctx(), |ui| {
                //     Flex::horizontal().show(ui, |flex| {
                //         flex.add(FlexItem::new(), Button::new("Button"));
                //         flex.add(FlexItem::new(), Button::new("Button"));
                //         flex.add(FlexItem::new(), Button::new("Button"));
                //     });
                // });

                // Area::new(Id::new("area")).show(ui.ctx(), |ui| {
                //     Flex::horizontal().wrap(false).show(ui, |flex| {
                //         flex.add(FlexItem::new(), FlexButton::new("Button"));
                //         flex.add(FlexItem::new(), FlexButton::new("Button"));
                //         flex.add(FlexItem::new(), FlexButton::new("Button"));
                //     });
                // });
            });
        },
    )
}
