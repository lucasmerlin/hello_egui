use eframe::{egui::{
    self,
    CursorIcon, Id, InnerResponse, LayerId, Order, Sense,
    Ui,
}, epaint::{Rect, Shape, Vec2}};

pub fn drag_source(
    ui: &mut Ui,
    id: Id,
    drag_handle: impl FnOnce(&mut Ui),
    drag_body: impl FnOnce(&mut Ui),
) -> Option<Rect> {
    let is_being_dragged = ui.memory().is_being_dragged(id);
    let row_resp = ui.horizontal(|gg| {
        let u = gg.scope(drag_handle);

        // Check for drags:
        // let response = ui.interact(response.rect, id, Sense::click());
        let response = gg.interact(u.response.rect, id, Sense::drag());

        if response.hovered() {
            gg.output().cursor_icon = CursorIcon::Grab;
        }

        drag_body(gg)
    });

    if !is_being_dragged {
        return Some(row_resp.response.rect);

        // sponse.clicked() {
        // println!("source clicked")
        // }
    } else {
        ui.output().cursor_icon = CursorIcon::Grabbing;

        // let response = ui.scope(body).response;

        // Paint the body to a new layer:
        let layer_id = LayerId::new(Order::Tooltip, id);
        // let response = ui.with_layer_id(layer_id, body).response;

        // Now we move the visuals of the body to where the mouse is.
        // Normally you need to decide a location for a widget first,
        // because otherwise that widget cannot interact with the mouse.
        // However, a dragged component cannot be interacted with anyway
        // (anything with `Order::Tooltip` always gets an empty [`Response`])
        // So this is fine!

        // if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
        //     let r = response.rect.center();

        //     let delta = pointer_pos - r;
        //     ui.ctx().translate_layer(layer_id, delta);
        // }

        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
            let u = egui::Area::new("draggable_item")
                .interactable(false)
                .fixed_pos(pointer_pos)
                .show(ui.ctx(), |x| {
                    x.horizontal(|gg| {
                        gg.label("dragging meeeee yayyyy")

                        // drag_handle(gg);
                        // drag_body(gg)
                    })
                });

            // u.response.rect.area()
        }
    }

    Some(row_resp.response.rect)
}

pub fn drop_target<R>(
    ui: &mut Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let is_being_dragged = ui.memory().is_anything_being_dragged();
    // let is_being_dragged = false;


    let margin = Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(Shape::Noop);

    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());

    let ret = body(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    // let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

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
