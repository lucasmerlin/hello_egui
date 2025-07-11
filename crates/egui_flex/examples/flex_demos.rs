use eframe::NativeOptions;
use egui::{Align2, Button, CentralPanel, ComboBox, Frame, Label};
use egui_flex::{
    item, Flex, FlexAlign, FlexAlignContent, FlexDirection, FlexInstance, FlexItem, FlexJustify,
};
use std::num::NonZeroUsize;

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
            ctx.options_mut(|opts| {
                opts.max_passes = NonZeroUsize::new(3).unwrap();
            });

            CentralPanel::default().show(ctx, |ui| {
                let frame = Frame::group(ui.style());

                Flex::new().w_full().show(ui, |flex| {
                    flex.add_ui(FlexItem::new(), |ui| {
                        ui.label("Demo direction:");
                    });
                    if flex
                        .add(
                            FlexItem::new(),
                            Button::new("Horizontal")
                                .selected(demo_dir == FlexDirection::Horizontal),
                        )
                        .clicked()
                    {
                        demo_dir = FlexDirection::Horizontal;
                    }
                    if flex
                        .add(
                            FlexItem::new(),
                            Button::new("Vertical").selected(demo_dir == FlexDirection::Vertical),
                        )
                        .clicked()
                    {
                        demo_dir = FlexDirection::Vertical;
                    }

                    flex.add_ui(FlexItem::new(), |ui| ui.checkbox(&mut grow, "Grow"));
                });

                let main_dir = if demo_dir == FlexDirection::Horizontal {
                    FlexDirection::Vertical
                } else {
                    FlexDirection::Horizontal
                };
                let grow_items = if grow { 1.0 } else { 0.0 };

                let heading = |flex: &mut FlexInstance, heading| {
                    flex.add_ui(FlexItem::new().grow(0.0), |ui| {
                        ui.heading(heading);
                    });
                };

                Flex::new()
                    .direction(main_dir)
                    .align_content(FlexAlignContent::Start)
                    .grow_items(1.0)
                    .wrap(true)
                    .show(ui, |flex| {
                        flex.add_flex(
                            FlexItem::new().frame(frame),
                            Flex::new().direction(demo_dir).grow_items(grow_items),
                            |flex| {
                                heading(flex, "Align");

                                for align in &[
                                    FlexAlign::Start,
                                    FlexAlign::Center,
                                    FlexAlign::End,
                                    FlexAlign::Stretch,
                                ] {
                                    flex.add_ui(
                                        FlexItem::new().align_self(*align).frame(frame),
                                        |ui| {
                                            ui.label(format!("{align:?}"));
                                        },
                                    );
                                }

                                flex.add_ui(FlexItem::new().frame(frame), |ui| {
                                    ui.label("Some bigger item\nwith some\nmore lines")
                                });
                            },
                        );

                        flex.add_flex(
                            FlexItem::new().frame(frame),
                            Flex::new().w_full().h_full().direction(demo_dir),
                            |flex| {
                                heading(flex, "Grow");

                                for grow in &[0.0, 1.0, 2.0, 3.0] {
                                    flex.add_ui(FlexItem::new().grow(*grow).frame(frame), |ui| {
                                        ui.label(format!("{grow:?}"));
                                    });
                                }
                            },
                        );

                        flex.add_flex(
                            FlexItem::new().frame(frame),
                            Flex::new().direction(demo_dir).grow_items(grow_items),
                            |flex| {
                                heading(flex, "Basis");

                                for basis in &[0.0, 50.0, 100.0, 200.0] {
                                    flex.add_ui(FlexItem::new().basis(*basis).frame(frame), |ui| {
                                        ui.label(format!("{basis:?}"));
                                    });
                                }
                            },
                        );

                        flex.add_flex(
                            FlexItem::new().frame(frame),
                            Flex::new().direction(demo_dir).grow_items(grow_items),
                            |flex| {
                                heading(flex, "Justify Content");

                                flex.add_flex(
                                    item().grow(1.0).frame(frame),
                                    Flex::new()
                                        .direction(main_dir)
                                        .grow_items(grow_items)
                                        .align_items(FlexAlign::Stretch),
                                    |flex| {
                                        for justify in &[
                                            FlexJustify::Start,
                                            FlexJustify::Center,
                                            FlexJustify::End,
                                            FlexJustify::SpaceBetween,
                                            FlexJustify::SpaceAround,
                                            FlexJustify::SpaceEvenly,
                                        ] {
                                            flex.add_flex(
                                                FlexItem::new().frame(frame),
                                                Flex::new().direction(demo_dir).justify(*justify),
                                                |flex| {
                                                    flex.add(
                                                        item(),
                                                        Label::new(format!("{justify:?}")),
                                                    );
                                                    flex.add(item(), Label::new("two"));
                                                    flex.add(item(), Label::new("three"));
                                                },
                                            );
                                        }
                                    },
                                );
                            },
                        );

                        flex.add_flex(
                            FlexItem::new().frame(frame),
                            Flex::new().direction(demo_dir).grow_items(grow_items),
                            |flex| {
                                heading(flex, "Nested");

                                flex.add_flex(
                                    FlexItem::new().frame(frame),
                                    Flex::new().direction(main_dir).grow_items(grow_items),
                                    |flex| {
                                        flex.add_ui(FlexItem::new().frame(frame), |ui| {
                                            ui.label("one");
                                        });
                                        flex.add_flex(
                                            FlexItem::new().frame(frame),
                                            Flex::new().direction(demo_dir).grow_items(grow_items),
                                            |flex| {
                                                flex.add_ui(FlexItem::new().frame(frame), |ui| {
                                                    ui.label("two");
                                                });
                                                flex.add_ui(FlexItem::new().frame(frame), |ui| {
                                                    ui.label("three");
                                                });
                                            },
                                        );
                                    },
                                );
                                flex.add_flex(
                                    FlexItem::new().frame(frame),
                                    Flex::new().direction(main_dir).grow_items(grow_items),
                                    |flex| {
                                        flex.add_ui(FlexItem::new().frame(frame), |ui| {
                                            ui.label("one");
                                        });
                                        flex.add_ui(FlexItem::new().frame(frame), |ui| {
                                            ui.label("two");
                                        });
                                        flex.add_ui(FlexItem::new().frame(frame), |ui| {
                                            ui.label("three");
                                        });
                                    },
                                );
                            },
                        );

                        flex.add_flex(
                            FlexItem::new().frame(frame),
                            Flex::new().direction(demo_dir).grow_items(grow_items),
                            |flex| {
                                heading(flex, "Align Self Content");

                                flex.add_ui(FlexItem::new(), |ui| {
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
                                    flex.add_ui(
                                        FlexItem::new()
                                            .align_self(*align)
                                            .align_self_content(align_content)
                                            .frame(frame),
                                        |ui| {
                                            ui.label(format!("{align:?}"));
                                        },
                                    );
                                }

                                flex.add_ui(FlexItem::new().frame(frame), |ui| {
                                    ui.label("Some bigger item\nwith some\nmore lines")
                                });
                            },
                        );
                    });
            });
        },
    )
}
