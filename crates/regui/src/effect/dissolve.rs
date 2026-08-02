//! A dissolve: hide a child ui by breaking it up.

use super::{Effect, EffectContext, util};
use egui::{Color32, Id, Vec2};
use egui_wgpu::wgpu;
use std::collections::HashMap;

/// `Params` in `dissolve.wgsl`: `burn`, `direction`, `size`, `threshold`, `softness`,
/// `cell`, `wipe`.
const PARAMS: usize = 12;

/// How the ui breaks up.
enum Pattern {
    /// Speckles, from value noise. The cell size is in points.
    Noise { size: f32 },

    /// A wipe along a unit direction.
    Wipe { direction: Vec2 },
}

/// Hide a child ui by breaking it up.
///
/// A pattern gives every pixel a number. Pixels below the current progress stay, the rest
/// go, so the ui falls apart rather than fading evenly. Value noise breaks it into
/// speckles; a direction wipes it away from one side.
///
/// Progress runs from 1, the whole ui, to 0, nothing at all. Animate it with
/// [`egui::Context::animate_bool_with_time_and_easing`]:
///
/// ```ignore
/// let progress = ui.ctx().animate_bool_with_time_and_easing(id, open, 0.4, egui::emath::easing::cubic_in_out);
/// Regui::new("panel")
///     .effect(Dissolve::new(progress).burn(Color32::from_rgb(255, 140, 40)))
///     .show(ui, |ui| ..);
/// ```
///
/// The child stays interactive while it dissolves, which is rarely what you want, so pair
/// this with [`crate::Regui::interactive`].
pub struct Dissolve {
    progress: f32,
    pattern: Pattern,
    softness: f32,
    burn: Color32,
}

impl Dissolve {
    /// Dissolve into speckles at the given progress, 1 for whole and 0 for gone.
    ///
    /// Anything outside that range is clamped.
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            pattern: Pattern::Noise { size: 8.0 },
            softness: 0.15,
            burn: Color32::TRANSPARENT,
        }
    }

    /// Break the ui into speckles roughly `size` points across.
    ///
    /// Small values give grit, large ones give patches. This is the default, at 8 points.
    pub fn noise(mut self, size: f32) -> Self {
        self.pattern = Pattern::Noise {
            size: size.max(0.1),
        };
        self
    }

    /// Wipe the ui away instead of speckling it.
    ///
    /// The pixels furthest along `direction` go first, so [`Vec2::RIGHT`] eats the ui from
    /// its right edge. A direction of zero is ignored.
    pub fn wipe(mut self, direction: Vec2) -> Self {
        if direction != Vec2::ZERO {
            self.pattern = Pattern::Wipe {
                direction: direction.normalized(),
            };
        }
        self
    }

    /// How wide the fading band around the front is, from 0 to 1.
    ///
    /// Zero clips the pixels off hard. Wider bands look softer, and at 1 the dissolve is
    /// close to a plain fade. The default is 0.15.
    pub fn softness(mut self, softness: f32) -> Self {
        self.softness = softness.clamp(0.0, 1.0);
        self
    }

    /// Tint the front towards a colour, so that the dissolve looks like it is burning.
    ///
    /// The colour's alpha is how hard the tint bites. Off by default.
    pub fn burn(mut self, colour: Color32) -> Self {
        self.burn = colour;
        self
    }
}

impl Effect for Dissolve {
    fn run(
        &self,
        ctx: &mut EffectContext<'_>,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let [width, height] = ctx.size().map(|size| size as f32);

        // A band of zero would divide by zero in the shader, and a hard edge is only ever
        // one pixel away from this one anyway.
        let softness = self.softness.max(1e-4);

        // Slide the threshold a whole band below zero and up to one, so that full progress
        // keeps every pixel and no progress keeps none, whatever the band is worth.
        let threshold = self.progress.mul_add(1.0 + softness, -softness);

        let (direction, cell, wipe) = match self.pattern {
            Pattern::Noise { size } => (
                Vec2::ZERO,
                (size * ctx.pixels_per_point()).max(1.0),
                0.0_f32,
            ),
            Pattern::Wipe { direction } => (direction, 1.0, 1.0_f32),
        };

        let burn = burn_colour(self.burn, ctx.format());

        let id = ctx.id();
        let device = ctx.device();
        let queue = ctx.queue();
        let format = ctx.format();

        let cache: &mut Cache = ctx
            .resources()
            .entry()
            .or_insert_with(|| Cache::new(device, format));
        cache.rebuild_if_needed(device, format);

        let params = cache.params(device, id);
        util::write_floats(
            queue,
            &params,
            &[
                burn[0],
                burn[1],
                burn[2],
                burn[3],
                direction.x,
                direction.y,
                width,
                height,
                threshold,
                softness,
                cell,
                wipe,
            ],
        );

        // Built every frame rather than cached: a bind group points at textures the child
        // gets new ones of whenever it resizes, and rebuilding one is far cheaper than
        // working out whether it went stale.
        let bind_group = util::bind_group(
            device,
            "regui_dissolve",
            &cache.layout,
            &[input],
            &cache.sampler,
            &params,
        );

        let pipeline = cache.pipeline.clone();
        ctx.pass(output, &pipeline, &bind_group);
    }
}

/// The burn colour as the shader wants it: straight rgb, then the strength.
///
/// The shader writes whatever the target holds. An sRGB target converts for it, so it has
/// to work in linear; a plain target does not, so it has to work in gamma.
fn burn_colour(colour: Color32, format: wgpu::TextureFormat) -> [f32; 4] {
    let [r, g, b, a] = colour.to_srgba_unmultiplied();
    let channel = |value: u8| {
        if format.is_srgb() {
            egui::ecolor::linear_f32_from_gamma_u8(value)
        } else {
            f32::from(value) / 255.0
        }
    };
    [channel(r), channel(g), channel(b), f32::from(a) / 255.0]
}

/// What the dissolve keeps between frames.
struct Cache {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    /// One uniform buffer per child, since two children can be at different points of
    /// their dissolve in the same frame.
    params: HashMap<Id, wgpu::Buffer>,
}

impl Cache {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("dissolve.wgsl"));
        let layout = util::bind_group_layout(device, "regui_dissolve", 1);
        Self {
            format,
            pipeline: util::pipeline(
                device,
                "regui_dissolve",
                &shader,
                "fs_main",
                &layout,
                format,
            ),
            layout,
            sampler: util::sampler(device, "regui_dissolve"),
            params: HashMap::new(),
        }
    }

    /// The pipeline is tied to the target's format, which can change if the surface does.
    fn rebuild_if_needed(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.format != format {
            *self = Self::new(device, format);
        }
    }

    /// The buffer for one child. Cloned out, since a `Buffer` is only a handle and the rest
    /// of the cache is needed while it is in use.
    fn params(&mut self, device: &wgpu::Device, id: Id) -> wgpu::Buffer {
        self.params
            .entry(id)
            .or_insert_with(|| util::uniform_buffer(device, "regui_dissolve", PARAMS))
            .clone()
    }
}
