use eframe::{
    egui::{InnerResponse, Sense, Ui},
    epaint::{Rect, Shape, Vec2},
};

pub mod state;

pub fn drop_target<R>(
    ui: &mut Ui,
    _can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let _is_being_dragged = ui.memory().is_anything_being_dragged();
    // let is_being_dragged = false;

    let margin = Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let _where_to_put_background = ui.painter().add(Shape::Noop);

    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());

    let ret = body(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    // let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());
    let (_rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

    // let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
    //     ui.visuals().widgets.active
    // } else {
    //     ui.visuals().widgets.inactive
    // };

    // let mut fill = style.bg_fill;
    // let mut stroke = style.bg_stroke;
    // let mut stroke: Stroke = (0.4, Color32::RED).into();
    // if is_being_dragged && !can_accept_what_is_being_dragged {
    //     // gray out:
    //     fill = ecolor::tint_color_towards(fill, ui.visuals().window_fill());
    //     stroke.color = ecolor::tint_color_towards(stroke.color, ui.visuals().window_fill());
    // }

    // ui.painter().set(
    //     where_to_put_background,
    //     epaint::RectShape {
    //         rounding: style.rounding,
    //         fill,
    //         stroke,
    //         rect,
    //     },
    // );

    InnerResponse::new(ret, response)
}
