//! A separable Gaussian blur over a child ui's rendered image.

use egui::Id;
use egui_wgpu::wgpu;
use std::{collections::HashMap, sync::Arc};

/// `Params` in `child_blur.wgsl`: `step`, `sigma`, `radius`, and one float of padding to
/// reach a 16 byte boundary.
const PARAMS_SIZE: u64 = 4 * 4;

fn params_bytes(step: [f32; 2], sigma: f32, radius: f32) -> [u8; PARAMS_SIZE as usize] {
    let mut bytes = [0_u8; PARAMS_SIZE as usize];
    for (chunk, float) in bytes
        .chunks_exact_mut(4)
        .zip([step[0], step[1], sigma, radius])
    {
        chunk.copy_from_slice(&float.to_le_bytes());
    }
    bytes
}

/// Blurs child uis. One of these is shared by every blurred child, with per-child buffers
/// and textures inside.
pub(crate) struct ChildBlur {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    per_child: HashMap<Id, PerChild>,
}

/// One blurred child's own textures and uniforms.
///
/// These cannot be shared the way the backdrop blur shares its scratch images: a child is
/// rendered off-screen during the ui pass and drawn later, so its blurred image has to
/// still be there at the end of the frame, by which time other children have had their
/// turn.
struct PerChild {
    size_in_pixels: [u32; 2],
    format: wgpu::TextureFormat,

    /// The child's render target, which the horizontal bind group reads. Kept so we can
    /// tell when the child got a new one and the bind group has to be rebuilt.
    source: Arc<wgpu::TextureView>,

    /// Holds the horizontally blurred image.
    first: Arc<wgpu::TextureView>,

    /// Holds the finished blur, and is what the parent samples.
    second: Arc<wgpu::TextureView>,

    /// Uniforms for the horizontal and the vertical pass.
    ///
    /// Two buffers rather than one written twice: `Queue::write_buffer` lands before any
    /// command runs, so a single buffer would leave both passes blurring the same way.
    horizontal_params: wgpu::Buffer,
    vertical_params: wgpu::Buffer,

    /// Reads the child's image, writes `first`.
    horizontal: wgpu::BindGroup,

    /// Reads `first`, writes `second`.
    vertical: wgpu::BindGroup,
}

impl ChildBlur {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../child_blur.wgsl"));

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("regui_child_blur"),
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
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("regui_child_blur"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("regui_child_blur"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_blur"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Each pass fills its whole target, so there is nothing to blend with.
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            layout,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("regui_child_blur"),
                // A child ui is transparent around its content, so clamping is what keeps
                // the blur from pulling in nothing at the edges.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            per_child: HashMap::new(),
        }
    }

    /// Blur `source` and return the view holding the result.
    #[expect(clippy::too_many_arguments)] // All of it is needed, and none of it groups well.
    pub(crate) fn run(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        id: Id,
        source: &Arc<wgpu::TextureView>,
        size_in_pixels: [u32; 2],
        format: wgpu::TextureFormat,
        radius: f32,
    ) -> Option<Arc<wgpu::TextureView>> {
        self.update(device, id, source, size_in_pixels, format);
        let per_child = self.per_child.get(&id)?;

        let [width, height] = size_in_pixels.map(|size| size as f32);

        // A Gaussian is visually finished at three standard deviations, so treat the radius
        // asked for as the point where the blur has faded out.
        let sigma = (radius / 3.0).max(0.1);
        let taps = radius.ceil().clamp(1.0, 128.0);

        queue.write_buffer(
            &per_child.horizontal_params,
            0,
            &params_bytes([1.0 / width, 0.0], sigma, taps),
        );
        queue.write_buffer(
            &per_child.vertical_params,
            0,
            &params_bytes([0.0, 1.0 / height], sigma, taps),
        );

        self.pass(encoder, &per_child.horizontal, &per_child.first);
        self.pass(encoder, &per_child.vertical, &per_child.second);

        Some(Arc::clone(&per_child.second))
    }

    /// Make or remake this child's textures, uniforms and bind groups if anything changed.
    ///
    /// The bind groups point at the child's own render target, which is replaced whenever
    /// the child resizes, so they are rebuilt alongside the textures.
    fn update(
        &mut self,
        device: &wgpu::Device,
        id: Id,
        source: &Arc<wgpu::TextureView>,
        size_in_pixels: [u32; 2],
        format: wgpu::TextureFormat,
    ) {
        let matches = self.per_child.get(&id).is_some_and(|per_child| {
            per_child.size_in_pixels == size_in_pixels
                && per_child.format == format
                && Arc::ptr_eq(&per_child.source, source)
        });
        if matches {
            return;
        }

        let first = Arc::new(super::create_view(device, size_in_pixels, format));
        let second = Arc::new(super::create_view(device, size_in_pixels, format));

        let params = |label| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: PARAMS_SIZE,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let horizontal_params = params("regui_child_blur_horizontal");
        let vertical_params = params("regui_child_blur_vertical");

        self.per_child.insert(
            id,
            PerChild {
                size_in_pixels,
                format,
                horizontal: self.bind_group(device, source, &horizontal_params),
                vertical: self.bind_group(device, &first, &vertical_params),
                source: Arc::clone(source),
                first,
                second,
                horizontal_params,
                vertical_params,
            },
        );
    }

    fn bind_group(
        &self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
        params: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("regui_child_blur"),
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
                    resource: params.as_entire_binding(),
                },
            ],
        })
    }

    fn pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bind_group: &wgpu::BindGroup,
        target: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("regui_child_blur"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
