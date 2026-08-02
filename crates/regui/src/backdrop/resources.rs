//! The wgpu objects the backdrop blur needs, and the passes it runs.

use egui::{Color32, Id, Rect};
use egui_wgpu::{Backdrop, wgpu};
use std::collections::HashMap;

/// One `Params` in `blur.wgsl`.
///
/// Written by hand rather than with `bytemuck`, to keep the dependency out.
struct Params {
    step: [f32; 2],
    sigma: f32,
    radius: f32,
    target_size: [f32; 2],
    feather: f32,
    tint: [f32; 4],
    rect_min: [f32; 2],
    rect_max: [f32; 2],
    corner_radii: [f32; 4],
    refraction: f32,
    specular: f32,
    thickness: f32,
    light: [f32; 2],
    squircle: f32,
    lens: f32,
    sheen: f32,
    grain: f32,
    dispersion: f32,
}

impl Params {
    /// 36 floats: see `Params` in `blur.wgsl`, including the padding that keeps `tint` on a
    /// 16 byte boundary and the trailing padding that rounds the struct out to whole vec4s.
    const SIZE: u64 = 36 * 4;

    fn to_bytes(&self) -> [u8; Self::SIZE as usize] {
        let floats = [
            self.step[0],
            self.step[1],
            self.sigma,
            self.radius,
            self.target_size[0],
            self.target_size[1],
            self.feather,
            0.0,
            self.tint[0],
            self.tint[1],
            self.tint[2],
            self.tint[3],
            self.rect_min[0],
            self.rect_min[1],
            self.rect_max[0],
            self.rect_max[1],
            self.corner_radii[0],
            self.corner_radii[1],
            self.corner_radii[2],
            self.corner_radii[3],
            self.refraction,
            self.specular,
            self.thickness,
            0.0,
            self.light[0],
            self.light[1],
            0.0,
            0.0,
            self.squircle,
            self.lens,
            self.sheen,
            self.grain,
            self.dispersion,
            0.0,
            0.0,
            0.0,
        ];
        let mut bytes = [0_u8; Self::SIZE as usize];
        for (chunk, float) in bytes.chunks_exact_mut(4).zip(floats) {
            chunk.copy_from_slice(&float.to_le_bytes());
        }
        bytes
    }
}

/// A whole-pixel part of the target, ready for `set_scissor_rect`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Region {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Region {
    /// Every pixel `rect` touches, clamped to a target `size_in_pixels` across.
    pub(crate) fn new(rect: Rect, size_in_pixels: [u32; 2]) -> Self {
        let [width, height] = size_in_pixels.map(|size| size as f32);
        let min_x = rect.min.x.floor().clamp(0.0, width);
        let min_y = rect.min.y.floor().clamp(0.0, height);
        let max_x = rect.max.x.ceil().clamp(min_x, width);
        let max_y = rect.max.y.ceil().clamp(min_y, height);
        Self {
            x: min_x as u32,
            y: min_y as u32,
            width: (max_x - min_x) as u32,
            height: (max_y - min_y) as u32,
        }
    }

    /// `[x, y, width, height]`, as [`Backdrop::valid_in_pixels`] gives it.
    fn from_array([x, y, width, height]: [u32; 4]) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// The part of `self` that is also in `other`.
    fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        Self {
            x,
            y,
            width: right.saturating_sub(x),
            height: bottom.saturating_sub(y),
        }
    }

    fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// How the blur should look this frame.
#[derive(Clone, Copy)]
pub(crate) struct Settings {
    /// Blur radius, in physical pixels.
    pub radius: f32,

    /// Laid over the blur; its alpha fades the blur towards it.
    pub tint: Color32,

    /// The rect being blurred, in physical pixels.
    pub rect_in_pixels: Rect,

    /// The pixels the final pass covers: [`Self::rect_in_pixels`] plus room for the fade.
    pub drawn_rect_in_pixels: Rect,

    /// How far outside [`Self::drawn_rect_in_pixels`] the final pass can sample, in
    /// physical pixels, because the rim bends what it shows outwards.
    pub sample_margin: f32,

    /// North west, north east, south west, south east, in physical pixels.
    pub corner_radii: [f32; 4],

    /// How far the edge of the glass fades out, in physical pixels. Zero for a hard edge.
    pub feather: f32,

    /// How far the rim bends what it shows, in physical pixels. Zero switches it off.
    pub refraction: f32,

    /// How bright the lit rim is, from 0 to 1. Zero switches it off.
    pub specular: f32,

    /// How thick the pane is, in physical pixels. Both rim effects live in a band this wide.
    pub thickness: f32,

    /// Unit vector pointing at the light, in screen space. y grows downward.
    pub light: [f32; 2],

    /// Superellipse power. Zero uses the rounded rectangle and its corner radii instead.
    pub squircle: f32,

    /// How hard the rim squeezes what is behind the pane, from 0 to 1.
    pub lens: f32,

    /// How bright the sheen running round the rim is.
    pub sheen: f32,

    /// How much grain is laid over the glass, from 0 to 1.
    pub grain: f32,

    /// How far the colours split where the lens bends. Zero switches it off.
    pub dispersion: f32,
}

/// Which of the three passes a set of uniforms belongs to.
///
/// Each needs its own buffer: `Queue::write_buffer` is applied before any of the commands
/// run, so writing one buffer three times would leave all three passes using the last
/// value.
#[derive(Clone, Copy)]
enum Pass {
    Horizontal = 0,
    Vertical = 1,
    Draw = 2,
}

const PASS_COUNT: usize = 3;

pub(crate) struct BlurResources {
    blur_pipeline: wgpu::RenderPipeline,
    draw_pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    /// Allocated on first use and whenever the target resizes.
    textures: Option<Textures>,

    /// Bumped whenever `textures` is rebuilt, so per-blur bind groups know they are stale.
    generation: u64,

    /// One set of uniforms per blurred widget.
    ///
    /// Every blur in a pass shares this one `BlurResources`, since callback resources are
    /// keyed by type. The scratch textures can be shared, because the passes run one after
    /// another in the encoder, but the uniforms cannot: `Queue::write_buffer` lands before
    /// any command runs, so a shared buffer would leave every blur using the last one's
    /// settings and the rest drawing nothing.
    ///
    /// TODO(lucas): entries for widgets that go away are never dropped. They are a couple
    /// of hundred bytes each, so this is a slow leak rather than a problem, but it should
    /// be pruned once egui-wgpu gives callbacks a frame boundary to hook.
    per_blur: HashMap<Id, PerBlur>,
}

/// The uniforms and bind groups belonging to one blurred widget.
struct PerBlur {
    /// One per [`Pass`].
    uniforms: [wgpu::Buffer; PASS_COUNT],

    /// Reads the first scratch image, writes the second.
    vertical: wgpu::BindGroup,

    /// Reads the second scratch image, writes egui's target.
    draw: wgpu::BindGroup,

    /// Which `BlurResources::generation` the bind groups were built against.
    generation: u64,
}

/// The two intermediate images the separable blur bounces between.
///
/// Shared by every blur in the pass: each one runs its blur passes and then draws, all in
/// encoder order, so nothing is still reading these when the next blur overwrites them.
struct Textures {
    size_in_pixels: [u32; 2],
    format: wgpu::TextureFormat,

    /// Holds the horizontally blurred backdrop.
    first_view: wgpu::TextureView,

    /// Holds the finished blur, and is what the final pass samples.
    second_view: wgpu::TextureView,
}

impl BlurResources {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("blur.wgsl"));
        let layout = create_bind_group_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("regui_backdrop_blur"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        Self {
            blur_pipeline: create_pipeline(
                device,
                &pipeline_layout,
                &shader,
                format,
                "fs_blur",
                None,
            ),
            // Ordinary premultiplied compositing. Inside the rect the blur is opaque and
            // so replaces what is there, which is what we want, and around the rounded
            // corners it fades into the unblurred background instead of cutting a step
            // out of it.
            draw_pipeline: create_pipeline(
                device,
                &pipeline_layout,
                &shader,
                format,
                "fs_draw",
                Some(PREMULTIPLIED),
            ),
            layout,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("regui_backdrop_blur"),
                // Clamping means the blur reuses the edge pixels rather than darkening
                // against nothing when it samples past the edge of the screen.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            textures: None,
            generation: 0,
            per_blur: HashMap::new(),
        }
    }
}

/// Premultiplied alpha compositing: `source + destination * (1 - source alpha)`.
const PREMULTIPLIED: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
};

fn create_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    entry_point: &str,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("regui_backdrop_blur"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(entry_point),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("regui_backdrop_blur"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

impl BlurResources {
    /// Make sure the shared images and this blur's own uniforms exist and are the right size.
    pub(crate) fn update(
        &mut self,
        device: &wgpu::Device,
        id: Id,
        size_in_pixels: [u32; 2],
        format: wgpu::TextureFormat,
    ) {
        self.update_textures(device, size_in_pixels, format);

        let stale = self
            .per_blur
            .get(&id)
            .is_none_or(|per_blur| per_blur.generation != self.generation);
        if stale {
            let per_blur = self.create_per_blur(device);
            self.per_blur.insert(id, per_blur);
        }
    }

    /// Build the uniforms and bind groups for one blurred widget.
    fn create_per_blur(&self, device: &wgpu::Device) -> PerBlur {
        let uniforms: [wgpu::Buffer; PASS_COUNT] = std::array::from_fn(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("regui_backdrop_blur_params"),
                size: Params::SIZE,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        // The horizontal pass reads the backdrop, which is handed to us per frame, so its
        // bind group is built in `blur`. These two only depend on our own textures.
        let (first, second) = match &self.textures {
            Some(textures) => (&textures.first_view, &textures.second_view),
            None => unreachable!("update_textures runs first"),
        };
        PerBlur {
            vertical: self.bind_group(device, first, &uniforms[Pass::Vertical as usize]),
            draw: self.bind_group(device, second, &uniforms[Pass::Draw as usize]),
            uniforms,
            generation: self.generation,
        }
    }

    /// Allocate the intermediate images, or reallocate them if the target changed size.
    fn update_textures(
        &mut self,
        device: &wgpu::Device,
        size_in_pixels: [u32; 2],
        format: wgpu::TextureFormat,
    ) {
        let size_in_pixels = [size_in_pixels[0].max(1), size_in_pixels[1].max(1)];
        let matches = self.textures.as_ref().is_some_and(|textures| {
            textures.size_in_pixels == size_in_pixels && textures.format == format
        });
        if matches {
            return;
        }

        let texture = |label| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: size_in_pixels[0],
                        height: size_in_pixels[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[format],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };

        self.textures = Some(Textures {
            size_in_pixels,
            format,
            first_view: texture("regui_backdrop_blur_first"),
            second_view: texture("regui_backdrop_blur_second"),
        });

        // Every per-blur bind group points at the old images, so they all have to go.
        self.generation += 1;
    }

    fn bind_group(
        &self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
        uniforms: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("regui_backdrop_blur"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniforms.as_entire_binding(),
                },
            ],
        })
    }

    /// Run the two blur passes over the backdrop.
    pub(crate) fn blur(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        backdrop: &Backdrop<'_>,
        id: Id,
        settings: Settings,
    ) {
        let (Some(textures), Some(per_blur)) = (&self.textures, self.per_blur.get(&id)) else {
            return;
        };
        if textures.size_in_pixels != backdrop.size_in_pixels {
            // `prepare` sized these from the screen descriptor, so this should not happen.
            log::warn!(
                "regui: the blur textures are {:?} but the backdrop is {:?}; skipping the blur",
                textures.size_in_pixels,
                backdrop.size_in_pixels
            );
            return;
        }

        let [width, height] = backdrop.size_in_pixels.map(|size| size as f32);
        let target_size = [width, height];

        // A Gaussian is visually finished at three standard deviations, so treat the radius
        // the user asked for as the point where the blur has faded out.
        let sigma = (settings.radius / 3.0).max(0.1);
        let taps = settings.radius.ceil().clamp(1.0, 128.0);

        // Sampling an sRGB texture hands the shader linear values and writing to one
        // converts back, so the tint has to be in whichever space the target works in, or
        // it comes out at the wrong brightness. Both forms are premultiplied.
        let tint = if backdrop.format.is_srgb() {
            egui::Rgba::from(settings.tint).to_array()
        } else {
            settings.tint.to_normalized_gamma_f32()
        };

        let write = |pass: Pass, step: [f32; 2]| {
            queue.write_buffer(
                &per_blur.uniforms[pass as usize],
                0,
                &Params {
                    step,
                    sigma,
                    radius: taps,
                    target_size,
                    feather: settings.feather,
                    tint,
                    rect_min: [settings.rect_in_pixels.min.x, settings.rect_in_pixels.min.y],
                    rect_max: [settings.rect_in_pixels.max.x, settings.rect_in_pixels.max.y],
                    corner_radii: settings.corner_radii,
                    refraction: settings.refraction,
                    specular: settings.specular,
                    thickness: settings.thickness,
                    light: settings.light,
                    squircle: settings.squircle,
                    lens: settings.lens,
                    sheen: settings.sheen,
                    grain: settings.grain,
                    dispersion: settings.dispersion,
                }
                .to_bytes(),
            );
        };
        write(Pass::Horizontal, [1.0 / width, 0.0]);
        write(Pass::Vertical, [0.0, 1.0 / height]);
        write(Pass::Draw, [0.0, 0.0]);

        // Only the pixels the panel will actually show are worth blurring. The final pass
        // reads the vertical pass wherever it draws, plus however far the rim bends its
        // sample outwards; the vertical pass reads the horizontal one `taps` pixels up and
        // down; the horizontal pass reads the backdrop `taps` pixels to each side. So each
        // stage covers the one after it, grown by its own reach. On a panel-sized rect that
        // is a small fraction of the screen the full-screen passes used to cover.
        //
        // Anything outside the piece of the frame that was captured for us is clipped away
        // before it reaches the screen — that is why it was not captured — so trim to that
        // as well. It is what keeps a blur inside a scrolled area cheap.
        let valid = Region::from_array(backdrop.valid_in_pixels);
        let vertical_rect = settings.drawn_rect_in_pixels.expand(settings.sample_margin);
        let vertical = Region::new(vertical_rect, backdrop.size_in_pixels).intersect(valid);
        let horizontal =
            Region::new(vertical_rect.expand(taps), backdrop.size_in_pixels).intersect(valid);
        if vertical.is_empty() || horizontal.is_empty() {
            return;
        }

        let horizontal_bind_group = self.bind_group(
            device,
            backdrop.view,
            &per_blur.uniforms[Pass::Horizontal as usize],
        );
        self.run(
            encoder,
            "regui_backdrop_blur_horizontal",
            &horizontal_bind_group,
            &textures.first_view,
            horizontal,
        );
        self.run(
            encoder,
            "regui_backdrop_blur_vertical",
            &per_blur.vertical,
            &textures.second_view,
            vertical,
        );
    }

    fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        label: &str,
        bind_group: &wgpu::BindGroup,
        output: &wgpu::TextureView,
        region: Region,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    // Not `Clear`: a load op covers the whole attachment however small the
                    // scissor is, so clearing here would put the full-screen cost we are
                    // trying to avoid straight back. Whatever is left outside the region is
                    // from an earlier blur and is never sampled, since every pass that reads
                    // this image stays inside the region we set here.
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_scissor_rect(region.x, region.y, region.width, region.height);
        pass.set_pipeline(&self.blur_pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Draw the finished blur into egui's render pass.
    pub(crate) fn draw(&self, render_pass: &mut wgpu::RenderPass<'static>, id: Id) {
        let Some(per_blur) = self.per_blur.get(&id) else {
            return;
        };
        render_pass.set_pipeline(&self.draw_pipeline);
        render_pass.set_bind_group(0, &per_blur.draw, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
