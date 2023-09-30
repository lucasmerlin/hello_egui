mod collapse;

use std::fmt::Debug;
use std::hash::Hash;

pub use collapse::*;
use egui::{Context, Id, Pos2, Rect, Sense, Ui, Vec2};
use hello_egui_utils::current_scroll_delta;

#[derive(Debug, Clone)]
struct AnimationState {
    source: f32,
    target: f32,
}

type Easing = fn(f32) -> f32;

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

    let mut child = ui.child_ui(rect, *ui.layout());

    let _response = child
        .allocate_ui_at_rect(Rect::from_min_size(current_pos, rect.size()), |ui| {
            ui.allocate_ui(size, |ui| {
                content(ui);
            });
        })
        .response;

    rect
}
