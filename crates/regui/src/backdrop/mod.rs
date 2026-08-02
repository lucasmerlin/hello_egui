//! Blur what egui has already drawn, so a panel can sit on a blurred background.

mod resources;

use crate::wgpu_state;
use egui::{
    Color32, Context, CornerRadius, Id, InnerResponse, LayerId, Margin, Order, Rect, Shape, Ui,
    Vec2, Visuals, layers::ShapeIdx,
};
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

    /// `None` means the window fill of whatever theme is in use, faded down.
    tint: Option<Color32>,
    margin: Margin,
    corner_radius: CornerRadius,

    /// How far the edge of the glass fades out, in points. Zero for a hard edge.
    feather: f32,

    /// How far the rim bends what it shows, in points. Zero for no refraction.
    refraction: f32,

    /// How bright the lit rim is, from 0 to 1. Zero for no highlight.
    specular: f32,

    /// How thick the pane is, in points. Sets how wide the rim band is.
    thickness: f32,

    /// Which way the light lies, in screen space. y grows downward.
    light_direction: Vec2,

    /// Superellipse power. Zero uses the rounded rectangle and its corner radii instead.
    squircle: f32,

    /// How hard the rim squeezes what is behind the pane, from 0 to 1. Zero for none.
    lens: f32,

    /// How bright the sheen running round the rim is. Zero for none.
    sheen: f32,

    /// How much grain is laid over the glass, from 0 to 1. Zero for none.
    grain: f32,

    /// How far the colours split where the lens bends. Zero for none.
    dispersion: f32,
}

/// How opaque the default tint is.
///
/// Enough of the theme's own colour to keep text readable, but still clearly a blur of
/// what is behind rather than a solid panel.
const DEFAULT_TINT_OPACITY: f32 = 0.6;

/// How thick the pane is by default, in points.
///
/// Wide enough to see the rim bend and catch the light, narrow enough to leave most of a
/// small panel flat.
const DEFAULT_THICKNESS: f32 = 10.0;

/// Where the light lies by default: above and to the left, as in most icon sets.
const DEFAULT_LIGHT_DIRECTION: Vec2 = Vec2::new(-0.6, -0.8);

impl BackdropBlur {
    /// Blur the background with the given radius, in points.
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            tint: None,
            margin: Margin::same(8),
            corner_radius: CornerRadius::ZERO,
            feather: 0.0,
            refraction: 0.0,
            specular: 0.0,
            thickness: DEFAULT_THICKNESS,
            light_direction: DEFAULT_LIGHT_DIRECTION,
            squircle: 0.0,
            lens: 0.0,
            sheen: 0.0,
            grain: 0.0,
            dispersion: 0.0,
        }
    }

    /// A colour laid over the blurred background.
    ///
    /// Its alpha says how far to fade the blur towards it, so
    /// `Color32::from_white_alpha(64)` gives light frosted glass and
    /// `Color32::from_black_alpha(64)` a dark one.
    ///
    /// By default this is the theme's [`egui::Visuals::window_fill`] at 60% opacity, so the
    /// glass follows the theme and whatever you put on it stays readable. Pass
    /// [`Color32::TRANSPARENT`] to leave the blur alone.
    #[inline]
    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = Some(tint);
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

    /// Fade the edge of the glass out over this many points, instead of stopping at it.
    ///
    /// By default the glass ends at its rect, one pixel of anti-aliasing aside, so its
    /// outline stays crisp however strong the blur is. A feather spreads that edge, half of
    /// it outside the rect and half inside, which is what you want while animating a panel
    /// in or out: fade the feather down to zero as the panel arrives and the glass gathers
    /// itself into a sharp shape.
    ///
    /// The blurred region grows to make room for the fade, so a feathered blur touches
    /// pixels outside its rect.
    #[inline]
    pub fn feather(mut self, feather: f32) -> Self {
        self.feather = feather;
        self
    }

    /// Bend what the rim shows, by up to this many points.
    ///
    /// A real pane of glass is thick, so its ground edge shows you what lies beside it
    /// rather than what lies under it. This walks the sample outwards along the edge,
    /// hardest at the rim and not at all in the middle, which is what makes the glass look
    /// like an object instead of a blurred rectangle.
    ///
    /// Off by default. Two or three points reads as a thin sheet; ten bends the rim hard.
    /// [`Self::thickness`] sets how wide the bent band is.
    #[inline]
    pub fn refraction(mut self, strength: f32) -> Self {
        self.refraction = strength;
        self
    }

    /// Light the rim from one side, with this strength from 0 to 1.
    ///
    /// The part of the rim that turns towards the light gets brighter, the way a bevel
    /// catches a window. The far side stays dark, so the glass reads as a solid thing lit
    /// from somewhere.
    ///
    /// Off by default. Use [`Self::light_direction`] to move the light, and
    /// [`Self::thickness`] to widen the band it lights.
    #[inline]
    pub fn specular(mut self, strength: f32) -> Self {
        self.specular = strength;
        self
    }

    /// How thick the pane is, in points.
    ///
    /// Both [`Self::refraction`] and [`Self::specular`] live in a band this wide around the
    /// edge; the rest of the pane stays flat. Ten points by default.
    #[inline]
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Which way the light lies, for [`Self::specular`].
    ///
    /// This points at the light, in screen space, so y grows downward. It is normalised for
    /// you. By default the light is above and to the left.
    #[inline]
    pub fn light_direction(mut self, direction: Vec2) -> Self {
        self.light_direction = direction;
        self
    }

    /// Shape the pane as a superellipse of this power, instead of a rounded rectangle.
    ///
    /// `|x|ⁿ + |y|ⁿ = 1`. At 2 that is an ellipse, and it squares up as the power grows.
    /// Around 4 it is the rounded square Apple uses, where the straight sides run into the
    /// corners with nothing to see at the join; a rounded rectangle meets its corner arcs at
    /// a point where the curvature jumps, and the eye picks that up.
    ///
    /// The shape fills the whole rect, so this replaces [`Self::corner_radius`] rather than
    /// working with it.
    #[inline]
    pub fn squircle(mut self, power: f32) -> Self {
        self.squircle = power;
        self
    }

    /// Squeeze what is behind the rim, with this strength from 0 to 1.
    ///
    /// A thick lens does not bend its edge a little; it gathers a wide band of the
    /// background into a thin ring, so the ring shows a squeezed copy of everything around
    /// the pane. That is what makes Apple's glass look like a solid object being held over
    /// the screen rather than a blurred hole in it.
    ///
    /// This and [`Self::refraction`] both move where a pixel looks, and they add up. Use one
    /// or the other: refraction is a thin ground edge, this is a whole lens.
    ///
    /// Off by default.
    #[inline]
    pub fn lens(mut self, strength: f32) -> Self {
        self.lens = strength;
        self
    }

    /// Run a sheen round the rim, with this strength.
    ///
    /// Bright down one side of the pane and dark down the other, following the angle round
    /// the edge. [`Self::specular`] is a point of light on the bevel; this is the whole rim
    /// picking up the room, and it is what stops a wide edge reading as a flat grey band.
    ///
    /// Off by default. Around 0.3 is enough to see.
    #[inline]
    pub fn sheen(mut self, strength: f32) -> Self {
        self.sheen = strength;
        self
    }

    /// Lay grain over the glass, from 0 to 1.
    ///
    /// A blurred image is very smooth, and large areas of it band. A little grain hides
    /// that, and reads as the surface of the glass rather than as noise.
    ///
    /// Off by default. Around 0.03 is enough.
    #[inline]
    pub fn grain(mut self, amount: f32) -> Self {
        self.grain = amount;
        self
    }

    /// Split the colours where [`Self::lens`] bends hardest.
    ///
    /// Glass does not bend red and blue by the same amount, so a hard bend fringes. This
    /// takes the red and the blue from a little further along the same path, as a fraction
    /// of the pane's own size, and only where the lens is working.
    ///
    /// Off by default. Around 0.03 is a fringe you notice without being able to name.
    #[inline]
    pub fn dispersion(mut self, amount: f32) -> Self {
        self.dispersion = amount;
        self
    }

    /// Every glass option at once, set to something that looks like Apple's Liquid Glass.
    ///
    /// A squircle, a full lens at the rim, a sheen round the edge, a lit bevel and a little
    /// grain. Set any of them again afterwards to taste, and pick the blur radius with
    /// [`Self::new`].
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// # let rect = ui.max_rect();
    /// regui::BackdropBlur::new(16.0)
    ///     .liquid_glass()
    ///     .paint_at(ui, rect);
    /// # });
    /// ```
    #[inline]
    pub fn liquid_glass(self) -> Self {
        self.squircle(4.0)
            .lens(1.0)
            .sheen(0.3)
            .specular(0.5)
            .thickness(14.0)
            .grain(0.03)
            .dispersion(0.03)
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

    /// Blur behind a window or area, underneath everything it draws.
    ///
    /// A window reserves the slot for its frame before its body runs, so nothing added
    /// from inside the body can get below it: a blur put there would cover the frame's
    /// fill. This claims the first slot in the window's layer instead, before the window
    /// has drawn anything, so its fill, stroke, shadow and rounded corners all land on top
    /// of the blur.
    ///
    /// Call this _before_ showing the window, then hand the returned [`PendingBlur`] the
    /// window's rect. [`Self::show_window`] does both for you.
    ///
    /// A window's layer is `LayerId::new(Order::Middle, id)`, where `id` is what you passed
    /// to [`egui::Window::id`], or `Id::new(title)` if you did not pass one.
    #[must_use = "Give the returned PendingBlur a rect, or nothing is drawn"]
    pub fn behind_layer(self, ctx: &Context, layer_id: LayerId) -> PendingBlur {
        // Claiming a slot in a layer the window has not created yet is fine: egui keys
        // paint lists by layer id and drains them in area order at the end of the pass, so
        // the window's own shapes simply land after this one.
        let index = ctx.layer_painter(layer_id).add(Shape::Noop);
        PendingBlur {
            blur: self,
            layer_id,
            index,
        }
    }

    /// Show a window whose background is blurred, with its frame drawn on top.
    ///
    /// The window is given `id`, so that the blur can find its layer. Note that this
    /// assumes the window is at [`egui::Order::Middle`], which is the default.
    ///
    /// Use a frame with a see-through fill, or the fill will hide the blur. The blur brings
    /// its own [`Self::tint`], so `Color32::TRANSPARENT` is usually what you want:
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// use egui::{Color32, Frame, Id, Window};
    /// use regui::BackdropBlur;
    ///
    /// let frame = Frame::window(ui.style()).fill(Color32::TRANSPARENT);
    /// BackdropBlur::new(12.0)
    ///     .corner_radius(frame.corner_radius)
    ///     .show_window(
    ///         ui,
    ///         Id::new("frosted"),
    ///         Window::new("frosted").frame(frame),
    ///         |ui| {
    ///             ui.label("on glass");
    ///         },
    ///     );
    /// # });
    /// ```
    pub fn show_window<R>(
        self,
        ui: &mut Ui,
        id: Id,
        window: egui::Window<'_>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<Option<R>>> {
        let pending = self.behind_layer(ui.ctx(), LayerId::new(Order::Middle, id));
        let response = window.id(id).show(ui, add_contents);
        match &response {
            Some(response) => pending.set_rect(ui.ctx(), response.response.rect),
            // The window is closed or collapsed, so there is nothing to blur behind.
            None => pending.discard(),
        }
        response
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

    fn put_at(self, ui: &Ui, rect: Rect, index: ShapeIdx) {
        // Each blur needs an id of its own to keep its shader uniforms apart from every
        // other blur in the pass. Two blurs in the same `Ui` would share one, and the
        // second would win; give them their own `Ui` if you need both.
        let id = ui.id().with("regui_backdrop_blur");
        if let Some(shape) = self.shape(ui.ctx(), ui.visuals(), id, rect) {
            ui.painter().set(index, shape);
        }
    }

    /// The paint callback that does the work, or `None` if there is nothing to draw.
    fn shape(self, ctx: &Context, visuals: &Visuals, id: Id, rect: Rect) -> Option<Shape> {
        if self.radius <= 0.0 {
            // Nothing to do, and no reason to make egui interrupt its render pass.
            return None;
        }
        let Some(render_state) = wgpu_state::render_state(ctx) else {
            wgpu_state::warn_not_installed(ctx, "BackdropBlur");
            return None;
        };

        // The shader works in physical pixels, since that is what it samples.
        let pixels_per_point = ctx.pixels_per_point();
        let scale = |radius: u8| f32::from(radius) * pixels_per_point;

        // The fade straddles the edge, so half of it falls outside the rect. Give the
        // callback that room, or the outer half would be clipped away and the edge would
        // look cut off rather than soft. The extra point covers the pixel of anti-aliasing
        // the mask always has.
        //
        // The rim needs no room of its own. Refraction moves what a pixel samples, not
        // which pixels are drawn, and the coverage mask is applied after the highlight, so
        // neither can reach past the fade.
        let drawn_rect = rect.expand(self.feather / 2.0 + 1.0);

        // Refraction walks the sample outwards along the rim, and dispersion pushes the red
        // and blue a little further along the same path, so the final pass can read outside
        // the pixels it draws. Nothing else does: the lens only pulls inwards.
        let sample_margin = self.refraction + self.dispersion * rect.size().length() / 2.0;

        // The blur is separable, so the horizontal pass reaches `taps` pixels to each side
        // and the vertical one `taps` up and down. Everything the callback samples out of
        // the backdrop is therefore within this of its own rect, which is what lets
        // egui-wgpu copy a panel-sized piece of the frame instead of all of it.
        let taps = (self.radius * pixels_per_point).ceil().clamp(1.0, 128.0) / pixels_per_point;
        let backdrop_margin = sample_margin + 2.0 * taps;

        let callback = egui_wgpu::Callback::new_paint_callback(
            drawn_rect,
            BlurCallback {
                format: render_state.target_format,
                id,
                backdrop_margin,
                settings: Settings {
                    radius: self.radius * pixels_per_point,
                    drawn_rect_in_pixels: Rect::from_min_max(
                        (drawn_rect.min.to_vec2() * pixels_per_point).to_pos2(),
                        (drawn_rect.max.to_vec2() * pixels_per_point).to_pos2(),
                    ),
                    sample_margin: sample_margin * pixels_per_point,
                    tint: self.tint.unwrap_or_else(|| {
                        // `gamma_multiply` scales the alpha along with the rest, which is
                        // what fading a premultiplied colour means.
                        visuals.window_fill.gamma_multiply(DEFAULT_TINT_OPACITY)
                    }),
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
                    feather: self.feather * pixels_per_point,
                    refraction: self.refraction * pixels_per_point,
                    specular: self.specular,
                    thickness: self.thickness * pixels_per_point,
                    light: self.light_direction.normalized().into(),
                    // These four are ratios and powers, so they are the same at any scale.
                    squircle: self.squircle,
                    lens: self.lens,
                    sheen: self.sheen,
                    grain: self.grain,
                    dispersion: self.dispersion,
                },
            },
        );
        Some(callback.into())
    }
}

/// A blur that has claimed its place in the paint order but does not know where it goes yet.
///
/// See [`BackdropBlur::behind_layer`].
#[must_use = "Give this a rect, or nothing is drawn"]
pub struct PendingBlur {
    blur: BackdropBlur,
    layer_id: LayerId,
    index: ShapeIdx,
}

impl PendingBlur {
    /// Blur behind this rect. Usually a window's `response.rect`.
    pub fn set_rect(self, ctx: &Context, rect: Rect) {
        let style = ctx.global_style();
        // The layer is the window's own, so its id is unique per blurred window.
        let id = self.layer_id.id.with("regui_backdrop_blur");
        if let Some(shape) = self.blur.shape(ctx, &style.visuals, id, rect) {
            ctx.layer_painter(self.layer_id).set(self.index, shape);
        }
    }

    /// Draw nothing after all, e.g. because the window turned out to be closed.
    pub fn discard(self) {
        // The slot we claimed still holds the `Shape::Noop` we put there, so there is
        // genuinely nothing to undo.
    }
}

/// The part that runs on the render thread.
struct BlurCallback {
    format: wgpu::TextureFormat,

    /// Which blurred widget this is, so it gets its own uniforms rather than sharing them
    /// with every other blur in the pass.
    id: Id,

    /// How far outside its rect this blur reads the backdrop, in points.
    backdrop_margin: f32,
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
        resources.update(
            device,
            self.id,
            screen_descriptor.size_in_pixels,
            self.format,
        );
        Vec::new()
    }

    fn needs_backdrop(&self) -> bool {
        self.settings.radius > 0.0
    }

    fn backdrop_rect(&self, drawn: Rect) -> Rect {
        // A Gaussian reaches the same distance on every side, so this is the one case where
        // a region says no more than a margin would.
        drawn.expand(self.backdrop_margin)
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
        resources.blur(device, queue, encoder, backdrop, self.id, self.settings);
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
        resources.draw(render_pass, self.id);
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
