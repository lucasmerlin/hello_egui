#![allow(clippy::too_many_lines)] // It's ok as it is an example
use eframe::emath::Align;
use eframe::{egui, NativeOptions};
use egui::{CentralPanel, Color32, Direction, Frame, Id, Layout, Resize, ScrollArea, Ui};
use rand::seq::IndexedRandom;
use taffy::prelude::{
    AlignContent, AlignItems, Dimension, FlexDirection, FlexWrap, JustifyContent, LengthPercentage,
    Rect, Size,
};
use taffy::style::{AlignSelf, Display, JustifyItems, JustifySelf, Style};

use egui_taffy::TaffyPass;

const LOREM: &str = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.";

const TEXTS: [&str; 5] = [
    "You can",
    "have",
    "buttons in varying",
    "sizes",
    "flow nicely in your layout",
];
pub fn main() -> eframe::Result<()> {
    let buttons: Vec<_> = TEXTS.iter().map(std::string::ToString::to_string).collect();

    let many_buttons = (0..100).fold(Vec::new(), |mut acc, _| {
        acc.push(buttons.choose(&mut rand::rng()).unwrap().to_string());
        acc
    });

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(ui.available_height());

                    let mut taffy = TaffyPass::new(
                        ui,
                        Id::new("flexible"),
                        Style {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            size: Size {
                                width: Dimension::Percent(1.0),
                                height: Dimension::Auto,
                            },

                            gap: Size {
                                width: LengthPercentage::Length(10.0),
                                height: LengthPercentage::Length(10.0),
                            },

                            ..Default::default()
                        },
                    );

                    taffy.add(
                        Id::new("child_2"),
                        Style {
                            flex_grow: 1.0,
                            ..Default::default()
                        },
                        Layout::centered_and_justified(Direction::TopDown),
                        |ui| {
                            let _ = ui.button("Button 2 With long text");
                        },
                    );

                    taffy.add_children_with_ui(
                        Style {
                            justify_self: Some(JustifySelf::Stretch),
                            align_self: Some(AlignSelf::Stretch),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            justify_content: Some(JustifyContent::Stretch),
                            align_content: Some(AlignContent::Stretch),
                            justify_items: Some(JustifyItems::Stretch),
                            align_items: Some(AlignItems::Stretch),
                            size: Size {
                                width: Dimension::Auto,
                                height: Dimension::Auto,
                            },

                            gap: Size {
                                width: LengthPercentage::Length(10.0),
                                height: LengthPercentage::Length(10.0),
                            },

                            flex_grow: 0.4,

                            padding: Rect::length(10.0),
                            //margin: Rect::length(10.0),
                            ..Default::default()
                        },
                        |ui| {
                            Frame::group(ui.style()).show(ui, |ui| {
                                ui.set_min_size(ui.available_size());
                            });
                        },
                        |taffy| {
                            taffy.add(
                                Id::new("child_1"),
                                Style {
                                    flex_grow: 1.0,
                                    size: Size {
                                        width: Dimension::Auto,
                                        height: Dimension::Length(30.0),
                                    },
                                    ..Default::default()
                                },
                                Layout::centered_and_justified(Direction::TopDown),
                                |ui| {
                                    let _ = ui.button("Button 1");
                                },
                            );
                            taffy.add(
                                Id::new("child_2"),
                                Style {
                                    flex_grow: 1.0,
                                    ..Default::default()
                                },
                                Layout::centered_and_justified(Direction::TopDown),
                                |ui| {
                                    let _ = ui.button("Button 2 With long text");
                                },
                            );
                        },
                    );
                    taffy.add(
                        Id::new(1),
                        Style {
                            align_self: Some(AlignItems::Center),
                            justify_self: Some(JustifySelf::Center),
                            ..Default::default()
                        },
                        Layout::top_down(Align::Center),
                        |ui| {
                            let _ = ui.button("Button 1");
                            let _ = ui.button("Button 2");
                        },
                    );
                    taffy.add(
                        Id::new(2),
                        Style {
                            flex_grow: 1.0,
                            align_self: Some(AlignItems::End),
                            size: Size {
                                width: Dimension::Auto,
                                height: Dimension::Length(100.0),
                            },
                            ..Default::default()
                        },
                        Layout::centered_and_justified(Direction::TopDown),
                        |ui| {
                            let _ = ui.button("Button 2");
                        },
                    );
                    taffy.show();

                    ui.separator();

                    let mut flex = TaffyPass::new(
                        ui,
                        Id::new("flex"),
                        Style {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            justify_content: Some(JustifyContent::Stretch),
                            gap: Size {
                                width: LengthPercentage::Length(10.0),
                                height: LengthPercentage::Length(10.0),
                            },
                            align_items: Some(AlignItems::Start),
                            size: Size {
                                width: Dimension::Percent(1.0),
                                height: Dimension::Auto,
                            },
                            flex_wrap: FlexWrap::Wrap,
                            ..Default::default()
                        },
                    );

                    for (i, button) in many_buttons.iter().enumerate() {
                        flex.add(
                            Id::new(i),
                            Style {
                                flex_grow: 1.0,
                                ..Default::default()
                            },
                            Layout::centered_and_justified(Direction::TopDown),
                            move |ui| {
                                let _ = ui.button(button);
                            },
                        );
                    }

                    flex.show();

                    ui.separator();

                    Resize::default().show(ui, list_example);
                });
            });
        },
    )
}

fn list_example(ui: &mut Ui) {
    let texts: Vec<_> = [2, 10, 20, 5, 18]
        .iter()
        .map(|words| {
            let text = LOREM
                .split(' ')
                .take(*words)
                .collect::<Vec<&str>>()
                .join(" ");
            text
        })
        .collect();

    let texts: Vec<_> = texts.iter().map(std::string::String::as_str).collect();

    {
        let mut taffy = TaffyPass::new(
            ui,
            Id::new("List Example"),
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Auto,
                },

                padding: Rect::length(10.0),
                gap: Size::length(10.0),

                ..Default::default()
            },
        );

        for text in &texts {
            taffy.add_children_with_ui(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,

                    align_items: Some(AlignItems::Start),
                    gap: Size::length(8.0),
                    padding: Rect::length(8.0),
                    margin: Rect::length(4.0),

                    ..Default::default()
                },
                |ui| {
                    Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_min_size(ui.available_size());
                    });
                },
                |taffy| {
                    taffy.add(
                        Id::new("icon"),
                        Style {
                            size: Size {
                                width: Dimension::Length(40.0),
                                height: Dimension::Length(40.0),
                            },
                            flex_shrink: 0.0,
                            ..Default::default()
                        },
                        Layout::centered_and_justified(Direction::TopDown),
                        |ui| {
                            Frame::NONE
                                .fill(ui.style().visuals.warn_fg_color)
                                .corner_radius(30.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(ui.available_size());
                                });
                        },
                    );

                    taffy.add(
                        Id::new("text"),
                        Style {
                            align_self: Some(AlignItems::Center),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            ..Default::default()
                        },
                        Layout::left_to_right(Align::Center).with_main_wrap(true),
                        |ui| {
                            ui.heading(*text);
                        },
                    );

                    for i in 1..3 {
                        taffy.add(
                            Id::new(i),
                            Style {
                                size: Size {
                                    width: Dimension::Length(30.0),
                                    height: Dimension::Length(30.0),
                                },
                                flex_shrink: 0.0,
                                ..Default::default()
                            },
                            Layout::centered_and_justified(Direction::TopDown),
                            |ui| {
                                Frame::NONE
                                    .fill(Color32::from_gray(100))
                                    .corner_radius(30.0)
                                    .show(ui, |ui| {
                                        ui.set_min_size(ui.available_size());
                                    });
                            },
                        );
                    }
                },
            );
        }

        taffy.show();
    }
}
