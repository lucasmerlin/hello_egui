#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod collapse;

use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

/// Re-export of [simple_easing](https://crates.io/crates/simple_easing)
pub mod easing {
    pub use simple_easing::*;
}

pub use collapse::*;
use egui::{Context, Id, Pos2, Rect, Sense, Ui, UiBuilder, Vec2};
use hello_egui_utils::current_scroll_delta;

#[derive(Debug, Clone)]
struct AnimationState {
    source: f32,
    target: f32,
}

type Easing = fn(f32) -> f32;

/// Same as [`Context::animate_bool_with_time`] but with an easing function.
pub fn animate_bool_eased(
    ctx: &Context,
    id: impl Hash + Sized,
    bool: bool,
    easing: Easing,
    time: f32,
) -> f32 {
    let x = ctx.animate_bool_with_time(Id::new(id), bool, time);
    easing(x)
}

/// Same as [`Context::animate_value_with_time`] but with an easing function.
pub fn animate_eased(
    ctx: &Context,
    id: impl Hash + Sized,
    value: f32,
    time: f32,
    easing: Easing,
) -> f32 {
    let id = Id::new(id).with("animate_eased");

    let (source, target) = ctx.memory_mut(|mem| {
        let state = mem.data.get_temp_mut_or_insert_with(id, || AnimationState {
            source: value,
            target: value,
        });

        if state.target != value {
            state.source = state.target;
            state.target = value;
        }
        (state.source, state.target)
    });

    let x = ctx.animate_value_with_time(id, value, time);

    if target == source {
        return target;
    }

    let x = (x - source) / (target - source);
    easing(x) * (target - source) + source
}

/// Animate a position. Useful to e.g. animate swapping items in a list.
/// This is basically a wrapper around [`animate_eased`] that animates both x and y.
/// It will try to correct for scrolling, since in egui, scroll will change a widgets y position.
pub fn animate_position(
    ui: &mut Ui,
    id: impl Hash + Sized,
    value: Pos2,
    time: f32,
    easing: Easing,
    scroll_correction: bool,
) -> Pos2 {
    let id1 = Id::new(id);

    let scroll_offset = if scroll_correction {
        current_scroll_delta(ui)
    } else {
        Vec2::ZERO
    };

    let value = value + scroll_offset;

    let position = Pos2::new(
        animate_eased(ui.ctx(), id1.with("x"), value.x, time, easing),
        animate_eased(ui.ctx(), id1.with("y"), value.y, time, easing),
    );

    position - scroll_offset
}

/// A wrapper around [`animate_position`] that animates the position of a child ui.
pub fn animate_ui_translation(
    ui: &mut Ui,
    id: impl Hash + Sized + Debug + Copy,
    easing: Easing,
    size: Vec2,
    prevent_scroll_animation: bool,
    content: impl FnOnce(&mut Ui),
) -> Rect {
    let (_, response) = ui.allocate_exact_size(size, Sense::hover());
    let rect = response.rect;

    let target_pos = rect.min;

    let current_pos = animate_position(ui, id, target_pos, 1.0, easing, prevent_scroll_animation);

    let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(*ui.layout()));

    let _response = child
        .allocate_new_ui(
            UiBuilder::new().max_rect(Rect::from_min_size(current_pos, rect.size())),
            |ui| {
                ui.allocate_ui(size, |ui| {
                    content(ui);
                });
            },
        )
        .response;

    rect
}

/// Creates a repeating animation based on the current time.
/// Useful for e.g. animating a loading spinner.
/// It will repeatedly go from 0.0 to 1.0 and jump back to 0.0.
pub fn animate_repeating(ui: &mut Ui, easing: Easing, duration: Duration, offset: f32) -> f32 {
    ui.ctx().request_repaint();

    let t = ui.input(|i| i.time as f32 + offset);
    let x = t % duration.as_secs_f32();
    easing(x / duration.as_secs_f32())
}

/// Creates a continuous animation based on the current time.
/// Useful for e.g. animating a bouncing ball.
/// It will repeatedly go from 0.0 to 1.0 and back to 0.0.
pub fn animate_continuous(ui: &mut Ui, easing: Easing, duration: Duration, offset: f32) -> f32 {
    let t = animate_repeating(ui, easing::linear, duration, offset);
    easing::roundtrip(easing(t))
}
