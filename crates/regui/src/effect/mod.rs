//! Shaders that run over the child ui after it has been rendered.

mod blur;
mod dissolve;
mod motion_blur;
mod shadow;
mod util;

pub use blur::Blur;
pub use dissolve::Dissolve;
pub use motion_blur::MotionBlur;
pub use shadow::Shadow;

use egui::{Id, Vec2};
use egui_wgpu::wgpu::{self, Device, Queue, TextureFormat, TextureView};
use std::sync::Arc;

/// A shader pass that runs over the rendered child ui.
///
/// Add one with [`crate::Regui::effect`]. Effects run in the order they were added, each
/// one reading what the one before it wrote.
///
/// Effects are only available with the `wgpu` feature, because they need the child to be
/// rendered into a texture first.
///
/// Implement this to run your own shaders over a child ui. [`Blur`] is a worked example.
pub trait Effect: Send + Sync {
    /// Run the effect.
    ///
    /// Read from `input` and write to `output`. Both are the same size and format, and
    /// hold premultiplied alpha, which is what egui renders and what the parent expects
    /// back. They are never the same texture, so it is safe to read one while writing the
    /// other.
    ///
    /// Call `ctx.pass(..)` once per full-screen pass you need. To read and write the same
    /// image, use [`EffectContext::scratch`] as an intermediate, the way [`Blur`] does for
    /// its two directions.
    ///
    /// Do not build pipelines and buffers again every frame. Keep them in
    /// [`EffectContext::resources`], keyed by [`EffectContext::id`] where they belong to
    /// one child.
    fn run(&self, ctx: &mut EffectContext<'_>, input: &TextureView, output: &TextureView);

    /// How many full-screen passes this effect runs.
    ///
    /// One less than this many scratch textures are prepared, since the last pass writes
    /// to `output`. Defaults to 1, which asks for no scratch at all.
    ///
    /// Returning too few is not unsafe: [`EffectContext::scratch`] then hands back `None`
    /// and the effect has to cope.
    fn passes(&self) -> u32 {
        1
    }

    /// How far outside the child this effect paints, in points.
    ///
    /// A blur spreads, a shadow is thrown to one side, and both would be cut off at the
    /// child's own edge. Whatever is asked for here is added around the child before it is
    /// rendered, so the effect has somewhere to spread into.
    ///
    /// The room asked for by every effect in the chain is added up, so an effect only has
    /// to account for itself. Defaults to nothing.
    fn padding(&self) -> Vec2 {
        Vec2::ZERO
    }
}

/// What an [`Effect`] gets to work with.
pub struct EffectContext<'a> {
    pub(crate) device: &'a Device,
    pub(crate) queue: &'a Queue,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,

    /// The size of every texture involved, in physical pixels.
    pub(crate) size: [u32; 2],

    pub(crate) format: TextureFormat,

    /// How many physical pixels there are to a point.
    pub(crate) pixels_per_point: f32,

    /// Which child is being drawn.
    pub(crate) id: Id,

    /// Spare textures the effect can bounce through, same size and format as the input.
    pub(crate) scratch: &'a [Arc<TextureView>],

    /// Somewhere to keep pipelines and buffers between frames.
    pub(crate) resources: &'a mut egui_wgpu::CallbackResources,
}

impl<'a> EffectContext<'a> {
    /// The wgpu device, for creating pipelines and buffers.
    pub fn device(&self) -> &'a Device {
        self.device
    }

    /// The wgpu queue, for uploading uniforms.
    pub fn queue(&self) -> &'a Queue {
        self.queue
    }

    /// The size of the textures, in physical pixels.
    pub fn size(&self) -> [u32; 2] {
        self.size
    }

    /// The format every texture in this effect chain uses.
    pub fn format(&self) -> TextureFormat {
        self.format
    }

    /// How many physical pixels there are to a point.
    ///
    /// Effects are set up in points, like the rest of egui, but they work in pixels. This
    /// is what converts between the two.
    pub fn pixels_per_point(&self) -> f32 {
        self.pixels_per_point
    }

    /// Which child ui is being drawn.
    ///
    /// The same effect can be on several children at once, and each of them needs its own
    /// buffers and bind groups. Use this as the key for them.
    pub fn id(&self) -> Id {
        self.id
    }

    /// A spare texture to bounce through, if the effect needs more than one pass.
    ///
    /// `index` counts from zero. Returns `None` if you ask for more than you promised in
    /// [`Effect::passes`].
    pub fn scratch(&self, index: usize) -> Option<&'a TextureView> {
        self.scratch.get(index).map(Arc::as_ref)
    }

    /// Where to keep anything that has to outlive the frame.
    ///
    /// This is the renderer's own type map, shared by every effect and every callback in
    /// the application, so key it on a type of your own:
    ///
    /// ```ignore
    /// let cache: &mut MyCache = ctx.resources().entry().or_insert_with(MyCache::default);
    /// ```
    ///
    /// Building a pipeline every frame is slow enough to see, so this is not optional.
    pub fn resources(&mut self) -> &mut egui_wgpu::CallbackResources {
        self.resources
    }

    /// The encoder every pass is recorded into, for work [`Self::pass`] cannot do.
    pub fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.encoder
    }

    /// Draw a full-screen triangle into `output` with the given pipeline and bind group.
    ///
    /// Three vertices are drawn with no vertex buffer bound, so the pipeline's vertex
    /// shader is expected to build its own geometry from `vertex_index`. See `blur.wgsl`
    /// for the usual way to do that: one oversized triangle whose visible part covers the
    /// target exactly, with texture coordinates running 0 to 1 across it.
    ///
    /// The target is cleared first, so the pass has to draw everything it wants to keep.
    pub fn pass(
        &mut self,
        output: &TextureView,
        pipeline: &wgpu::RenderPipeline,
        bind_group: &wgpu::BindGroup,
    ) {
        let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("regui effect"),
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

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
