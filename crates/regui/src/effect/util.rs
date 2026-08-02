//! The parts every effect needs: a bind group layout, a pipeline, and a uniform buffer.
//!
//! Effects here all have the same shape. They read one or more textures through a sampler,
//! read a handful of floats, and draw one full-screen triangle. This is that shape, so an
//! effect only has to bring its own shader.

use egui_wgpu::wgpu;

/// Bindings for `textures` texture slots, a sampler, and one uniform buffer.
///
/// The textures come first, at bindings 0 to `textures - 1`. The sampler and the uniforms
/// follow. Shaders in this module are written to match.
pub(crate) fn bind_group_layout(
    device: &wgpu::Device,
    label: &str,
    textures: u32,
) -> wgpu::BindGroupLayout {
    let mut entries: Vec<_> = (0..textures)
        .map(|binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        })
        .collect();

    entries.push(wgpu::BindGroupLayoutEntry {
        binding: textures,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    });
    entries.push(wgpu::BindGroupLayoutEntry {
        binding: textures + 1,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    });

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &entries,
    })
}

/// A pipeline that draws one full-screen triangle, with no vertex buffer.
///
/// The vertex shader is always `vs_main`; the fragment entry point is the effect's own.
/// Blending is off: each pass fills its whole target, so there is nothing to blend with.
pub(crate) fn pipeline(
    device: &wgpu::Device,
    label: &str,
    shader: &wgpu::ShaderModule,
    fragment_entry: &str,
    layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry),
            targets: &[Some(wgpu::ColorTargetState {
                format,
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
    })
}

/// A linear sampler that clamps.
///
/// A child ui is transparent around its content, so clamping is what keeps an effect from
/// pulling in nothing at the edges.
pub(crate) fn sampler(device: &wgpu::Device, label: &str) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    })
}

/// A bind group matching [`bind_group_layout`], in the same order.
pub(crate) fn bind_group(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    textures: &[&wgpu::TextureView],
    sampler: &wgpu::Sampler,
    params: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let mut entries: Vec<_> = textures
        .iter()
        .enumerate()
        .map(|(index, view)| wgpu::BindGroupEntry {
            binding: index as u32,
            resource: wgpu::BindingResource::TextureView(view),
        })
        .collect();

    entries.push(wgpu::BindGroupEntry {
        binding: textures.len() as u32,
        resource: wgpu::BindingResource::Sampler(sampler),
    });
    entries.push(wgpu::BindGroupEntry {
        binding: textures.len() as u32 + 1,
        resource: params.as_entire_binding(),
    });

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

/// How big a uniform buffer holding `floats` floats has to be.
///
/// WGSL rounds a uniform struct up to 16 bytes, so the buffer has to be at least that big
/// however few floats are actually in it.
pub(crate) const fn uniform_size(floats: usize) -> u64 {
    (floats as u64 * 4).next_multiple_of(16)
}

/// A uniform buffer for `floats` floats, to be filled with [`write_floats`].
pub(crate) fn uniform_buffer(device: &wgpu::Device, label: &str, floats: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: uniform_size(floats),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// Upload floats to a uniform buffer, in the order the shader's `Params` declares them.
///
/// The caller is trusted to keep that order and the shader's alignment in step; nothing
/// here can check it.
pub(crate) fn write_floats(queue: &wgpu::Queue, buffer: &wgpu::Buffer, floats: &[f32]) {
    let mut bytes = Vec::with_capacity(uniform_size(floats.len()) as usize);
    for float in floats {
        bytes.extend_from_slice(&float.to_le_bytes());
    }
    bytes.resize(uniform_size(floats.len()) as usize, 0);
    queue.write_buffer(buffer, 0, &bytes);
}
