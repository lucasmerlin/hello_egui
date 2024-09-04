use eframe::NativeOptions;
use egui::{CentralPanel, Label, TextEdit, Widget};
use egui_flex::{Flex, FlexItem};

fn main() -> eframe::Result {
    let mut text = "Hello, world!".to_owned();
    eframe::run_simple_native(
        "flex example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.vertical(|ui| {
                    let items = vec![
                        "I",
                        "can have veeeeeeeeeeeeery long",
                        "and",
                        "very",
                        "short",
                        "and",
                        "even",
                        "some middle length",
                        "items",
                    ];

                    Flex::new().show(ui, |flex| {
                        flex.add_container(FlexItem::default().grow(1.0), |ui, content| {
                            ui.group(|ui| {
                                content.content(ui, |ui| {
                                    ui.label("Hello");
                                })
                            })
                            .inner
                        });

                        for item in items {
                            flex.add_container(FlexItem::default().grow(1.0), |ui, content| {
                                ui.group(|ui| {
                                    content.content(ui, |ui| {
                                        Label::new(item).wrap().ui(ui);
                                    })
                                })
                                .inner
                            });
                        }

                        flex.add_container(
                            FlexItem::default().grow(1.0).basis(100.0),
                            |ui, content| {
                                ui.group(|ui| {
                                    content.content(ui, |ui| {
                                        TextEdit::singleline(&mut text)
                                            // .desired_width(ui.available_width())
                                            .ui(ui);
                                    })
                                })
                                .inner
                            },
                        );

                        flex.add_container(
                            FlexItem::default().grow(1.0).basis(100.0),
                            |ui, content| {
                                ui.group(|ui| {
                                    content.content(ui, |ui| {
                                        ui.label("I have flex basis 100");
                                    })
                                })
                                .inner
                            },
                        );

                        // flex.add_container(
                        //     FlexItem::default().grow(1.0).basis(100.0),
                        //     |ui, content| {
                        //         ui.group(|ui| {
                        //             content.content(ui, |ui| {
                        //                 ui.vertical(|ui| {
                        //                     Flex::new().show(ui, |flex| {
                        //                         flex.add_simple(FlexItem::default(), |ui| {
                        //                             ui.label("Hello World");
                        //                         });
                        //
                        //                         flex.add_simple(FlexItem::default(), |ui| {
                        //                             ui.label("Drop");
                        //                         });
                        //
                        //                         flex.add_simple(FlexItem::default(), |ui| {
                        //                             ui.label("Two \nLines");
                        //                         });
                        //                     });
                        //                 });
                        //             })
                        //         })
                        //         .inner
                        //     },
                        // );
                    });
                });

                ui.horizontal_top(|ui| {
                    Flex::new().show(ui, |flex| {
                        for i in 0..1000 {
                            flex.add_simple(FlexItem::default().grow(1.0), |ui| {
                                ui.label(i.to_string());
                            });
                        }
                        flex.add_simple(FlexItem::default().grow(1000000.0), |ui| {});
                    });
                });

                ui.button("Button");
            });
        },
    )
}
