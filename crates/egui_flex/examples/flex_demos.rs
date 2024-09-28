use eframe::NativeOptions;
use egui::{Align2, Button, CentralPanel, ComboBox, Frame};
use egui_flex::{Flex, FlexAlign, FlexAlignContent, FlexDirection, FlexInstance, FlexItem};

const ALIGNS: [Align2; 9] = [
    Align2::LEFT_TOP,
    Align2::CENTER_TOP,
    Align2::RIGHT_TOP,
    Align2::LEFT_CENTER,
    Align2::CENTER_CENTER,
    Align2::RIGHT_CENTER,
    Align2::LEFT_BOTTOM,
    Align2::CENTER_BOTTOM,
    Align2::RIGHT_BOTTOM,
];

#[allow(clippy::too_many_lines)] // It's an example
fn main() -> eframe::Result {
    let mut demo_dir = FlexDirection::Horizontal;

    let mut grow = false;

    let mut align_self_content: usize = 0;

    eframe::run_simple_native(
        "egui_flex demos",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                let frame = Frame::group(ui.style());

                Flex::new().show(ui, |flex| {
                    flex.add_simple(FlexItem::new(), |ui| {
                        ui.label("Demo direction:");
                    });
                    if flex
                        .add(
                            FlexItem::new(),
                            Button::new("Horizontal")
                                .selected(demo_dir == FlexDirection::Horizontal),
                        )
                        .inner
                        .clicked()
                    {
                        demo_dir = FlexDirection::Horizontal;
                    };
                    if flex
                        .add(
                            FlexItem::new(),
                            Button::new("Vertical").selected(demo_dir == FlexDirection::Vertical),
                        )
                        .inner
                        .clicked()
                    {
                        demo_dir = FlexDirection::Vertical;
                    };

                    flex.add_simple(FlexItem::new(), |ui| ui.checkbox(&mut grow, "Grow"));
                });

                let main_dir = if demo_dir == FlexDirection::Horizontal {
                    FlexDirection::Vertical
                } else {
                    FlexDirection::Horizontal
                };
                let grow_items = if grow { 1.0 } else { 0.0 };

                let heading = |flex: &mut FlexInstance, heading| {
                    flex.add_simple(FlexItem::new().grow(0.0), |ui| {
                        ui.heading(heading);
                    });
                };

                Flex::new()
                    .direction(main_dir)
                    .align_content(FlexAlignContent::Normal)
                    .grow_items(1.0)
                    .show(ui, |flex| {
                        flex.add_flex_frame(
                            FlexItem::new(),
                            Flex::new().direction(demo_dir).grow_items(grow_items),
                            frame,
                            |flex| {
                                heading(flex, "Align");

                                for align in &[
                                    FlexAlign::Start,
                                    FlexAlign::Center,
                                    FlexAlign::End,
                                    FlexAlign::Stretch,
                                ] {
                                    flex.add_frame(
                                        FlexItem::new().align_self(*align),
                                        frame,
                                        |ui| {
                                            ui.label(format!("{align:?}"));
                                        },
                                    );
                                }

                                flex.add_frame(FlexItem::new(), frame, |ui| {
                                    ui.label("Some bigger item\nwith some\nmore lines")
                                });
                            },
                        );

                        flex.add_flex_frame(
                            FlexItem::new(),
                            Flex::new().direction(demo_dir),
                            frame,
                            |flex| {
                                heading(flex, "Grow");

                                for grow in &[0.0, 1.0, 2.0, 3.0] {
                                    flex.add_frame(FlexItem::new().grow(*grow), frame, |ui| {
                                        ui.label(format!("{grow:?}"));
                                    });
                                }
                            },
                        );

                        flex.add_flex_frame(
                            FlexItem::new(),
                            Flex::new().direction(demo_dir).grow_items(grow_items),
                            frame,
                            |flex| {
                                heading(flex, "Basis");

                                for basis in &[0.0, 50.0, 100.0, 200.0] {
                                    flex.add_frame(FlexItem::new().basis(*basis), frame, |ui| {
                                        ui.label(format!("{basis:?}"));
                                    });
                                }
                            },
                        );

                        flex.add_flex_frame(
                            FlexItem::new(),
                            Flex::new().direction(demo_dir).grow_items(grow_items),
                            frame,
                            |flex| {
                                heading(flex, "Nested");

                                flex.add_flex_frame(
                                    FlexItem::new(),
                                    Flex::new().direction(main_dir).grow_items(grow_items),
                                    frame,
                                    |flex| {
                                        flex.add_frame(FlexItem::new(), frame, |ui| {
                                            ui.label("one");
                                        });
                                        flex.add_flex_frame(
                                            FlexItem::new(),
                                            Flex::new().direction(demo_dir).grow_items(grow_items),
                                            frame,
                                            |flex| {
                                                flex.add_frame(FlexItem::new(), frame, |ui| {
                                                    ui.label("two");
                                                });
                                                flex.add_frame(FlexItem::new(), frame, |ui| {
                                                    ui.label("three");
                                                });
                                            },
                                        );
                                    },
                                );
                                flex.add_flex_frame(
                                    FlexItem::new(),
                                    Flex::new().direction(main_dir).grow_items(grow_items),
                                    frame,
                                    |flex| {
                                        flex.add_frame(FlexItem::new(), frame, |ui| {
                                            ui.label("one");
                                        });
                                        flex.add_frame(FlexItem::new(), frame, |ui| {
                                            ui.label("two");
                                        });
                                        flex.add_frame(FlexItem::new(), frame, |ui| {
                                            ui.label("three");
                                        });
                                    },
                                );
                            },
                        );

                        flex.add_flex_frame(
                            FlexItem::new(),
                            Flex::new().direction(demo_dir).grow_items(grow_items),
                            frame,
                            |flex| {
                                heading(flex, "Align Self Content");

                                flex.add_simple(FlexItem::new(), |ui| {
                                    ComboBox::new("self content", "").show_index(
                                        ui,
                                        &mut align_self_content,
                                        ALIGNS.len(),
                                        |index| format!("{:?}", ALIGNS[index]),
                                    );
                                });

                                let align_content = ALIGNS[align_self_content];

                                for align in &[
                                    FlexAlign::Start,
                                    FlexAlign::Center,
                                    FlexAlign::End,
                                    FlexAlign::Stretch,
                                ] {
                                    flex.add_frame(
                                        FlexItem::new()
                                            .align_self(*align)
                                            .align_self_content(align_content),
                                        frame,
                                        |ui| {
                                            ui.label(format!("{align:?}"));
                                        },
                                    );
                                }

                                flex.add_frame(FlexItem::new(), frame, |ui| {
                                    ui.label("Some bigger item\nwith some\nmore lines")
                                });
                            },
                        );
                    });
            });
        },
    )
}
