//! The wgpu objects the backdrop blur needs, and the passes it runs.

use egui::{Color32, Rect};
use egui_wgpu::{Backdrop, wgpu};

/// One `Params` in `blur.wgsl`.
///
/// Written by hand rather than with `bytemuck`, to keep the dependency out.
struct Params {
    step: [f32; 2],
    sigma: f32,
    radius: f32,
    target_size: [f32; 2],
    tint: [f32; 4],
    rect_min: [f32; 2],
    rect_max: [f32; 2],
    corner_radii: [f32; 4],
}

impl Params {
    /// 20 floats: see `Params` in `blur.wgsl`, including the two of padding that keep
    /// `tint` on a 16 byte boundary.
    const SIZE: u64 = 20 * 4;

    fn to_bytes(&self) -> [u8; Self::SIZE as usize] {
        let floats = [
            self.step[0],
            self.step[1],
            self.sigma,
            self.radius,
            self.target_size[0],
            self.target_size[1],
            0.0,
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
        ];
        let mut bytes = [0_u8; Self::SIZE as usize];
        for (chunk, float) in bytes.chunks_exact_mut(4).zip(floats) {
            chunk.copy_from_slice(&float.to_le_bytes());
        }
        bytes
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

    /// North west, north east, south west, south east, in physical pixels.
    pub corner_radii: [f32; 4],
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

    /// One per [`Pass`].
    uniforms: [wgpu::Buffer; PASS_COUNT],

    /// Allocated on first use and whenever the target resizes.
    textures: Option<Textures>,
}

/// The two intermediate images the separable blur bounces between.
struct Textures {
    size_in_pixels: [u32; 2],
    format: wgpu::TextureFormat,

    /// Holds the horizontally blurred backdrop.
    first_view: wgpu::TextureView,

    /// Holds the finished blur, and is what the final pass samples.
    second_view: wgpu::TextureView,

    /// Reads `first_view`, writes `second_view`.
    vertical: wgpu::BindGroup,

    /// Reads `second_view`, writes the egui target.
    draw: wgpu::BindGroup,
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
            uniforms: std::array::from_fn(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("regui_backdrop_blur_params"),
                    size: Params::SIZE,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            }),
            textures: None,
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
    /// Allocate the intermediate images, or reallocate them if the target changed size.
    pub(crate) fn update(
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

        let first_view = texture("regui_backdrop_blur_first");
        let second_view = texture("regui_backdrop_blur_second");

        // The horizontal pass reads the backdrop, which is handed to us per frame, so its
        // bind group is built in `blur`. These two only depend on our own textures.
        self.textures = Some(Textures {
            size_in_pixels,
            format,
            vertical: self.bind_group(device, &first_view, Pass::Vertical),
            draw: self.bind_group(device, &second_view, Pass::Draw),
            first_view,
            second_view,
        });
    }

    fn bind_group(
        &self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
        pass: Pass,
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
                    resource: self.uniforms[pass as usize].as_entire_binding(),
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
        settings: Settings,
    ) {
        let Some(textures) = &self.textures else {
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
                &self.uniforms[pass as usize],
                0,
                &Params {
                    step,
                    sigma,
                    radius: taps,
                    target_size,
                    tint,
                    rect_min: [settings.rect_in_pixels.min.x, settings.rect_in_pixels.min.y],
                    rect_max: [settings.rect_in_pixels.max.x, settings.rect_in_pixels.max.y],
                    corner_radii: settings.corner_radii,
                }
                .to_bytes(),
            );
        };
        write(Pass::Horizontal, [1.0 / width, 0.0]);
        write(Pass::Vertical, [0.0, 1.0 / height]);
        write(Pass::Draw, [0.0, 0.0]);

        let horizontal = self.bind_group(device, backdrop.view, Pass::Horizontal);
        self.run(
            encoder,
            "regui_backdrop_blur_horizontal",
            &horizontal,
            &textures.first_view,
        );
        self.run(
            encoder,
            "regui_backdrop_blur_vertical",
            &textures.vertical,
            &textures.second_view,
        );
    }

    fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        label: &str,
        bind_group: &wgpu::BindGroup,
        output: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.blur_pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Draw the finished blur into egui's render pass.
    pub(crate) fn draw(&self, render_pass: &mut wgpu::RenderPass<'static>) {
        let Some(textures) = &self.textures else {
            return;
        };
        render_pass.set_pipeline(&self.draw_pipeline);
        render_pass.set_bind_group(0, &textures.draw, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
