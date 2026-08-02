//! A drop shadow thrown by the child's own shape.

use super::{Effect, EffectContext, util};
use egui::{Color32, Id, Vec2};
use egui_wgpu::wgpu;
use std::collections::HashMap;

/// `Params` in `shadow.wgsl`: `color`, `step`, `offset`, `sigma`, `radius`, `gain`.
const PARAMS: usize = 11;

/// The default blur radius, in points.
const DEFAULT_RADIUS: f32 = 12.0;

/// How far the default shadow falls, in points. Down, as if the light were above.
const DEFAULT_OFFSET: Vec2 = Vec2::new(0.0, 6.0);

/// How dark the default shadow is.
const DEFAULT_ALPHA: u8 = 96;

/// Throw a shadow from the child ui.
///
/// [`egui::epaint::Shadow`] works from shapes, so it can only shade a rectangle. This works
/// from the child's rendered image, so it shades whatever the child is: a rounded panel, a
/// circle, a handful of widgets with gaps between them, or text.
///
/// The shadow comes from the child's alpha, blurred, coloured and moved to one side. The
/// child itself is then drawn over it, sharp.
///
/// Three passes: two for the separable blur, one to put the two together.
pub struct Shadow {
    color: Color32,
    radius: f32,
    offset: Vec2,
    spread: f32,
}

impl Default for Shadow {
    fn default() -> Self {
        Self::new()
    }
}

impl Shadow {
    /// A soft black shadow, a little below the child.
    pub fn new() -> Self {
        Self {
            color: Color32::from_black_alpha(DEFAULT_ALPHA),
            radius: DEFAULT_RADIUS,
            offset: DEFAULT_OFFSET,
            spread: 0.0,
        }
    }

    /// The shadow's colour.
    ///
    /// The alpha says how dark the shadow is. A shadow is usually black, but a colour picked
    /// from the child reads better over a bright background.
    #[must_use]
    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    /// How soft the shadow is, in points.
    ///
    /// This is where the blur has faded out, not its standard deviation.
    #[must_use]
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    /// How far the shadow falls, in points, and which way.
    ///
    /// Down and to the right is the usual choice, since a screen is lit from above.
    #[must_use]
    pub fn offset(mut self, offset: impl Into<Vec2>) -> Self {
        self.offset = offset.into();
        self
    }

    /// How much to grow the shadow before it is blurred, in points.
    ///
    /// This is what [`egui::epaint::Shadow`] calls `spread`. Grow it to lift the child
    /// further off the background; shrink it, with a negative number, to tuck the shadow in
    /// behind the child.
    #[must_use]
    pub fn spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }
}

impl Effect for Shadow {
    fn passes(&self) -> u32 {
        3
    }

    fn padding(&self) -> Vec2 {
        // The shadow reaches `radius + spread` past the child, and the offset moves all of
        // that. Padding is added to both sides, so the offset counts either way round.
        let reach = self.radius + self.spread.max(0.0);
        Vec2::splat(reach) + self.offset.abs()
    }

    fn run(
        &self,
        ctx: &mut EffectContext<'_>,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let (Some(horizontal_target), Some(vertical_target)) = (ctx.scratch(0), ctx.scratch(1))
        else {
            log::warn!("regui: the shadow effect was not given two scratch textures");
            return;
        };

        let pixels_per_point = ctx.pixels_per_point();
        let radius = self.radius * pixels_per_point;

        // A Gaussian is visually finished at three standard deviations, so treat the radius
        // asked for as the point where the shadow has faded out.
        let sigma = (radius / 3.0).max(0.1);
        let taps = radius.ceil().clamp(1.0, 128.0);
        let gain = gain(self.spread * pixels_per_point, sigma);

        let [width, height] = ctx.size().map(|size| size as f32);
        let offset = [
            self.offset.x * pixels_per_point / width,
            self.offset.y * pixels_per_point / height,
        ];

        let id = ctx.id();
        let device = ctx.device();
        let queue = ctx.queue();
        let format = ctx.format();

        // Sampling an sRGB texture hands the shader linear values and writing to one converts
        // back, so the colour has to be in whichever space the target works in. Both forms
        // are premultiplied, which is what the composite expects.
        let color = if format.is_srgb() {
            egui::Rgba::from(self.color).to_array()
        } else {
            self.color.to_normalized_gamma_f32()
        };

        let cache: &mut Cache = ctx
            .resources()
            .entry()
            .or_insert_with(|| Cache::new(device, format));
        cache.rebuild_if_needed(device, format);

        // One buffer per pass, not one written three times: `Queue::write_buffer` lands
        // before any command runs, so a shared buffer would leave every pass with the last
        // set of numbers.
        let params = cache.params(device, id);
        let write = |index: usize, step: [f32; 2], gain: f32| {
            util::write_floats(
                queue,
                &params[index],
                &[
                    color[0], color[1], color[2], color[3], step[0], step[1], offset[0], offset[1],
                    sigma, taps, gain,
                ],
            );
        };
        // The spread waits for the second pass, so it works on a blur finished in both
        // directions rather than on a half-blurred smear.
        write(0, [1.0 / width, 0.0], 1.0);
        write(1, [0.0, 1.0 / height], gain);
        write(2, [0.0, 0.0], 1.0);

        // Built every frame rather than cached: a bind group points at textures the child
        // gets new ones of whenever it resizes, and rebuilding one is far cheaper than
        // working out whether it went stale.
        let group = |label, textures: [&wgpu::TextureView; 2], index: usize| {
            util::bind_group(
                device,
                label,
                &cache.layout,
                &textures,
                &cache.sampler,
                &params[index],
            )
        };
        // The blur reads one texture, but the layout has two slots for the composite's sake,
        // so the source goes in both and the shader ignores the second.
        let horizontal = group("regui_shadow_horizontal", [input, input], 0);
        let vertical = group(
            "regui_shadow_vertical",
            [horizontal_target, horizontal_target],
            1,
        );
        let composite = group("regui_shadow_composite", [input, vertical_target], 2);

        let blur = cache.blur.clone();
        let pipeline = cache.composite.clone();
        ctx.pass(horizontal_target, &blur, &horizontal);
        ctx.pass(vertical_target, &blur, &vertical);
        ctx.pass(output, &pipeline, &composite);
    }
}

/// How much to scale the blurred alpha by to grow the shadow by `spread` pixels.
///
/// A min/max filter would be the honest way to grow a shape, but it costs another pass and
/// another set of taps. Scaling the blurred alpha instead moves its half-way line outwards,
/// which is the same thing to first order and free: the blurred edge of a straight line
/// falls at `1 / (sigma * sqrt(2 * pi))` per pixel, so scaling by `gain` moves the half-way
/// line `sigma * (1 - 1 / gain) / (2 * 0.3989)` pixels out. This solves that for `gain`.
///
/// It is only right near the edge, and the clamp keeps a wide spread from turning the whole
/// falloff into a hard line.
fn gain(spread: f32, sigma: f32) -> f32 {
    /// The height of a unit Gaussian at its centre, `1 / sqrt(2 * pi)`.
    const PEAK: f32 = 0.398_942_3;

    1.0 / (1.0 - 2.0 * PEAK * spread / sigma).clamp(0.25, 4.0)
}

/// What the shadow keeps between frames.
struct Cache {
    format: wgpu::TextureFormat,
    blur: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    /// One set of uniform buffers per child, since two children can be shaded differently in
    /// the same frame.
    params: HashMap<Id, [wgpu::Buffer; 3]>,
}

impl Cache {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shadow.wgsl"));
        let layout = util::bind_group_layout(device, "regui_shadow", 2);
        Self {
            format,
            blur: util::pipeline(
                device,
                "regui_shadow_blur",
                &shader,
                "fs_blur",
                &layout,
                format,
            ),
            composite: util::pipeline(
                device,
                "regui_shadow_composite",
                &shader,
                "fs_composite",
                &layout,
                format,
            ),
            layout,
            sampler: util::sampler(device, "regui_shadow"),
            params: HashMap::new(),
        }
    }

    /// The pipelines are tied to the target's format, which can change if the surface does.
    fn rebuild_if_needed(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.format != format {
            *self = Self::new(device, format);
        }
    }

    /// The buffers for one child. Cloned out, since a `Buffer` is only a handle and the rest
    /// of the cache is needed while they are in use.
    fn params(&mut self, device: &wgpu::Device, id: Id) -> [wgpu::Buffer; 3] {
        self.params
            .entry(id)
            .or_insert_with(|| {
                [
                    util::uniform_buffer(device, "regui_shadow_horizontal", PARAMS),
                    util::uniform_buffer(device, "regui_shadow_vertical", PARAMS),
                    util::uniform_buffer(device, "regui_shadow_composite", PARAMS),
                ]
            })
            .clone()
    }
}
