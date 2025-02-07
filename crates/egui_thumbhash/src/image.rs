use egui::epaint::RectShape;
use egui::load::TexturePoll;
use egui::{
    Color32, CornerRadius, Id, Image, Pos2, Rect, Response, SizeHint, TextureOptions, Ui, Vec2,
    Widget,
};

use crate::thumbhash_to_uri;

/// A widget that displays a thumbhash while the actual image is loading.
pub struct ThumbhashImage<'a, 'h> {
    image: Image<'a>,
    thumbhash: &'h [u8],
    fade: bool,
    fit_to_exact_size: Option<egui::Vec2>,
    rounding: Option<CornerRadius>,
    id: Id,
}

impl<'a, 'h> ThumbhashImage<'a, 'h> {
    /// Create a new `ThumbhashImage` widget.
    /// You should pass a [Image] with the configuration you want.
    /// Since the width of the egui Image is currently a bit finicky, you can use
    /// [`Image::fit_to_exact_size`] to make sure the image is the size you want.
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

    /// Set a unique id for this widget, used for the fade animation.
    /// By default, the thumbhash data is used as the id.
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Set whether the image should fade in when it's loaded.
    /// Defaults to true.
    pub fn fade(mut self, fade: bool) -> Self {
        self.fade = fade;
        self
    }

    /// Set the exact size the image should be shown at.
    /// This will override the size of the image widget.
    pub fn fit_to_exact_size(mut self, size: egui::Vec2) -> Self {
        self.fit_to_exact_size = Some(size);
        self
    }

    /// Set the rounding of the image.
    /// Use this instead of [`Image::rounding`] to make sure the rounding is applied to the
    /// thumbhash image as well.
    pub fn rounding(mut self, rounding: impl Into<CornerRadius>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Show the image.
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
            self.image = self.image.corner_radius(rounding);
        }

        let response = self.image.ui(ui);

        if t > 0.0 {
            let i = (t * 255.0) as u8;

            let image = ui.ctx().try_load_texture(
                &thumbhash_to_uri(self.thumbhash),
                TextureOptions::LINEAR,
                SizeHint::default(),
            );
            if let Ok(TexturePoll::Ready { texture, .. }) = image {
                ui.painter().add(
                    RectShape::filled(
                        Rect::from_min_size(
                            response.rect.min,
                            self.fit_to_exact_size.unwrap_or(response.rect.size()),
                        ),
                        self.rounding.unwrap_or_default(),
                        Color32::from_rgba_premultiplied(i, i, i, i),
                    )
                    .with_texture(
                        texture.id,
                        Rect::from_min_size(Pos2::default(), Vec2::new(1.0, 1.0)),
                    ),
                );
            }
        }

        if let Ok(TexturePoll::Pending { .. }) = result {}

        response
    }
}

impl Widget for ThumbhashImage<'_, '_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.ui(ui)
    }
}
