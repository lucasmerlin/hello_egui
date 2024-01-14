use egui::{Id, Image, OpenUrl, ScrollArea, Ui, Vec2};
use serde::Deserialize;

use egui_infinite_scroll::InfiniteScroll;
use egui_pull_to_refresh::PullToRefresh;
use egui_thumbhash::ThumbhashImage;

use crate::crate_ui::{crate_usage_ui, Crate, CrateUsage};
use crate::shared_state::SharedState;
use crate::sidebar::Example;
use crate::{crate_usage, demo_area};

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct GalleryItem {
    id: i32,
    title: String,
    #[serde(rename = "imageUrl")]
    image_url: String,
    thumbhash: Vec<u8>,
    width: f32,
    height: f32,
}

pub struct Gallery {
    items: InfiniteScroll<GalleryItem, usize>,
}

impl Gallery {
    pub fn new() -> Gallery {
        let items = include_str!("gallery/index.json");
        let backend = serde_json::from_str::<Vec<GalleryItem>>(items).unwrap();
        let items = InfiniteScroll::new().end_loader(move |cursor, callback| {
            let cursor = cursor.unwrap_or(0);
            let items: Vec<_> = backend.iter().skip(cursor).take(10).cloned().collect();
            if items.is_empty() {
                println!("Resetting");
                // For the sake of the example we want the gallery to be infinite
                callback(Ok((backend[0..10].to_vec(), Some(10))));
            } else {
                callback(Ok((items, Some(cursor + 10))));
            }
        });
        Self { items }
    }
}

impl Example for Gallery {
    fn name(&self) -> &'static str {
        "Gallery"
    }

    crate_usage!(
        CrateUsage::simple(Crate::EguiThumbhash),
        CrateUsage::simple(Crate::EguiInfiniteScroll),
        CrateUsage::simple(Crate::EguiPullToRefresh),
    );

    fn ui(&mut self, ui: &mut Ui, _shared_state: &mut SharedState) {
        demo_area(ui, self.name(), 1000.0, |ui| {
            let height = 300.0;

            let refresh_response =
                PullToRefresh::new(false)
                    .scroll_area_ui(ui, |ui|
                ScrollArea::vertical()
                    .max_height(ui.available_height() * 0.9 - 32.0)
                    .auto_shrink([false ,false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::splat(16.0);
                        let item_spacing = ui.spacing_mut().item_spacing.x;

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label("This is a gallery of images from ");
                            ui.hyperlink_to("malmal.io", "https://malmal.io");
                            ui.label(" and ");
                            ui.hyperlink_to("hellopaint.io", "https://hellopaint.io");
                            ui.label(
                                ". For the sake of the example, the list repeats after 100 items. \
                                  Once I've implemented thumbhash on hellopaint I will update this \
                                  example to use the actual api.",
                            );
                        });

                        self.items.ui_custom_layout(ui, 10, |ui, start_idx, item| {
                            let total_width = ui.available_width();

                            let mut count = 1;
                            let mut combined_width = item.first().map(|item| item.width).unwrap_or(0.0);

                            while combined_width < total_width - item_spacing * (count - 1) as f32
                                && count < item.len()
                            {
                                count += 1;
                                let item = &item[count - 1];
                                let item_aspect_ratio = item.width / item.height;
                                let item_width = height * item_aspect_ratio;
                                combined_width += item_width;
                            }

                            let scale =
                                (total_width - item_spacing * (count - 1) as f32) / combined_width;

                            let height = height * scale;

                            ui.horizontal(|ui| {
                                for (idx, item) in item.iter().enumerate().take(count) {
                                    let size = Vec2::new(item.width * scale, height);
                                    let response = ui.add_sized(
                                        size,
                                        ThumbhashImage::new(
                                            Image::new(format!(
                                                "https://raw.githubusercontent.com/lucasmerlin/hello_egui/main/fancy-example/src/gallery/{}.webp#{}",
                                                item.id,
                                                start_idx + idx
                                            )),
                                            &item.thumbhash,
                                        )
                                            .id(Id::new("gallery_item").with(start_idx + idx))
                                            .rounding(8.0),
                                    );

                                    // Workaround for buttons blocking touch scroll: https://github.com/emilk/egui/pull/3815
                                    if ui.input(|i| i.pointer.any_released() && !i.pointer.is_decidedly_dragging()) && response.hovered() {
                                        ui.ctx().open_url(OpenUrl::new_tab(format!(
                                            "https://hellopaint.io/gallery/post/{}",
                                            item.id
                                        )));
                                    }
                                }
                            });

                            count
                        });
                    })
            );

            if refresh_response.should_refresh() {
                self.items.reset();
                ui.ctx().forget_all_images();
                ui.ctx().clear_animations();
            }

            ui.add_space(8.0);

            crate_usage_ui(ui, self.crates(), _shared_state);
        });
    }
}
