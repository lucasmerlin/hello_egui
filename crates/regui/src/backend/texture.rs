//! Render the child ui into an off-screen texture, then draw that texture into the parent.
//!
//! This is what makes effects on the child's own content possible: once the child is an
//! image, a shader can do whatever it likes to it. It also clips exactly when the child is
//! rotated, and keeps text crisp at any scale, because the child is rasterized at the size
//! it ends up being drawn at rather than having its geometry stretched.

use crate::{
    Transform,
    effect::{Effect, EffectContext},
};
use egui::{
    ClippedPrimitive, Color32, Id, Mesh, Rect, Shape, TextureId, Ui, Vec2, epaint::Primitive, pos2,
};
use egui_wgpu::{RenderState, ScreenDescriptor, wgpu};
use std::{collections::HashMap, sync::Arc};

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

    /// Shaders to run over the child's image, in order.
    pub effects: Vec<Box<dyn Effect>>,
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
        mut primitives,
        size,
        pixels_per_point,
        transform,
        effects,
    } = request;

    // An effect spreads outside the child, so give it room to spread into. Without this the
    // texture ends exactly where the child does, and content touching an edge gets smeared
    // along it by the clamping sampler instead of fading out, which looks like the blur has
    // been cut off. Whole pixels, so the child still lands on the pixel grid.
    let asked_for = effects.iter().fold(Vec2::ZERO, |total, effect| {
        total + effect.padding().max(Vec2::ZERO)
    });
    let padding = Vec2::new(
        (asked_for.x * pixels_per_point).ceil() / pixels_per_point,
        (asked_for.y * pixels_per_point).ceil() / pixels_per_point,
    );

    // The child is rendered inset by the padding, and the quad the parent draws covers the
    // padded rect, so an effect fades out beyond the child's own bounds.
    let padded_size = size + 2.0 * padding;
    offset_primitives(&mut primitives, padding);

    let size_in_pixels = [
        (padded_size.x * pixels_per_point).ceil().max(1.0) as u32,
        (padded_size.y * pixels_per_point).ceil().max(1.0) as u32,
    ];
    if size_in_pixels[0] > u32::from(u16::MAX) || size_in_pixels[1] > u32::from(u16::MAX) {
        log::warn!("regui: child ui of {size_in_pixels:?} pixels is too big to render");
        return None;
    }

    let mut renderer = render_state.renderer.write();

    apply_pending_textures(ui, &mut renderer, render_state);

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

    // Run the effects over the child's image. Each writes to a texture of its own, so the
    // parent is pointed at whichever one came out last.
    let drawn = run_effects(
        &mut renderer,
        render_state,
        &mut encoder,
        &effects,
        Chain {
            id,
            source: &view,
            size_in_pixels,
            format,
            pixels_per_point,
        },
    );
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

    Some(Shape::Mesh(Arc::new(mesh(
        transform,
        Rect::from_min_size(egui::Pos2::ZERO, size).expand2(padding),
        texture_id,
    ))))
}

/// What one chain of effects runs over.
#[derive(Clone, Copy)]
struct Chain<'a> {
    id: Id,

    /// The child's own rendered image, which the first effect reads.
    source: &'a Arc<wgpu::TextureView>,

    size_in_pixels: [u32; 2],
    format: wgpu::TextureFormat,
    pixels_per_point: f32,
}

/// Run every effect in turn and return the texture holding the result.
///
/// With no effects this is the child's own image, untouched.
fn run_effects(
    renderer: &mut egui_wgpu::Renderer,
    render_state: &RenderState,
    encoder: &mut wgpu::CommandEncoder,
    effects: &[Box<dyn Effect>],
    chain: Chain<'_>,
) -> Arc<wgpu::TextureView> {
    if effects.is_empty() {
        return Arc::clone(chain.source);
    }

    // Two textures for the effects to hand their images along in, and enough scratch for
    // the greediest single effect. Scratch is shared: an effect only uses it while it runs.
    let scratch_count = effects
        .iter()
        .map(|effect| effect.passes().saturating_sub(1) as usize)
        .max()
        .unwrap_or(0);
    let pool = ensure_pool(
        renderer,
        &render_state.device,
        chain.id,
        2 + scratch_count,
        chain.size_in_pixels,
        chain.format,
    );

    let mut input = Arc::clone(chain.source);
    for (index, effect) in effects.iter().enumerate() {
        // Alternating targets, so an effect never reads the texture it is writing.
        let output = Arc::clone(&pool[index % 2]);
        let mut ctx = EffectContext {
            device: &render_state.device,
            queue: &render_state.queue,
            encoder,
            size: chain.size_in_pixels,
            format: chain.format,
            pixels_per_point: chain.pixels_per_point,
            id: chain.id,
            scratch: &pool[2..],
            resources: &mut renderer.callback_resources,
        };
        effect.run(&mut ctx, &input, &output);
        input = output;
    }
    input
}

/// The textures the effects on one child hand their images along in.
///
/// Per child, not shared: a child is rendered off-screen during the ui pass and drawn at
/// the end of the frame, by which time other children have had their turn.
#[derive(Default)]
struct ChildPools(HashMap<Id, Pool>);

struct Pool {
    size_in_pixels: [u32; 2],
    format: wgpu::TextureFormat,
    views: Vec<Arc<wgpu::TextureView>>,
}

/// Get this child's pool of textures, making or resizing it as needed.
///
/// The views are handed back cloned rather than borrowed, because the effects need the
/// renderer's resource map at the same time.
fn ensure_pool(
    renderer: &mut egui_wgpu::Renderer,
    device: &wgpu::Device,
    id: Id,
    count: usize,
    size_in_pixels: [u32; 2],
    format: wgpu::TextureFormat,
) -> Vec<Arc<wgpu::TextureView>> {
    let pools: &mut ChildPools = renderer
        .callback_resources
        .entry()
        .or_insert_with(ChildPools::default);

    let pool = pools.0.entry(id).or_insert_with(|| Pool {
        size_in_pixels,
        format,
        views: Vec::new(),
    });

    if pool.size_in_pixels != size_in_pixels || pool.format != format {
        pool.size_in_pixels = size_in_pixels;
        pool.format = format;
        pool.views.clear();
    }
    while pool.views.len() < count {
        pool.views
            .push(Arc::new(create_view(device, size_in_pixels, format)));
    }

    pool.views.clone()
}

/// Shift everything the child painted, so it can be rendered inset into a bigger texture.
fn offset_primitives(primitives: &mut [ClippedPrimitive], offset: Vec2) {
    if offset == Vec2::ZERO {
        return;
    }
    for ClippedPrimitive {
        clip_rect,
        primitive,
    } in primitives
    {
        *clip_rect = clip_rect.translate(offset);
        match primitive {
            Primitive::Mesh(mesh) => {
                for vertex in &mut mesh.vertices {
                    vertex.pos += offset;
                }
            }
            Primitive::Callback(callback) => callback.rect = callback.rect.translate(offset),
        }
    }
}

/// Upload this frame's textures before the child is rendered.
///
/// The application's backend only applies them once the ui is done, and we are rendering
/// part way through it, so without this the child gets a stale font atlas and any glyph
/// first drawn this frame is missing from it. Peeking rather than taking leaves the uploads
/// for the backend, which applies them again later; re-uploading the same data is harmless.
fn apply_pending_textures(ui: &Ui, renderer: &mut egui_wgpu::Renderer, render_state: &RenderState) {
    let manager = ui.ctx().tex_manager();
    let manager = manager.read();
    for (texture_id, image_deltas) in &manager.pending_delta().set {
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
