use eframe::egui;
use eframe::emath::Align;
use egui::{CentralPanel, Direction, Id, Layout, ScrollArea};
use egui_taffy::TaffyPass;
use rand::prelude::SliceRandom;
use taffy::prelude::{
    AlignItems, Dimension, FlexDirection, FlexWrap, JustifyContent, LengthPercentage, Size,
};
use taffy::style::{Display, JustifySelf, Style};
use taffy::Taffy;

pub fn main() -> eframe::Result<()> {
    let mut buttons: Vec<_> = [
        "You can",
        "have",
        "buttons in varying",
        "sizes",
        "flow nicely in yor layout",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let mut many_buttons = (0..100).fold(Vec::new(), |mut acc, _| {
        acc.push(buttons.choose(&mut rand::thread_rng()).unwrap().to_string());
        acc
    });

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(ui.available_height());

                    ui.label("Hey");

                    TaffyPass::new(
                        ui,
                        Id::new("flexible"),
                        Style {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            size: Size {
                                width: Dimension::Percent(1.0),
                                height: Dimension::Auto,
                            },

                            ..Default::default()
                        },
                    )
                    .child(
                        Id::new(1),
                        Style {
                            align_self: Some(AlignItems::Center),
                            justify_self: Some(JustifySelf::Center),
                            ..Default::default()
                        },
                        Layout::top_down(Align::Center),
                        |ui| {
                            ui.button("Button 1");
                            ui.button("Button 2");
                        },
                    )
                    .child(
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
                            ui.button("Button 2");
                        },
                    )
                    .show();

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
                        flex = flex.child(
                            Id::new(i),
                            Style {
                                flex_grow: 1.0,
                                ..Default::default()
                            },
                            Layout::centered_and_justified(Direction::TopDown),
                            move |ui| {
                                ui.button(&*button);
                            },
                        );
                    }

                    flex.show();
                });
            });
        },
    )
}
