//! Shaders that run over the child ui after it has been rendered.

mod blur;

pub use blur::Blur;

use wgpu::{Device, Queue, TextureFormat, TextureView};

/// A shader pass that runs over the rendered child ui.
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
    /// back.
    ///
    /// Call `ctx.pass(..)` once per full-screen pass you need. To read and write the same
    /// image, use [`EffectContext::scratch`] as an intermediate, the way [`Blur`] does for
    /// its two directions.
    fn run(&self, ctx: &mut EffectContext<'_>, input: &TextureView, output: &TextureView);

    /// How many passes this effect will run, so that the right number of scratch textures
    /// can be prepared.
    ///
    /// Returning too few is not unsafe, it just costs an extra allocation. Defaults to 1.
    fn passes(&self) -> u32 {
        1
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

    /// Spare textures the effect can bounce through, same size and format as the input.
    pub(crate) scratch: &'a [TextureView],
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

    /// A spare texture to bounce through, if the effect needs more than one pass.
    ///
    /// `index` counts from zero. Returns `None` if you ask for more than you promised in
    /// [`Effect::passes`].
    pub fn scratch(&self, index: usize) -> Option<&'a TextureView> {
        self.scratch.get(index)
    }

    /// Draw a full-screen triangle into `output` with the given pipeline and bind group.
    ///
    /// Three vertices are drawn with no vertex buffer bound, so the pipeline's vertex
    /// shader is expected to build its own geometry from `vertex_index`. See `blur.wgsl`
    /// for the usual way to do that: one oversized triangle whose visible part covers the
    /// target exactly, with texture coordinates running 0 to 1 across it.
    pub fn pass(
        &mut self,
        output: &TextureView,
        pipeline: &wgpu::RenderPipeline,
        bind_group: &wgpu::BindGroup,
    ) {
        let mut pass = self
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
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
            });

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
