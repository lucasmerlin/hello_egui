use crate::Transform;
use egui::{Event, Pos2, RawInput, Rect, Response, Ui, Vec2, ViewportId, ViewportInfo};

/// Which of the parent's events the child is allowed to see this pass.
#[derive(Clone, Copy)]
pub(crate) struct Gate {
    /// Forward pointer and touch events.
    pub pointer: bool,

    /// Forward keyboard, text and IME events.
    pub keyboard: bool,
}

/// Build the [`RawInput`] for the child viewport out of the parent's input.
pub(crate) fn child_input(
    ui: &Ui,
    viewport_id: ViewportId,
    size: Vec2,
    pixels_per_point: f32,
    to_child: Transform,
    gate: Gate,
    send_pointer_gone: bool,
) -> RawInput {
    let mut input = ui.input(|input| RawInput {
        viewport_id,
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
        // The resolved value, not `raw.max_texture_side`, which the integration only sets
        // on some passes and is `None` on the rest. Getting this wrong is expensive and
        // not obviously a texture problem: it feeds into the font atlas' `TextOptions`, so
        // a child that disagrees with its parent makes egui rebuild the atlas at the start
        // of every pass, twice per frame. The rebuild during the child's pass throws away
        // the atlas the parent's already-laid-out text points into, and the parent renders
        // gibberish.
        max_texture_side: Some(input.max_texture_side),
        time: input.raw.time,
        predicted_dt: input.raw.predicted_dt,
        focused: input.raw.focused,
        system_theme: input.raw.system_theme,
        events: input
            .raw
            .events
            .iter()
            .filter_map(|event| remap_event(event, to_child, gate))
            .collect(),
        ..Default::default()
    });

    if send_pointer_gone {
        input.events.push(Event::PointerGone);
    }

    // `run_hosted_viewport` would inherit the parent's scale, but we want our own, so
    // that text stays crisp when the child is magnified.
    let focused = input.focused;
    input.viewports.insert(
        viewport_id,
        ViewportInfo {
            native_pixels_per_point: Some(pixels_per_point),
            focused: Some(focused),
            ..Default::default()
        },
    );

    input
}

/// Translate one parent event into child space, or drop it if the child may not see it.
fn remap_event(event: &Event, to_child: Transform, gate: Gate) -> Option<Event> {
    let pointer = gate.pointer;
    let keyboard = gate.keyboard;

    match event {
        // Positional: the child's coordinate space is not the parent's.
        Event::PointerMoved(pos) => pointer.then(|| Event::PointerMoved(to_child.mul_pos(*pos))),
        Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers,
        } => pointer.then(|| Event::PointerButton {
            pos: to_child.mul_pos(*pos),
            button: *button,
            pressed: *pressed,
            modifiers: *modifiers,
        }),
        Event::Touch {
            device_id,
            id,
            phase,
            pos,
            force,
        } => pointer.then(|| Event::Touch {
            device_id: *device_id,
            id: *id,
            phase: *phase,
            pos: to_child.mul_pos(*pos),
            force: *force,
        }),

        // Directional: rotate and scale, but do not translate.
        Event::MouseMoved(delta) => pointer.then(|| Event::MouseMoved(to_child.mul_vec(*delta))),
        Event::MouseWheel {
            unit,
            delta,
            phase,
            modifiers,
        } => pointer.then(|| Event::MouseWheel {
            unit: *unit,
            delta: to_child.mul_vec(*delta),
            phase: *phase,
            modifiers: *modifiers,
        }),

        // Pointer events with nothing to remap.
        Event::PointerGone => pointer.then_some(Event::PointerGone),
        Event::Zoom(_) | Event::Rotate(_) => pointer.then(|| event.clone()),

        // Keyboard and clipboard.
        Event::Key { .. }
        | Event::Text(_)
        | Event::Copy
        | Event::Cut
        | Event::Paste(_)
        | Event::Ime(_) => keyboard.then(|| event.clone()),

        // Modifiers gate nothing: the child needs them to interpret its own events, and
        // dropping them would leave it thinking a key is still held.
        Event::ModifiersChanged(_) | Event::WindowFocused(_) => Some(event.clone()),

        // Accessibility and screenshots address a specific widget or viewport, so they
        // are never ours to forward.
        _ => None,
    }
}

/// Should the child see pointer events this pass?
///
/// Yes while the pointer is over us, and yes while the child is being dragged even if the
/// pointer has since left - otherwise dragging a slider would stop the moment you left the
/// child's rect.
pub(crate) fn wants_pointer(response: &Response) -> bool {
    response.contains_pointer() || response.dragged() || response.is_pointer_button_down_on()
}
