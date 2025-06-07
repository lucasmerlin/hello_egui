use egui::{Id, Image, OpenUrl, ScrollArea, Sense, Ui, Vec2};
use serde::Deserialize;

use egui_infinite_scroll::InfiniteScroll;
use egui_pull_to_refresh::PullToRefresh;
use egui_thumbhash::ThumbhashImage;

use crate::crate_ui::{crate_usage_ui, Crate, CrateUsage};
use crate::demo_area;
use crate::example::{Example, ExampleTrait};
use crate::shared_state::SharedState;

pub const GALLERY_EXAMPLE: Example = Example {
    name: "Gallery",
    slug: "gallery",
    crates: &[
        CrateUsage::simple(Crate::EguiThumbhash),
        CrateUsage::simple(Crate::EguiInfiniteScroll),
        CrateUsage::simple(Crate::EguiPullToRefresh),
    ],
    get: || Box::new(Gallery::new()),
};

#[expect(dead_code, reason = "This is used for deserializing the JSON data")]
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

impl Default for Gallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Gallery {
    #[must_use]
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

impl ExampleTrait for Gallery {
    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState) {
        demo_area(ui, "Gallery", 1000.0, |ui| {
            let height = 300.0;

            let refresh_response = PullToRefresh::new(false).scroll_area_ui(ui, |ui| {
                ScrollArea::vertical()
                    .max_height(ui.available_height() * 0.9 - 32.0)
                    .auto_shrink([false, false])
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
                            let mut combined_width = item.first().map_or(0.0, |item| item.width);

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
                                    // TODO: Reintroduce cache busting once fixed: https://github.com/emilk/egui/issues/5341
                                    // let image_url = format!(
                                    //     "https://raw.githubusercontent.com/lucasmerlin/hello_egui/main/fancy-example/src/gallery/{}.webp#{}",
                                    //     item.id,
                                    //     start_idx + idx
                                    // );
                                    let image_url =
                                        if cfg!(feature = "mock") {
                                            let path = env!("CARGO_MANIFEST_DIR");
                                            format!("file://{path}/src/gallery/{}.webp", item.id)
                                        } else {
                                            format!(
                                                "https://raw.githubusercontent.com/lucasmerlin/hello_egui\
                                        /main/fancy-example/src/gallery/{}.webp#{}",
                                                item.id,
                                                start_idx + idx
                                            )
                                        };
                                    let image = Image::new(image_url).sense(Sense::click());
                                    let image = ThumbhashImage::new(image, &item.thumbhash)
                                        .id(Id::new("gallery_item").with(start_idx + idx));
                                    let response = ui.add_sized(size, image.rounding(8.0));

                                    // Workaround for buttons blocking touch scroll: https://github.com/emilk/egui/pull/3815
                                    if response.clicked() {
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
            });

            if refresh_response.should_refresh() {
                self.items.reset();
                ui.ctx().forget_all_images();
                ui.ctx().clear_animations();
            }

            ui.add_space(8.0);

            crate_usage_ui(ui, GALLERY_EXAMPLE.crates, shared_state);
        });
    }
}
