use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use egui::{Frame, Id, ScrollArea, Ui, Vec2};
use egui_extras::RetainedImage;
use ehttp::Request;
use serde::Deserialize;

use egui_dnd::{dnd, DragDropConfig};
use egui_infinite_scroll::InfiniteScroll;

#[derive(Default)]
pub enum ImageState {
    #[default]
    None,
    Loading,
    Data(RetainedImage),
    Error(String),
}

#[derive(Deserialize)]
pub struct Stargazer {
    pub login: String,
    pub html_url: String,
    pub avatar_url: String,
    #[serde(skip)]
    pub image: Arc<Mutex<ImageState>>,
}

impl Hash for Stargazer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.login.hash(state);
    }
}

impl Debug for Stargazer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stargazer")
            .field("login", &self.login)
            .field("html_url", &self.html_url)
            .field("avatar_url", &self.avatar_url)
            .finish()
    }
}

impl Stargazer {
    pub fn load_image(&self) {
        {
            let mut guard = self.image.lock().unwrap();
            if let ImageState::None = *guard {
                *guard = ImageState::Loading;
            } else {
                return;
            }
        }

        let image_state = self.image.clone();
        let login = self.login.clone();
        let avatar_url = self.avatar_url.clone();
        ehttp::fetch(Request::get(avatar_url), move |result| {
            if let Ok(data) = result {
                let image = RetainedImage::from_image_bytes(login, &data.bytes);

                let mut guard = image_state.lock().unwrap();
                match image {
                    Ok(image) => {
                        *guard = ImageState::Data(image);
                    }
                    Err(err) => {
                        dbg!(err);
                        *guard = ImageState::Error("Failed to load image".to_string());
                    }
                }
            }
        });
    }
}

pub struct Stargazers {
    infinite_scroll: InfiniteScroll<Stargazer, usize>,
}

impl Stargazers {
    pub fn new() -> Self {
        Self {
            infinite_scroll: InfiniteScroll::new().end_loader(|cursor, callback| {


                ehttp::fetch(
                    Request::get(format!("https://api.github.com/repos/lucasmerlin/egui_dnd/stargazers?per_page=100&page={}", cursor.unwrap_or(1))),
                    move |result| {
                        dbg!(&result);
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
        ScrollArea::vertical()
            .max_height(250.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Like");
                    ui.hyperlink_to(
                        " egui_dnd on GitHub ",
                        "https://github.com/lucasmerlin/egui_dnd",
                    );
                    ui.label("to be listed here!");
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("On mobile you can drag to scroll and hold + drag to sort items.");
                });

                self.stargazers_dnd_ui(ui);
            });
    }

    pub fn stargazers_dnd_ui(&mut self, ui: &mut Ui) {
        let response = dnd(ui, "stargazers_dnd")
            .with_touch_config(Some(DragDropConfig::touch_scroll()))
            .show_custom(|ui, iter| {
                self.infinite_scroll.ui(ui, 10, |ui, index, item| {
                    iter.next(ui, Id::new(&*item.login), index, |ui, item_handle| {
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
                                            if ui.is_rect_visible(ui.min_rect().expand2(size)) {
                                                item.load_image();
                                            }

                                            let image = item.image.lock().unwrap();
                                            match &*image {
                                                ImageState::Data(image) => {
                                                    image.show_size(ui, size);
                                                }
                                                ImageState::Loading => {
                                                    ui.allocate_ui(size, |ui| {
                                                        ui.spinner();
                                                    });
                                                }
                                                ImageState::Error(e) => {
                                                    ui.allocate_ui(size, |ui| {
                                                        ui.label(e);
                                                    });
                                                }
                                                _ => {
                                                    ui.allocate_space(size);
                                                }
                                            }

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
