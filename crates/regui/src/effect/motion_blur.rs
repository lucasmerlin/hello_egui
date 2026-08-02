//! A directional blur over a child ui's rendered image.

use super::{Effect, EffectContext, util};
use egui::{Id, Vec2};
use egui_wgpu::wgpu;
use std::collections::HashMap;

/// `Params` in `motion_blur.wgsl`: `step`, `origin`, `samples`.
const PARAMS: usize = 5;

/// How many samples the shader takes at most.
///
/// One sample a pixel is what it takes to smear without gaps, but a fast panel can move
/// hundreds of pixels in a frame and that many taps costs too much. Past this the samples
/// are spread over the whole trail instead, which leaves visible copies of the child rather
/// than a smooth streak. Raise it with [`MotionBlur::samples`] if a long smear bands.
const MAX_SAMPLES: u32 = 32;

/// Which way the smear falls around the pixel.
///
/// Not public: it is reached through [`MotionBlur::trailing`] and
/// [`MotionBlur::symmetric`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Shutter {
    /// Smear behind the child only, the way a shutter does.
    #[default]
    Trailing,

    /// Smear both ways, centred on the pixel.
    Symmetric,
}

/// Smear the child along a vector.
///
/// [`crate::effect::Blur`] spreads a pixel over a disc; this one drags it along a line, the
/// way a camera does when the subject moves while the shutter is open. Drive it from how
/// far the child moved since the last frame and a panel that slides in sells the movement
/// far better than a fade.
///
/// One pass, however the velocity points: a directional blur does not separate into two
/// axes the way a Gaussian does.
pub struct MotionBlur {
    velocity: Vec2,
    shutter: Shutter,
    samples: u32,
}

impl MotionBlur {
    /// Smear along `velocity`, in points.
    ///
    /// The direction is the direction of the smear and the length is how far it reaches. A
    /// zero velocity does nothing.
    ///
    /// The usual source is the difference between where the child was last frame and where
    /// it is now.
    pub fn new(velocity: Vec2) -> Self {
        Self {
            velocity,
            shutter: Shutter::default(),
            samples: MAX_SAMPLES,
        }
    }

    /// Smear behind the child only. This is the default.
    ///
    /// A camera collects light while the subject moves, so the streak covers everywhere the
    /// subject has just been and stops where it is now. The child appears to lag behind its
    /// rect, which is what reads as movement.
    pub fn trailing(mut self) -> Self {
        self.shutter = Shutter::Trailing;
        self
    }

    /// Smear both ways, centred on the child.
    ///
    /// The child stays where it is and grows a streak on each side. Use it when the smear
    /// is decoration rather than motion, or when the velocity flips sign every frame.
    pub fn symmetric(mut self) -> Self {
        self.shutter = Shutter::Symmetric;
        self
    }

    /// The most samples one pixel may take.
    ///
    /// Fewer is faster and bands sooner. The shader never takes more than it needs to give
    /// one sample a pixel, so a short smear costs less than this whatever is asked for.
    pub fn samples(mut self, samples: u32) -> Self {
        self.samples = samples.clamp(1, 256);
        self
    }

    /// The two ends of the smear, as a fraction of the velocity.
    fn span(&self) -> (f32, f32) {
        match self.shutter {
            // Gather from ahead of the pixel, so the child is dragged backwards and leaves a
            // trail where it came from.
            Shutter::Trailing => (0.0, 1.0),
            Shutter::Symmetric => (-0.5, 0.5),
        }
    }
}

impl Effect for MotionBlur {
    fn padding(&self) -> Vec2 {
        // Room is added on both sides of each axis, so a trailing smear asks for twice what
        // it uses. Only the axis that moves pays for it, which is the whole point of a
        // `Vec2` here: a sideways smear needs no room above or below.
        let reach = match self.shutter {
            Shutter::Trailing => 1.0,
            Shutter::Symmetric => 0.5,
        };
        self.velocity.abs() * reach
    }

    fn run(
        &self,
        ctx: &mut EffectContext<'_>,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let velocity = self.velocity * ctx.pixels_per_point();
        let [width, height] = ctx.size().map(|size| size as f32);

        // One sample a pixel of trail, capped. With a zero velocity this is a single tap at
        // the pixel itself, so the pass copies the child through untouched.
        let (start, end) = self.span();
        let length = velocity.length() * (end - start);
        let taps = (length.ceil() as u32 + 1).clamp(1, self.samples);

        // Spread the taps over the whole trail, ends included. One tap sits at the start of
        // the span, which for a symmetric smear is behind the pixel; that is why a symmetric
        // smear of one tap is only exact when the velocity is zero.
        let step = if taps > 1 {
            velocity * (end - start) / (taps - 1) as f32
        } else {
            Vec2::ZERO
        };
        let origin = velocity * start;

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
                step.x / width,
                step.y / height,
                origin.x / width,
                origin.y / height,
                taps as f32,
            ],
        );

        // Built every frame: the child gets new textures whenever it resizes, and rebuilding
        // a bind group is cheaper than working out whether it went stale.
        let bind_group = util::bind_group(
            device,
            "regui_motion_blur",
            &cache.layout,
            &[input],
            &cache.sampler,
            &params,
        );

        let pipeline = cache.pipeline.clone();
        ctx.pass(output, &pipeline, &bind_group);
    }
}

/// What the motion blur keeps between frames.
struct Cache {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    /// One uniform buffer per child, since two children can move different ways in the same
    /// frame.
    params: HashMap<Id, wgpu::Buffer>,
}

impl Cache {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("motion_blur.wgsl"));
        let layout = util::bind_group_layout(device, "regui_motion_blur", 1);
        Self {
            format,
            pipeline: util::pipeline(
                device,
                "regui_motion_blur",
                &shader,
                "fs_main",
                &layout,
                format,
            ),
            layout,
            sampler: util::sampler(device, "regui_motion_blur"),
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
            .or_insert_with(|| util::uniform_buffer(device, "regui_motion_blur", PARAMS))
            .clone()
    }
}
