use crate::crate_ui::{crate_button_ui, ALL_CRATES};
use crate::example::EXAMPLES;
use crate::shared_state::SharedState;
use crate::FancyMessage;
use egui::{Align, Direction, Layout, RichText, Ui};
use egui_dnd::DragDropItem;
use egui_taffy::taffy;
use egui_taffy::taffy::{Display, FlexDirection, FlexWrap, Style};
use std::cell::RefCell;

pub struct SideBar {}

impl SideBar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn ui(&mut self, ui: &mut Ui, shared: &mut SharedState) -> bool {
        let mut clicked = false;
        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
            ui.add_space(4.0);
            ui.heading("hello_egui");
            ui.add_space(4.0);

            ui.label("Examples");
            ui.add_space(4.0);

            ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);

            for category in EXAMPLES.iter() {
                ui.small(category.name);
                for example in category.examples.iter() {
                    let route = format!("/example/{}", example.slug);
                    if ui
                        .selectable_label(
                            shared.active_route == route,
                            RichText::new(example.name).size(14.0),
                        )
                        .clicked()
                    {
                        clicked = true;
                        shared.tx.send(FancyMessage::Navigate(route)).ok();
                    };
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();

        ui.label("Crates in hello_egui");

        let taffy_response = RefCell::new(None);
        let response_ref = &taffy_response;

        let mut taffy = egui_taffy::TaffyPass::new(
            ui,
            "crate_list".into(),
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                gap: taffy::Size::length(8.0),
                ..Default::default()
            },
        );

        for item in ALL_CRATES.iter() {
            let route = format!("/crate/{}", item.name());
            let selected = shared.active_route == route;
            taffy.add(
                item.short_name().id(),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
                Layout::centered_and_justified(Direction::LeftToRight),
                move |ui| {
                    if crate_button_ui(ui, item.short_name(), selected).clicked() {
                        response_ref.borrow_mut().replace(route.clone());
                    }
                },
            );
        }

        taffy.show();

        if let Some(route) = taffy_response.borrow_mut().take() {
            shared.tx.send(FancyMessage::Navigate(route)).ok();
            clicked = true;
        }

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add_space(8.0);
            ui.hyperlink_to(
                "GitHub: hello_egui",
                "https://github.com/lucasmerlin/hello_egui",
            );
        });

        clicked
    }
}
