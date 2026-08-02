//! A separable Gaussian blur over a child ui's rendered image.

use super::{Effect, EffectContext, util};
use egui::{Id, Vec2};
use egui_wgpu::wgpu;
use std::collections::HashMap;

/// `Params` in `blur.wgsl`: `step`, `sigma`, `radius`.
const PARAMS: usize = 4;

/// Blur the child's own content.
///
/// Unlike [`crate::BackdropBlur`], which blurs what is _behind_ a rect, this blurs the
/// child ui itself: use it to push a panel out of focus, or to fade one in and out.
///
/// The blur is separable, so it costs two passes of `2n` samples rather than one pass of
/// `n²`.
pub struct Blur {
    radius: f32,
}

impl Blur {
    /// Blur with the given radius, in points.
    ///
    /// The radius is where the blur has faded out, not its standard deviation.
    pub fn new(radius: f32) -> Self {
        Self {
            radius: radius.max(0.0),
        }
    }
}

impl Effect for Blur {
    fn passes(&self) -> u32 {
        2
    }

    fn padding(&self) -> Vec2 {
        Vec2::splat(self.radius)
    }

    fn run(
        &self,
        ctx: &mut EffectContext<'_>,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let Some(scratch) = ctx.scratch(0) else {
            log::warn!("regui: the blur effect was not given a scratch texture");
            return;
        };

        let radius = self.radius * ctx.pixels_per_point();

        // A Gaussian is visually finished at three standard deviations, so treat the radius
        // asked for as the point where the blur has faded out.
        let sigma = (radius / 3.0).max(0.1);
        let taps = radius.ceil().clamp(1.0, 128.0);

        let [width, height] = ctx.size().map(|size| size as f32);
        let id = ctx.id();
        let device = ctx.device();
        let queue = ctx.queue();
        let format = ctx.format();

        let cache: &mut Cache = ctx
            .resources()
            .entry()
            .or_insert_with(|| Cache::new(device, format));
        cache.rebuild_if_needed(device, format);

        // Two buffers rather than one written twice: `Queue::write_buffer` lands before any
        // command runs, so a single buffer would leave both passes blurring the same way.
        let params = cache.params(device, id);
        util::write_floats(queue, &params[0], &[1.0 / width, 0.0, sigma, taps]);
        util::write_floats(queue, &params[1], &[0.0, 1.0 / height, sigma, taps]);

        // Built every frame rather than cached: a bind group points at textures the child
        // gets new ones of whenever it resizes, and rebuilding one is far cheaper than
        // working out whether it went stale.
        let horizontal = util::bind_group(
            device,
            "regui_blur_horizontal",
            &cache.layout,
            &[input],
            &cache.sampler,
            &params[0],
        );
        let vertical = util::bind_group(
            device,
            "regui_blur_vertical",
            &cache.layout,
            &[scratch],
            &cache.sampler,
            &params[1],
        );

        let pipeline = cache.pipeline.clone();
        ctx.pass(scratch, &pipeline, &horizontal);
        ctx.pass(output, &pipeline, &vertical);
    }
}

/// What the blur keeps between frames.
struct Cache {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    /// One pair of uniform buffers per child, since two children can be blurred by
    /// different amounts in the same frame.
    params: HashMap<Id, [wgpu::Buffer; 2]>,
}

impl Cache {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("blur.wgsl"));
        let layout = util::bind_group_layout(device, "regui_blur", 1);
        Self {
            format,
            pipeline: util::pipeline(device, "regui_blur", &shader, "fs_main", &layout, format),
            layout,
            sampler: util::sampler(device, "regui_blur"),
            params: HashMap::new(),
        }
    }

    /// The pipeline is tied to the target's format, which can change if the surface does.
    fn rebuild_if_needed(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.format != format {
            *self = Self::new(device, format);
        }
    }

    /// The pair of buffers for one child. Cloned out, since a `Buffer` is only a handle
    /// and the rest of the cache is needed while they are in use.
    fn params(&mut self, device: &wgpu::Device, id: Id) -> [wgpu::Buffer; 2] {
        self.params
            .entry(id)
            .or_insert_with(|| {
                [
                    util::uniform_buffer(device, "regui_blur_horizontal", PARAMS),
                    util::uniform_buffer(device, "regui_blur_vertical", PARAMS),
                ]
            })
            .clone()
    }
}
