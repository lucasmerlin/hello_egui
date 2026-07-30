use crate::Transform;
use egui::{Context, PlatformOutput, TexturesDelta, ViewportId};
use std::time::Duration;

/// Hand the child's texture uploads back to the [`Context`], so that the parent's own
/// backend picks them up.
///
/// The child's pass runs first, so `Context::end_pass` hands the child _all_ of this
/// pass' texture uploads, including any the parent caused, and the parent is left with
/// nothing. Queueing them again puts them back where the backend will find them.
pub(crate) fn forward_textures_delta(ctx: &Context, mut delta: TexturesDelta) {
    if delta.is_empty() {
        return;
    }

    // Drain rather than destructure: a `TexturesDelta` that still holds unapplied deltas
    // asserts when it is dropped, and it cannot be taken apart because of that.
    let manager = ctx.tex_manager();
    let mut manager = manager.write();
    for (id, image_deltas) in std::mem::take(&mut delta.set) {
        for image_delta in image_deltas {
            manager.set(id, image_delta);
        }
    }
    for id in std::mem::take(&mut delta.free) {
        manager.free(id);
    }
}

/// Merge the child's platform output into the parent's.
///
/// Most of a [`PlatformOutput`] is about the app as a whole rather than one viewport, so
/// it can be merged as-is: opening a url, copying text, or asking for another pass all
/// mean the same thing whichever viewport asked for them. The rest needs care, and is
/// handled here.
pub(crate) fn forward_platform_output(
    ctx: &Context,
    mut output: PlatformOutput,
    to_parent: Transform,
    pointer_is_over_child: bool,
) {
    // The IME rectangles are in child coordinates. Move them into the parent's space, or
    // the OS will put the candidate window in the wrong place.
    if let Some(ime) = &mut output.ime {
        ime.rect = to_parent.bounding_rect(ime.rect);
        ime.cursor_rect = to_parent.bounding_rect(ime.cursor_rect);
    }

    // Each pass produces a whole AccessKit tree, so `append` overwrites rather than
    // merges. Forwarding the child's tree would therefore throw the parent's away, and
    // the app would lose accessibility for everything outside the child. Dropping the
    // child's tree costs less: only the child is inaccessible.
    // TODO(lucas): graft the child's tree onto the parent's node instead.
    output.accesskit_update = None;

    // The parent counts its own passes.
    output.num_completed_passes = 0;

    ctx.output_mut(|parent| {
        // `append` takes the child's cursor unconditionally, but a child the pointer is
        // not even over has no business changing it.
        let parent_cursor = parent.cursor_icon;
        parent.append(output);
        if !pointer_is_over_child {
            parent.cursor_icon = parent_cursor;
        }
    });
}

/// Make the parent repaint whenever the child wants to.
///
/// A repaint request inside the child marks the _child_ viewport as needing a repaint, but
/// the child has no window of its own. Without this, a child animation would run for one
/// pass and then freeze until something else woke the app up.
pub(crate) fn forward_repaint(ctx: &Context, child_id: ViewportId, parent_id: ViewportId) {
    if ctx.has_requested_repaint_for(&child_id) {
        ctx.request_repaint_of(parent_id);
    }

    let delay = ctx.requested_repaint_delay_for(&child_id);
    if delay < Duration::MAX {
        ctx.request_repaint_after_for(delay, parent_id);
    }
}
