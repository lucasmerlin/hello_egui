//! Render the child ui into an off-screen texture, then draw that texture into the parent.
//!
//! This is what makes effects on the child's own content possible: once the child is an
//! image, a shader can do whatever it likes to it. It also clips exactly when the child is
//! rotated, and keeps text crisp at any scale, because the child is rasterized at the size
//! it ends up being drawn at rather than having its geometry stretched.

use crate::Transform;
use egui::{ClippedPrimitive, Color32, Id, Mesh, Rect, Shape, TextureId, Ui, Vec2, pos2};
use egui_wgpu::{RenderState, ScreenDescriptor, wgpu};
use std::{collections::HashMap, sync::Arc};

mod blur;

pub(crate) use blur::ChildBlur;

/// Everything the off-screen path needs to know for one frame.
pub(crate) struct Request {
    /// A stable id for this child, so its texture is reused between frames.
    pub id: Id,

    /// The child's tessellated shapes.
    pub primitives: Vec<ClippedPrimitive>,

    /// The child's size in its own points.
    pub size: Vec2,

    /// The child's scale, so the texture is allocated at the density it is drawn at.
    pub pixels_per_point: f32,

    /// Maps child coordinates to parent coordinates.
    pub transform: Transform,

    /// Texture uploads from the child's pass, which have to be applied before the child is
    /// rendered: the parent's backend only applies them after the ui is done, and glyphs
    /// the child added this frame would otherwise be missing.
    pub textures_delta: egui::TexturesDelta,

    /// Blur radius over the child's own content, in physical pixels. Zero for none.
    pub blur_radius: f32,
}

/// Per-child GPU state, kept in the renderer's callback resources so it survives between
/// frames without needing to be `Clone`.
#[derive(Default)]
struct ChildTargets(HashMap<Id, ChildTarget>);

struct ChildTarget {
    size_in_pixels: [u32; 2],
    format: wgpu::TextureFormat,

    /// What the child is rendered into.
    view: Arc<wgpu::TextureView>,

    /// How the parent refers to the finished image.
    ///
    /// Registered once and then pointed at a new view whenever the child resizes, so the
    /// meshes the parent has already built stay valid.
    texture_id: TextureId,
}

/// Render the child off-screen and return the mesh that draws it, or `None` if wgpu could
/// not be used.
pub(crate) fn render(ui: &Ui, render_state: &RenderState, request: Request) -> Option<Shape> {
    let Request {
        id,
        primitives,
        size,
        pixels_per_point,
        transform,
        textures_delta,
        blur_radius,
    } = request;

    let size_in_pixels = [
        (size.x * pixels_per_point).ceil().max(1.0) as u32,
        (size.y * pixels_per_point).ceil().max(1.0) as u32,
    ];
    if size_in_pixels[0] > u32::from(u16::MAX) || size_in_pixels[1] > u32::from(u16::MAX) {
        log::warn!("regui: child ui of {size_in_pixels:?} pixels is too big to render");
        // We took ownership of the uploads, so hand them on before giving up. Dropping a
        // `TexturesDelta` that still holds them asserts.
        crate::output::forward_textures_delta(ui.ctx(), textures_delta);
        return None;
    }

    let mut renderer = render_state.renderer.write();

    apply_textures(&mut renderer, render_state, &textures_delta);

    let format = render_state.target_format;
    let target = ensure_target(
        &mut renderer,
        &render_state.device,
        id,
        size_in_pixels,
        format,
    );
    let view = Arc::clone(&target.view);
    let texture_id = target.texture_id;

    let mut encoder = render_state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("regui_child"),
        });

    let user_buffers = draw_child(
        &mut renderer,
        render_state,
        &mut encoder,
        &view,
        &primitives,
        &ScreenDescriptor {
            size_in_pixels,
            pixels_per_point,
        },
    );

    // Blur the child's own image, if asked. The result goes into scratch textures, so the
    // parent is pointed at whichever texture came out last.
    let blurred = (blur_radius > 0.0).then(|| {
        let blur = renderer
            .callback_resources
            .entry()
            .or_insert_with(|| ChildBlur::new(&render_state.device, format));
        blur.run(
            &render_state.device,
            &render_state.queue,
            &mut encoder,
            id,
            &view,
            size_in_pixels,
            format,
            blur_radius,
        )
    });

    let drawn = blurred.flatten().unwrap_or_else(|| Arc::clone(&view));
    renderer.update_egui_texture_from_wgpu_texture(
        &render_state.device,
        &drawn,
        wgpu::FilterMode::Linear,
        texture_id,
    );

    // Submitted now, before the parent's own render, so the image is ready by the time the
    // parent draws the mesh below.
    render_state
        .queue
        .submit(std::iter::once(encoder.finish()).chain(user_buffers));
    drop(renderer);

    // Hand the child's uploads back so the parent's backend sees them too.
    crate::output::forward_textures_delta(ui.ctx(), textures_delta);

    Some(Shape::Mesh(Arc::new(mesh(
        transform,
        Rect::from_min_size(egui::Pos2::ZERO, size),
        texture_id,
    ))))
}

/// Upload the child's textures before it is rendered.
///
/// The child's pass drained this frame's uploads, the parent's included, so without this
/// the child renders with a stale font atlas and glyphs added this frame are missing. The
/// parent's backend applies them again later, which is harmless.
fn apply_textures(
    renderer: &mut egui_wgpu::Renderer,
    render_state: &RenderState,
    textures_delta: &egui::TexturesDelta,
) {
    for (texture_id, image_deltas) in &textures_delta.set {
        for image_delta in image_deltas {
            renderer.update_texture(
                &render_state.device,
                &render_state.queue,
                *texture_id,
                image_delta,
            );
        }
    }
}

/// Render the child's primitives into `view` with the parent's own renderer.
fn draw_child(
    renderer: &mut egui_wgpu::Renderer,
    render_state: &RenderState,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    primitives: &[ClippedPrimitive],
    screen_descriptor: &ScreenDescriptor,
) -> Vec<wgpu::CommandBuffer> {
    let user_buffers = renderer.update_buffers(
        &render_state.device,
        &render_state.queue,
        encoder,
        primitives,
        screen_descriptor,
    );

    let mut pass = encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("regui_child"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
        })
        .forget_lifetime();
    renderer.render(&mut pass, primitives, screen_descriptor);

    user_buffers
}

/// The quad that draws the child's image in the parent, transformed.
fn mesh(transform: Transform, child_rect: Rect, texture_id: TextureId) -> Mesh {
    let mut mesh = Mesh::with_texture(texture_id);
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    mesh.add_rect_with_uv(child_rect, uv, Color32::WHITE);
    for vertex in &mut mesh.vertices {
        vertex.pos = transform.mul_pos(vertex.pos);
    }
    mesh
}

/// Get this child's render target, making or resizing it as needed.
fn ensure_target<'a>(
    renderer: &'a mut egui_wgpu::Renderer,
    device: &wgpu::Device,
    id: Id,
    size_in_pixels: [u32; 2],
    format: wgpu::TextureFormat,
) -> &'a ChildTarget {
    let existing = renderer
        .callback_resources
        .get::<ChildTargets>()
        .and_then(|targets| targets.0.get(&id))
        .is_some_and(|target| target.size_in_pixels == size_in_pixels && target.format == format);

    if !existing {
        let view = Arc::new(create_view(device, size_in_pixels, format));
        // Registering allocates a new `TextureId`, so reuse the one this child already has
        // if it is only resizing. Otherwise meshes drawn for it would point at nothing.
        let previous = renderer
            .callback_resources
            .get::<ChildTargets>()
            .and_then(|targets| targets.0.get(&id))
            .map(|target| target.texture_id);
        let texture_id = previous.unwrap_or_else(|| {
            renderer.register_native_texture(device, &view, wgpu::FilterMode::Linear)
        });

        let targets: &mut ChildTargets = renderer
            .callback_resources
            .entry()
            .or_insert_with(ChildTargets::default);
        targets.0.insert(
            id,
            ChildTarget {
                size_in_pixels,
                format,
                view,
                texture_id,
            },
        );
    }

    #[expect(clippy::unwrap_used)] // Just inserted, or it was already there.
    renderer
        .callback_resources
        .get::<ChildTargets>()
        .unwrap()
        .0
        .get(&id)
        .unwrap()
}

pub(crate) fn create_view(
    device: &wgpu::Device,
    size_in_pixels: [u32; 2],
    format: wgpu::TextureFormat,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("regui_child"),
            size: wgpu::Extent3d {
                width: size_in_pixels[0],
                height: size_in_pixels[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[format],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}
