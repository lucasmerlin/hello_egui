use crate::crate_ui::ALL_CRATES;
use crate::example::EXAMPLES;
use crate::shared_state::SharedState;
use crate::FancyMessage;
use egui::style::WidgetVisuals;
use egui::{Align, Layout, RichText, Ui, Vec2};
use egui_flex::flex_button::FlexButton;
use egui_flex::{Flex, FlexItem};

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

        ui.scope(|ui| {
            let set = |visuals: &mut WidgetVisuals| {
                visuals.rounding = 16.0.into();
            };
            let visuals = ui.visuals_mut();
            set(&mut visuals.widgets.inactive);
            set(&mut visuals.widgets.hovered);
            set(&mut visuals.widgets.active);
            set(&mut visuals.widgets.noninteractive);
            set(&mut visuals.widgets.open);

            ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);
            ui.spacing_mut().item_spacing = Vec2::splat(8.0);

            Flex::horizontal().grow_items(1.0).show(ui, |flex| {
                for item in ALL_CRATES.iter() {
                    let route = format!("/crate/{}", item.name());
                    let selected = shared.active_route == route;

                    if flex
                        .add(
                            FlexItem::new(),
                            FlexButton::new(item.short_name()).selected(selected),
                        )
                        .inner
                        .clicked()
                    {
                        shared.tx.send(FancyMessage::Navigate(route)).ok();
                        clicked = true;
                    };
                }
            });
        });

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
