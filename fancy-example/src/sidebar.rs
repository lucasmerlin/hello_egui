use crate::crate_ui::ALL_CRATES;
use crate::example::EXAMPLES;
use crate::shared_state::SharedState;
use crate::FancyMessage;
use egui::{Align, Button, Layout, RichText, Ui, Vec2};
use egui_flex::{Flex, FlexItem};

pub struct SideBar {}

impl SideBar {
    pub fn ui(ui: &mut Ui, shared: &mut SharedState) -> bool {
        let mut clicked = false;
        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
            ui.add_space(4.0);
            ui.heading("hello_egui");
            ui.add_space(4.0);

            ui.label("Examples");
            ui.add_space(4.0);

            ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);

            for category in EXAMPLES {
                ui.small(category.name);
                for example in category.examples {
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

        clicked |= crate_list_ui(ui, shared);

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

pub fn crate_list_ui(ui: &mut Ui, shared: &SharedState) -> bool {
    let mut clicked = false;
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);
        ui.spacing_mut().item_spacing = Vec2::splat(8.0);

        Flex::horizontal()
            .grow_items(1.0)
            .wrap(true)
            .show(ui, |flex| {
                for item in ALL_CRATES {
                    let route = format!("/crate/{}", item.name());
                    let selected = shared.active_route == route;

                    if flex
                        .add(
                            FlexItem::new(),
                            Button::new(item.short_name())
                                .selected(selected)
                                .corner_radius(16.0),
                        )
                        .clicked()
                    {
                        shared.tx.send(FancyMessage::Navigate(route)).ok();
                        clicked = true;
                    };
                }
                // Add a final grow item to fill the remaining space on the last row
                flex.add_ui(FlexItem::new().grow(9999.0), |_| {});
            });
    });
    clicked
}
