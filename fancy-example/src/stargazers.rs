use std::fmt::Debug;
use std::hash::Hash;

use egui::{Frame, Id, Image, ScrollArea, Ui, Vec2};
use ehttp::Request;
use serde::Deserialize;

use crate::crate_ui::{crate_usage_ui, Crate, CrateUsage};
use crate::demo_area;
use crate::example::{Example, ExampleTrait};
use crate::shared_state::SharedState;
use egui_dnd::{dnd, DragDropConfig};
use egui_infinite_scroll::InfiniteScroll;

pub const STARGAZERS_EXAMPLE: Example = Example {
    name: "Stargazers",
    slug: "stargazers",
    crates: &[
        CrateUsage::simple(Crate::EguiDnd),
        CrateUsage::simple(Crate::EguiInfiniteScroll),
    ],
    get: || Box::new(Stargazers::new()),
};

#[derive(Deserialize, Debug)]
pub struct Stargazer {
    pub login: String,
    pub html_url: String,
    pub avatar_url: String,
}

pub fn example_stargazers() -> Vec<Stargazer> {
    vec![Stargazer {
        login: "lucasmerlin".to_string(),
        html_url: "https://github.com/lucasmerlin".to_string(),
        avatar_url: "https://avatars.githubusercontent.com/u/8009393?v=4".to_string(),
    }]
}

impl Hash for Stargazer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.login.hash(state);
    }
}

pub struct Stargazers {
    infinite_scroll: InfiniteScroll<Stargazer, usize>,
}

impl ExampleTrait for Stargazers {
    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState) {
        self.stargazers_ui(ui, shared_state);
    }
}

impl Default for Stargazers {
    fn default() -> Self {
        Self::new()
    }
}

impl Stargazers {
    pub fn new() -> Self {
        let mut infinite_scroll = InfiniteScroll::new();
        infinite_scroll.virtual_list.hide_on_resize(None);

        Self {
            infinite_scroll: infinite_scroll.end_loader(|cursor, callback| {
                if cfg!(feature = "mock") {
                    callback(Ok((example_stargazers(), None)));
                } else {
                    ehttp::fetch(
                        Request::get(format!(
                            "https://api.github.com/repos/lucasmerlin\
                            /hello_egui/stargazers?per_page=100&page={}",
                            cursor.unwrap_or(1)
                        )),
                        move |result| {
                            if let Ok(data) = result {
                                if let Ok(stargazers) =
                                    serde_json::from_slice::<Vec<Stargazer>>(&data.bytes)
                                {
                                    callback(Ok((stargazers, Some(cursor.unwrap_or(1) + 1))));
                                } else {
                                    callback(Err("Failed to parse stargazers".to_string()));
                                }
                            } else {
                                callback(Err("Failed to fetch stargazers".to_string()));
                            };
                        },
                    );
                }
            }),
        }
    }

    pub fn stargazers_ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState) {
        demo_area(ui, STARGAZERS_EXAMPLE.name, 300.0, |ui| {
            ScrollArea::vertical()
                .max_height(250.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("Like");
                        ui.hyperlink_to(
                            " egui_dnd on GitHub ",
                            "https://github.com/lucasmerlin/hello_egui/tree/main/crates/egui_dnd",
                        );
                        ui.label("to be listed here!");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("On mobile you can drag to scroll and hold + drag to sort items.");
                    });
                    self.stargazers_dnd_ui(ui);
                });

            crate_usage_ui(ui, STARGAZERS_EXAMPLE.crates, shared_state);
        });
    }

    pub fn stargazers_dnd_ui(&mut self, ui: &mut Ui) {
        let response = dnd(ui, "stargazers_dnd")
            .with_touch_config(Some(DragDropConfig::touch_scroll()))
            .show_custom(|ui, iter| {
                self.infinite_scroll.ui(ui, 10, |ui, index, item| {
                    iter.next(ui, Id::new(&*item.login), index, true, |ui, item_handle| {
                        item_handle.ui(ui, |ui, handle, _state| {
                            ui.horizontal(|ui| {
                                handle.ui(ui, |ui| {
                                    Frame::none()
                                        .fill(ui.visuals().faint_bg_color)
                                        .inner_margin(8.0)
                                        .outer_margin(2.0)
                                        .rounding(4.0)
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());

                                            let size = Vec2::new(32.0, 32.0);

                                            ui.add(
                                                Image::new(format!(
                                                    "{}&s={}",
                                                    item.avatar_url,
                                                    size.x as u32 * 2
                                                ))
                                                .fit_to_exact_size(size),
                                            );

                                            ui.hyperlink_to(
                                                item.login.as_str(),
                                                item.html_url.as_str(),
                                            );
                                        });
                                });
                            });
                        })
                    });
                });
            });
        response.update_vec(&mut self.infinite_scroll.items);
    }
}
