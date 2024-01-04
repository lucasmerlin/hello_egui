use std::fmt::Debug;
use std::hash::Hash;

use egui::{Frame, Id, Image, ScrollArea, Ui, Vec2};
use ehttp::Request;
use serde::Deserialize;

use crate::demo_area;
use crate::sidebar::Example;
use egui_dnd::{dnd, DragDropConfig};
use egui_infinite_scroll::InfiniteScroll;

#[derive(Deserialize, Debug)]
pub struct Stargazer {
    pub login: String,
    pub html_url: String,
    pub avatar_url: String,
}

impl Hash for Stargazer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.login.hash(state);
    }
}

pub struct Stargazers {
    infinite_scroll: InfiniteScroll<Stargazer, usize>,
}

impl Example for Stargazers {
    fn name(&self) -> &'static str {
        "Stargazers"
    }

    fn ui(&mut self, ui: &mut Ui, _shared_state: &mut crate::shared_state::SharedState) {
        self.stargazers_ui(ui);
    }
}

impl Stargazers {
    pub fn new() -> Self {
        Self {
            infinite_scroll: InfiniteScroll::new().end_loader(|cursor, callback| {
                ehttp::fetch(
                    Request::get(format!("https://api.github.com/repos/lucasmerlin/hello_egui/stargazers?per_page=100&page={}", cursor.unwrap_or(1))),
                    move |result| {
                        if let Ok(data) = result {
                            if let Ok(stargazers) = serde_json::from_slice::<Vec<Stargazer>>(&data.bytes) {
                                callback(Ok((stargazers, Some(cursor.unwrap_or(1) + 1))));
                            } else {
                                callback(Err("Failed to parse stargazers".to_string()));
                            }
                        } else {
                            callback(Err("Failed to fetch stargazers".to_string()));
                        };
                    },
                );
            }),
        }
    }

    pub fn stargazers_ui(&mut self, ui: &mut Ui) {
        demo_area(ui, self.name(), 300.0, |ui| {
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

            ui.separator();

            ui.label("This is a demo for egui_dnd, a drag and drop sorting library for egui.");

            ui.hyperlink_to(
                "View on GitHub",
                "https://github.com/lucasmerlin/hello_egui/tree/main/crates/egui_dnd",
            );
            ui.hyperlink_to("View on Crates.io", "https://crates.io/crates/egui_dnd");
            ui.hyperlink_to("View on docs.rs", "https://docs.rs/egui_dnd");
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
