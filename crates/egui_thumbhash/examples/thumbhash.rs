use eframe::epaint::Vec2;
use eframe::NativeOptions;
use egui::{vec2, CentralPanel, Frame, Image, Rect, ScrollArea, Sense, Ui};
use serde::Deserialize;

use egui_thumbhash::ThumbhashImage;

fn col(ui: &mut Ui, items: &[GalleryItem]) {
    let width = ui.available_width();
    ui.set_width(width);

    for item in items {
        // If you don't know the image size beforehand, you can use
        // `thumbhash::thumb_hash_to_approximate_aspect_ratio` to get an estimate
        // of the aspect ratio
        let image_size = vec2(item.width, item.height);
        // Calculate the target image size based on the column width
        let image_size = image_size * (width / image_size.x);

        // Only render images if they are visible (this is basically lazy loading)
        if ui.is_rect_visible(Rect::from_min_size(ui.next_widget_position(), image_size)) {
            // Since the egui image doesn't know it's size until it's loaded, and seems to ignore
            // fit_to_exact_size while loading, it's easiest to use add_sized to set the image to a fixed size.
            // If this is not done, the thumbhash size might not fit the final image size.
            ui.add_sized(
                image_size,
                ThumbhashImage::new(
                    Image::new(format!(
                        "https://raw.githubusercontent.com/lucasmerlin/hello_egui/main/fancy-example/src/gallery/{}.webp",
                        item.id
                    )).show_loading_spinner(false),
                    item.thumbhash.as_ref().unwrap(),
                ).rounding(8.0),
      );
        } else {
            ui.allocate_exact_size(image_size, Sense::hover());
        }
    }
}

fn main() -> eframe::Result<()> {
    let items = include_str!("../../../fancy-example/src/gallery/index.json");
    let items = serde_json::from_str::<Vec<GalleryItem>>(items).unwrap();

    eframe::run_simple_native(
        "Thumbhash Demo",
        NativeOptions::default(),
        move |ctx, _frame| {
            egui_extras::install_image_loaders(ctx);
            egui_thumbhash::register(ctx);

            CentralPanel::default()
                .frame(Frame::central_panel(&ctx.style()).inner_margin(16.0))
                .show(ctx, |ui| {
                    if ui.button("Reload").clicked() {
                        ui.ctx().forget_all_images();
                        ui.ctx().clear_animations();
                    }

                    ui.spacing_mut().item_spacing = Vec2::splat(16.0);

                    ScrollArea::vertical().show(ui, |ui| {
                        let cols = (ui.available_width() / 300.0).ceil() as usize;
                        let skip = items.len() / cols;
                        ui.horizontal(|ui| {
                            let width = ui.available_width() + ui.spacing().item_spacing.x;
                            for i in 0..cols {
                                let items = &items[i * skip..(i + 1) * skip];

                                ui.vertical(|ui| {
                                    ui.set_max_width(
                                        (width) / cols as f32 - ui.spacing().item_spacing.x,
                                    );
                                    col(ui, items);
                                });
                            }
                        });
                    });
                });
        },
    )
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GalleryItem {
    id: i32,
    title: String,
    #[serde(rename = "imageUrl")]
    image_url: String,
    thumbhash: Option<Vec<u8>>,
    width: f32,
    height: f32,
}
