//! Blur what egui has already drawn, so a panel can sit on a blurred background.

mod resources;

use crate::wgpu_state;
use egui::{Color32, CornerRadius, InnerResponse, Margin, Rect, Shape, Ui};
use egui_wgpu::{Backdrop, CallbackResources, CallbackTrait, ScreenDescriptor, wgpu};
use resources::{BlurResources, Settings};

/// Blurs whatever is behind it.
///
/// This is the "frosted glass" behind a dialog or a sidebar: the background is not hidden,
/// it is pushed out of focus so the content in front reads clearly.
///
/// Needs the `wgpu` feature and a call to [`crate::install_wgpu`] at startup.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use regui::BackdropBlur;
///
/// BackdropBlur::new(12.0).show(ui, |ui| {
///     ui.heading("On frosted glass");
///     ui.label("The background behind this is blurred.");
/// });
/// # });
/// ```
///
/// # How it works
///
/// egui draws in order, so by the time it reaches this widget the background is already
/// drawn. Nothing can sample the texture it is currently drawing into, so egui-wgpu
/// interrupts its render pass, copies the half-drawn frame aside, blurs it, and draws the
/// result back. That means the blur always shows exactly what is behind it, with no
/// guessing and no one-frame lag, but it does cost a pass split and a full-screen copy, so
/// use a handful of these rather than dozens.
#[must_use = "You should call .show() or .paint_at()"]
#[derive(Clone, Copy, Debug)]
pub struct BackdropBlur {
    radius: f32,
    tint: Color32,
    margin: Margin,
    corner_radius: CornerRadius,
}

impl BackdropBlur {
    /// Blur the background with the given radius, in points.
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            tint: Color32::TRANSPARENT,
            margin: Margin::same(8),
            corner_radius: CornerRadius::ZERO,
        }
    }

    /// A colour laid over the blurred background.
    ///
    /// Its alpha says how far to fade the blur towards it, so
    /// `Color32::from_white_alpha(64)` gives the usual light frosted glass and
    /// `Color32::from_black_alpha(64)` a dark one. The default,
    /// [`Color32::TRANSPARENT`], leaves the blur as it is.
    #[inline]
    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = tint;
        self
    }

    /// Round off the corners of the blurred rect, as in [`egui::Frame`].
    ///
    /// The corners fade into the unblurred background rather than cutting a hard step out
    /// of it, so this lines up with a rounded frame drawn on top.
    #[inline]
    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = corner_radius.into();
        self
    }

    /// Padding between the blurred rect and the content, as in [`egui::Frame`].
    #[inline]
    pub fn inner_margin(mut self, margin: impl Into<Margin>) -> Self {
        self.margin = margin.into();
        self
    }

    /// Blur the background behind `rect`.
    ///
    /// Whatever you draw after this lands on top of the blur, so call it first.
    pub fn paint_at(self, ui: &Ui, rect: Rect) {
        self.put_at(ui, rect, ui.painter().add(Shape::Noop));
    }

    /// Lay out `add_contents` and blur the background behind it, like a [`egui::Frame`].
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        // Reserve a slot in the paint list now and fill it in once the content has told us
        // how big it is. That puts the blur underneath the content even though we only
        // learn its rect afterwards.
        let index = ui.painter().add(Shape::Noop);

        let outer = ui.available_rect_before_wrap();
        let content_rect = outer - self.margin;
        let mut builder = egui::UiBuilder::new().max_rect(content_rect);
        builder.layout = Some(*ui.layout());
        let mut content_ui = ui.new_child(builder);

        let inner = add_contents(&mut content_ui);
        let rect = content_ui.min_rect() + self.margin;
        let response = ui.allocate_rect(rect, egui::Sense::hover());

        self.put_at(ui, rect, index);
        InnerResponse::new(inner, response)
    }

    fn put_at(self, ui: &Ui, rect: Rect, index: egui::layers::ShapeIdx) {
        if self.radius <= 0.0 {
            // Nothing to do, and no reason to make egui interrupt its render pass.
            return;
        }
        let Some(render_state) = wgpu_state::render_state(ui.ctx()) else {
            wgpu_state::warn_not_installed(ui.ctx(), "BackdropBlur");
            return;
        };

        // The shader works in physical pixels, since that is what it samples.
        let pixels_per_point = ui.ctx().pixels_per_point();
        let scale = |radius: u8| f32::from(radius) * pixels_per_point;

        ui.painter().set(
            index,
            egui_wgpu::Callback::new_paint_callback(
                rect,
                BlurCallback {
                    format: render_state.target_format,
                    settings: Settings {
                        radius: self.radius * pixels_per_point,
                        tint: self.tint,
                        rect_in_pixels: Rect::from_min_max(
                            (rect.min.to_vec2() * pixels_per_point).to_pos2(),
                            (rect.max.to_vec2() * pixels_per_point).to_pos2(),
                        ),
                        corner_radii: [
                            scale(self.corner_radius.nw),
                            scale(self.corner_radius.ne),
                            scale(self.corner_radius.sw),
                            scale(self.corner_radius.se),
                        ],
                    },
                },
            ),
        );
    }
}

/// The part that runs on the render thread.
struct BlurCallback {
    format: wgpu::TextureFormat,
    settings: Settings,
}

impl CallbackTrait for BlurCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // `paint_with_backdrop` only gets to read the resources, so everything has to be
        // allocated here.
        let resources = callback_resources
            .entry()
            .or_insert_with(|| BlurResources::new(device, self.format));
        resources.update(device, screen_descriptor.size_in_pixels, self.format);
        Vec::new()
    }

    fn needs_backdrop(&self) -> bool {
        self.settings.radius > 0.0
    }

    fn process_backdrop(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        callback_resources: &CallbackResources,
        backdrop: &Backdrop<'_>,
    ) {
        let Some(resources) = callback_resources.get::<BlurResources>() else {
            return;
        };
        resources.blur(device, queue, encoder, backdrop, self.settings);
    }

    fn paint_with_backdrop(
        &self,
        _info: egui::epaint::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
        _backdrop: &Backdrop<'_>,
    ) {
        let Some(resources) = callback_resources.get::<BlurResources>() else {
            return;
        };
        resources.draw(render_pass);
    }

    fn paint(
        &self,
        _info: egui::epaint::PaintCallbackInfo,
        _render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &CallbackResources,
    ) {
        // Reached when the integration cannot capture a backdrop. Drawing nothing leaves
        // the background as it was, which is the right way to degrade.
    }
}
