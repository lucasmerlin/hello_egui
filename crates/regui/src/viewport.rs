use crate::{
    Transform, backend,
    input::{self, Gate},
    output,
};
use egui::{AsId, FullOutput, Id, Pos2, Rect, Response, Sense, Ui, Vec2, ViewportId, emath::Rot2};

/// Run a part of your ui in its own egui viewport, and paint the result into this ui.
///
/// The child ui shares this [`egui::Context`], so it shares memory, style and fonts, but
/// it gets its own input, its own hit-testing and its own focus. Because it is painted
/// rather than laid out, you can move, scale and rotate it.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use regui::Regui;
///
/// let output = Regui::new("preview")
///     .size(egui::vec2(200.0, 100.0))
///     .scale(0.5)
///     .rotation(0.1)
///     .show(ui, |ui| ui.button("click me").clicked());
///
/// if output.inner {
///     println!("the button inside the child ui was clicked");
/// }
/// # });
/// ```
///
/// # Caveats
///
/// All of the child's shapes go into a single layer of the parent, so the child's popups
/// and tooltips cannot leave the child's rect.
#[must_use = "You should call .show()"]
pub struct Regui {
    id_salt: Id,
    size: Vec2,
    scale: f32,
    rotation: f32,
    offset: Vec2,
    crisp: bool,
    interactive: bool,

    /// Shaders to run over the child's image, in the order they were added.
    #[cfg(feature = "wgpu")]
    effects: Vec<Box<dyn crate::Effect>>,

    /// Render through a texture even without an effect asking for it.
    offscreen: bool,
}

/// What [`Regui::show`] gives you back.
pub struct ReguiOutput<R> {
    /// The parent's response for the area the child was painted into.
    ///
    /// Its rect is the child's rect after the transform, so it grows when you rotate the
    /// child.
    pub response: Response,

    /// Whatever your ui function returned.
    pub inner: R,

    /// Maps the child's coordinates to the parent's.
    pub transform: Transform,

    /// The viewport the child ran in.
    ///
    /// Useful with [`egui::Context::input_for`] and friends, to ask questions about the
    /// child rather than the parent.
    pub viewport_id: ViewportId,
}

/// What we need to remember between passes.
#[derive(Clone, Copy, Default)]
struct State {
    /// Did anything inside the child have keyboard focus at the end of the last pass?
    ///
    /// Focus is decided during a pass, so we can only know one pass late. That is a pass
    /// of latency on "may the child see key presses", which is invisible in practice: you
    /// have to click or tab into a widget before you can type into it.
    child_has_focus: bool,

    /// Was the child receiving pointer events last pass?
    ///
    /// Used to tell the child the pointer has left, which it cannot work out on its own:
    /// it just stops hearing about a pointer that, as far as it knows, is still there.
    had_pointer: bool,
}

impl Regui {
    /// Start building a child ui.
    ///
    /// `id_salt` only has to be unique within the parent ui, like the salt of any other
    /// egui widget.
    pub fn new(id_salt: impl AsId) -> Self {
        Self {
            id_salt: Id::new(id_salt),
            size: Vec2::splat(200.0),
            scale: 1.0,
            rotation: 0.0,
            offset: Vec2::ZERO,
            crisp: false,
            interactive: true,
            #[cfg(feature = "wgpu")]
            effects: Vec::new(),
            offscreen: false,
        }
    }

    /// How big the child ui thinks its screen is, in the child's own points.
    ///
    /// This is the child's `screen_rect`; the space taken up in the parent is this size
    /// after the transform. Defaults to 200x200.
    #[inline]
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Scale the child, around its own top left corner.
    #[inline]
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Rotate the child clockwise, in radians.
    ///
    /// The parent will reserve the space the rotated child needs, which is more than the
    /// child's own size.
    #[inline]
    pub fn rotation(mut self, radians: f32) -> Self {
        self.rotation = radians;
        self
    }

    /// Shift the child away from where it would otherwise be painted.
    ///
    /// This does not change how much space the child takes up in the parent, so it is a
    /// good way to nudge or animate a child without disturbing the layout around it.
    #[inline]
    pub fn offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Rasterize the child's text for the scale it is drawn at, instead of scaling the
    /// glyphs as geometry.
    ///
    /// Turn this on when you magnify a child and its text looks soft. It costs memory:
    /// every text size in the child gets its own entry in the font atlas.
    ///
    /// The `wgpu` backend does not need this.
    #[inline]
    pub fn crisp(mut self, crisp: bool) -> Self {
        self.crisp = crisp;
        self
    }

    /// Run a shader over the child's rendered image.
    ///
    /// Effects run in the order they were added, each reading what the one before it
    /// wrote. See [`crate::effect`] for the ones that come with regui, and
    /// [`crate::Effect`] for writing your own.
    ///
    /// Needs the `wgpu` feature and [`crate::install_wgpu`]; it turns [`Self::offscreen`]
    /// on, since a shader needs an image to work on.
    #[cfg(feature = "wgpu")]
    #[inline]
    pub fn effect(mut self, effect: impl crate::Effect + 'static) -> Self {
        self.effects.push(Box::new(effect));
        self
    }

    /// Blur the child's own content, with the given radius in points.
    ///
    /// Shorthand for [`Self::effect`] with [`crate::effect::Blur`]. A radius of zero adds
    /// nothing, so it costs nothing to animate one down to it.
    ///
    /// Unlike [`crate::BackdropBlur`], which blurs what is _behind_ a rect, this blurs the
    /// child ui itself: use it to push a panel out of focus, or to fade one in and out.
    /// The child stays interactive while blurred, which is usually not what you want, so
    /// pair it with [`Self::interactive`].
    #[cfg(feature = "wgpu")]
    #[inline]
    pub fn blur(self, radius: f32) -> Self {
        if radius > 0.0 {
            self.effect(crate::effect::Blur::new(radius))
        } else {
            self
        }
    }

    /// Render the child into a texture, rather than handing its triangles to the parent.
    ///
    /// Turn this on for exact clipping when the child is rotated, and for text that stays
    /// crisp at any scale without [`Self::crisp`]'s cost to the font atlas. It is on
    /// automatically when an effect needs it.
    ///
    /// Needs the `wgpu` feature and [`crate::install_wgpu`]. Without them this falls back
    /// to handing the parent triangles, and says so in the log.
    #[cfg(feature = "wgpu")]
    #[inline]
    pub fn offscreen(mut self, offscreen: bool) -> Self {
        self.offscreen = offscreen;
        self
    }

    /// May the user interact with the child?
    ///
    /// On by default. Turn it off for a child that should only be looked at: it then sees
    /// no input at all, and clicks fall through to whatever is behind it. Useful for
    /// previews, thumbnails and backdrops.
    #[inline]
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Run the child ui and paint it.
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> ReguiOutput<R> {
        let Self {
            id_salt,
            size,
            scale,
            rotation,
            offset,
            crisp,
            interactive,
            #[cfg(feature = "wgpu")]
            effects,
            offscreen,
        } = self;

        #[cfg(feature = "wgpu")]
        let offscreen = offscreen || !effects.is_empty();

        let id = ui.make_persistent_id(id_salt);
        let viewport_id = ViewportId::from_hash_of(id);
        let ctx = ui.ctx().clone();
        let parent_id = ctx.viewport_id();

        let (transform, response) = allocate(ui, size, scale, rotation, offset, interactive);

        if !transform.is_valid() {
            // A scale of zero or a NaN rotation would make the inverse transform, and
            // therefore every pointer position we hand the child, garbage.
            log::warn!("regui: skipping a child ui with an unusable transform: {transform:?}");
            let inner = content(ui);
            return ReguiOutput {
                response,
                inner,
                transform,
                viewport_id,
            };
        }

        let mut state: State = ctx.data_mut(|data| data.get_temp(id)).unwrap_or_default();

        let has_pointer = interactive && input::wants_pointer(&response);
        let gate = Gate {
            pointer: has_pointer,
            keyboard: interactive && state.child_has_focus,
        };
        let pointer_left = state.had_pointer && !has_pointer;
        state.had_pointer = has_pointer;

        // `native_pixels_per_point`, not the effective scale: egui multiplies it by the
        // global zoom factor for us, and doing that twice would zoom the child twice.
        let native_pixels_per_point = ctx.pixels_per_point() / ctx.zoom_factor();
        let child_pixels_per_point =
            native_pixels_per_point * if crisp { scale.abs() } else { 1.0 };

        let input = input::child_input(
            ui,
            viewport_id,
            size,
            child_pixels_per_point,
            transform.inverse(),
            gate,
            pointer_left,
        );

        let (output, (inner, child_has_focus)) =
            ctx.run_hosted_viewport(viewport_id, input, |ui| {
                let inner = content(ui);
                // Read this inside the pass: `Memory::focused` reports the focus of
                // whichever viewport is current, and in here that is the child.
                (inner, ui.memory(|memory| memory.focused().is_some()))
            });

        state.child_has_focus = child_has_focus;
        ctx.data_mut(|data| data.insert_temp(id, state));

        let FullOutput {
            platform_output,
            mut textures_delta,
            shapes,
            pixels_per_point,
            viewport_output: _,
        } = output;

        // A hosted viewport leaves this frame's texture uploads in the `Context` for the
        // viewport that hosts us to hand to its backend, so there is nothing here to pass
        // on. Anything else would mean egui had drained them behind our back.
        debug_assert!(
            textures_delta.is_empty(),
            "regui: expected egui to leave the texture uploads to the hosting viewport"
        );
        textures_delta.clear();

        output::forward_platform_output(
            &ctx,
            platform_output,
            transform,
            response.contains_pointer(),
        );
        output::forward_repaint(&ctx, viewport_id, parent_id);

        let primitives = ctx.tessellate(shapes, pixels_per_point);
        paint(
            ui,
            Painted {
                id,
                primitives,
                size,
                pixels_per_point,
                transform,
                #[cfg(feature = "wgpu")]
                effects,
                offscreen,
            },
        );

        ReguiOutput {
            response,
            inner,
            transform,
            viewport_id,
        }
    }
}

/// Everything the backends need to draw one pass of a child.
struct Painted {
    id: Id,
    primitives: Vec<egui::ClippedPrimitive>,
    size: Vec2,
    pixels_per_point: f32,
    transform: Transform,

    /// Shaders to run over the child's image, in order.
    #[cfg(feature = "wgpu")]
    effects: Vec<Box<dyn crate::Effect>>,

    /// Whether to go through a texture rather than hand the parent triangles.
    offscreen: bool,
}

/// Draw the child, off-screen if asked for and possible, and by replaying its triangles
/// otherwise.
fn paint(ui: &Ui, painted: Painted) {
    let Painted {
        primitives,
        transform,
        ..
    } = painted;

    #[cfg(feature = "wgpu")]
    let primitives = {
        let mut primitives = primitives;
        if painted.offscreen {
            if let Some(render_state) = crate::wgpu_state::render_state(ui.ctx()) {
                let request = backend::texture::Request {
                    id: painted.id,
                    primitives,
                    size: painted.size,
                    pixels_per_point: painted.pixels_per_point,
                    transform,
                    effects: painted.effects,
                };
                if let Some(shape) = backend::texture::render(ui, &render_state, request) {
                    ui.painter().add(shape);
                    return;
                }
                // It could not render after all, so there is nothing left to fall back with.
                primitives = Vec::new();
            } else {
                crate::wgpu_state::warn_not_installed(ui.ctx(), "Regui::offscreen");
                primitives = Vec::new();
            }
        }
        primitives
    };

    backend::shapes::paint(ui, primitives, transform);
}

/// Reserve room for the child in the parent and work out where it lands.
///
/// Rotating around the child's own origin moves it off the space we were given, so lay out
/// the bounding box of the rotated child and then shift the child back into it.
fn allocate(
    ui: &mut Ui,
    size: Vec2,
    scale: f32,
    rotation: f32,
    offset: Vec2,
    interactive: bool,
) -> (Transform, Response) {
    let child_rect = Rect::from_min_size(Pos2::ZERO, size);
    let unplaced = Transform {
        scale,
        rotation: Rot2::from_angle(rotation),
        translation: Vec2::ZERO,
    };
    let bounds = unplaced.bounding_rect(child_rect);
    let sense = if interactive {
        Sense::click_and_drag()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(bounds.size(), sense);
    let transform = Transform {
        translation: (rect.min - bounds.min) + offset,
        ..unplaced
    };
    (transform, response)
}
