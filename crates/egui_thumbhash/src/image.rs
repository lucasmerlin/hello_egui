use egui::epaint::RectShape;
use egui::load::TexturePoll;
use egui::{Color32, Id, Image, Rect, Response, Rounding, SizeHint, TextureOptions, Ui, Widget};

use crate::thumbhash_to_uri;

pub struct ThumbhashImage<'a, 'h> {
    image: Image<'a>,
    thumbhash: &'h [u8],
    fade: bool,
    fit_to_exact_size: Option<egui::Vec2>,
    rounding: Option<Rounding>,
    id: Id,
}

impl<'a, 'h> ThumbhashImage<'a, 'h> {
    pub fn new(image: Image<'a>, thumbhash: &'h [u8]) -> Self {
        Self {
            id: Id::new(thumbhash),
            image,
            thumbhash,
            fade: true,
            fit_to_exact_size: None,
            rounding: None,
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    pub fn fade(mut self, fade: bool) -> Self {
        self.fade = fade;
        self
    }

    pub fn fit_to_exact_size(mut self, size: egui::Vec2) -> Self {
        self.fit_to_exact_size = Some(size);
        self
    }

    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    pub fn ui(mut self, ui: &mut egui::Ui) -> Response {
        if let Some(size) = self.fit_to_exact_size {
            self.image = self.image.fit_to_exact_size(size);
        }

        let result = self.image.load_for_size(ui.ctx(), ui.available_size());

        let loading = matches!(result, Ok(TexturePoll::Pending { .. }));

        let t = ui
            .ctx()
            .animate_bool_with_time(self.id.with("fade"), loading, 0.4);

        // if self.fade {
        //     self.image = self
        //         .image
        //         .tint(Color32::from_white_alpha(((1.0 - t) * 255.0) as u8));
        // }

        if let Some(rounding) = self.rounding {
            self.image = self.image.rounding(rounding);
        }

        let response = self.image.ui(ui);

        if t > 0.0 {
            let i = (t * 255.0) as u8;

            let image = ui.ctx().try_load_texture(
                &thumbhash_to_uri(self.thumbhash),
                TextureOptions::LINEAR,
                SizeHint::default(),
            );
            // Image::new(image)
            //     .maintain_aspect_ratio(false)
            //     .tint(Color32::from_rgba_premultiplied(i, i, i, i))
            //     .sense(egui::Sense::hover())
            //     .paint_at(ui, response.rect);
            if let Ok(TexturePoll::Ready { texture, .. }) = image {
                ui.painter().add(RectShape {
                    rect: Rect::from_min_size(
                        response.rect.min,
                        self.fit_to_exact_size.unwrap_or(response.rect.size()),
                    ),
                    rounding: self.rounding.unwrap_or_default(),
                    fill_texture_id: texture.id,
                    fill: Color32::from_rgba_premultiplied(i, i, i, i),
                    stroke: Default::default(),
                    uv: Rect::from_min_size(Default::default(), egui::Vec2::new(1.0, 1.0)),
                });
            }
        }

        if let Ok(TexturePoll::Pending { .. }) = result {}

        response
    }
}

impl<'a, 'h> Widget for ThumbhashImage<'a, 'h> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.ui(ui)
    }
}
