use std::cmp::max;
use std::fmt::Debug;
use std::hash::Hash;
use std::ptr::replace;

use egui::{Context, Id, Pos2, Rect, Sense, Ui, Vec2};

#[derive(Debug, Clone)]
struct AnimationState {
    source: f32,
    target: f32,
}

type Easing = fn(f32) -> f32;

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
    ctx: &Context,
    id: impl Hash + Sized,
    value: Pos2,
    time: f32,
    easing: Easing,
) -> Pos2 {
    let id1 = Id::new(id);
    Pos2::new(
        animate_eased(ctx, id1.with("x"), value.x, time, easing),
        animate_eased(ctx, id1.with("y"), value.y, time, easing),
    )
}

pub fn animate_ui_translation(
    ui: &mut Ui,
    id: impl Hash + Sized + Debug + Copy,
    easing: Easing,
    size: Vec2,
    content: impl FnOnce(&mut Ui),
) {
    let (_, response) = ui.allocate_exact_size(size, Sense::hover());
    let rect = response.rect;

    let target_pos = rect.min;
    let current_pos = animate_position(
        ui.ctx(),
        Id::new(id).with("animate_ui_translation"),
        target_pos,
        1.0,
        easing,
    );

    // let max_rect = ui.available_rect_before_wrap();

    let mut child = ui.child_ui(rect, *ui.layout());

    //let rect = max_rect.translate(target_pos.to_vec2() - current_pos.to_vec2());

    let response = child
        .allocate_ui_at_rect(Rect::from_min_size(current_pos, rect.size()), |ui| {
            ui.allocate_ui(size, |ui| {
                content(ui);
            });
        })
        .response;

    dbg!(&id, response.rect.size());
}
