use crate::Transform;
use egui::{
    ClippedPrimitive, Shape, Ui,
    epaint::{Mesh, Primitive},
};
use std::sync::Arc;

/// Paint the child's triangles into the parent's paint list.
///
/// This is the backend that works everywhere: it hands the parent's painter ordinary
/// meshes, so any egui backend can draw them. The catch is clipping. egui clip rectangles
/// are axis-aligned, so once the child is rotated the child's own clip rectangles have to
/// be widened to their bounding boxes, and content that should have been cut off at the
/// edge of a scroll area can spill a little. Use the `wgpu` backend if that matters.
pub(crate) fn paint(ui: &Ui, primitives: Vec<ClippedPrimitive>, transform: Transform) {
    let painter = ui.painter();
    let parent_clip = painter.clip_rect();
    let exact_clipping = transform.is_axis_aligned();

    for ClippedPrimitive {
        clip_rect,
        primitive,
    } in primitives
    {
        let clip_rect = transform.bounding_rect(clip_rect).intersect(parent_clip);
        if !clip_rect.is_positive() {
            continue;
        }

        match primitive {
            Primitive::Mesh(mesh) => {
                painter
                    .with_clip_rect(clip_rect)
                    .add(Shape::Mesh(Arc::new(transform_mesh(mesh, transform))));
            }
            Primitive::Callback(mut callback) => {
                if exact_clipping {
                    callback.rect = transform.bounding_rect(callback.rect);
                    painter
                        .with_clip_rect(clip_rect)
                        .add(Shape::Callback(callback));
                } else {
                    // A paint callback draws into a rectangle of the screen. There is no
                    // way to tell it to draw into a rotated one.
                    log::warn!(
                        "regui: dropping a paint callback inside a rotated child ui, \
                         because a paint callback cannot be rotated"
                    );
                }
            }
        }
    }
}

/// Move the mesh from the child's coordinate space into the parent's.
///
/// `Mesh::transform` cannot rotate and `Mesh::rotate` cannot scale, and both walk every
/// vertex, so do the whole thing in one pass instead.
fn transform_mesh(mut mesh: Mesh, transform: Transform) -> Mesh {
    for vertex in &mut mesh.vertices {
        vertex.pos = transform.mul_pos(vertex.pos);
    }
    mesh
}
