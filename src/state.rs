use std::process::id;
use eframe::egui;
use eframe::egui::{CursorIcon, Id, LayerId, Order, Pos2, Rect, Sense, Ui, Vec2};

use crate::{ drop_target};

pub trait DragDropItem {
    fn id(&self) -> Id;
}


pub struct Response {
    pub from: usize,
    pub to: usize,
}

pub struct DragDropResponse {
    pub current_drag: Option<Response>,
    pub completed: Option<Response>,
}

#[derive(Default)]
pub struct DragDropUi {
    source_idx: Option<usize>,
    hovering_idx: Option<usize>,

    drag_delta: Option<Vec2>,
}

impl DragDropUi {
    pub fn ui<'a, T: DragDropItem + 'a>(&mut self, ui: &mut Ui, values: impl Iterator<Item=&'a mut T>, mut item_ui: impl FnMut(&mut T, &mut Ui) -> ()) -> DragDropResponse {
        let mut vec = values.enumerate().collect::<Vec<_>>();

        if let (Some(hovering_idx), Some(source_idx)) = (self.hovering_idx, self.source_idx) {
            shift_vec(source_idx, hovering_idx, &mut vec);
        }

        let mut rects = Vec::with_capacity(vec.len());

        let response = drop_target(ui, true, |ui| {
            vec.iter_mut().for_each(|(idx, item)| {
                let rect = self.drag_source(ui, item.id(), |ui| { ui.label("grab me"); }, |ui| {
                    item_ui(item, ui);
                });
                rects.push((*idx, rect));

                if ui.memory().is_being_dragged(item.id()) {
                    self.source_idx = Some(*idx);
                }
            });
        }).response;


        if ui.memory().is_anything_being_dragged() {
            let pos = ui.input().pointer.hover_pos();


            if let Some(pos) = pos {
                let pos = if let Some(delta) = self.drag_delta {
                    pos + delta
                } else {
                    pos
                };

                let mut closest: Option<(f32, usize, usize, Rect)> = None;

                let hovering = rects.into_iter().enumerate().for_each(|(new_idx, (idx, rect))| {
                    let dist = (rect.top() - pos.y).abs();
                    let val = (dist, new_idx, idx, rect);

                    if let Some((closest_dist, ..)) = closest {
                        if closest_dist > dist {
                            closest = Some(val)
                        }
                    } else {
                        closest = Some(val)
                    }
                });


                if let Some((_dist, new_idx, original_idx, rect)) = closest {
                    let mut i = if pos.y > rect.center().y {
                        new_idx + 1
                    } else {
                        new_idx
                    };

                    if let Some(idx) = self.source_idx {
                        if i > idx && i < vec.len() {
                            i += 1;
                        }
                    }

                    self.hovering_idx = Some(i);
                }
            }
        }


        if let (Some(target_idx), Some(source_idx)) = (self.hovering_idx, self.source_idx) {
            ui.label(format!("hovering: {}", target_idx));

            if ui.input().pointer.any_released() {
                self.source_idx = None;
                self.hovering_idx = None;

                return DragDropResponse {
                    completed: Some(Response {
                        from: source_idx,
                        to: target_idx,
                    }),
                    current_drag: None,
                };
            }

            return DragDropResponse {
                current_drag: Some(Response {
                    from: source_idx,
                    to: target_idx,
                }),
                completed: None,
            };
        }

        DragDropResponse {
            current_drag: None,
            completed: None,
        }
    }


    pub fn drag_source(
        &mut self,
        ui: &mut Ui,
        id: Id,
        drag_handle: impl FnOnce(&mut Ui),
        drag_body: impl FnOnce(&mut Ui),
    ) -> Rect {
        let is_being_dragged = ui.memory().is_being_dragged(id);


        if !is_being_dragged {
            let row_resp = ui.horizontal(|gg| {
                let u = gg.scope(drag_handle);

                // Check for drags:
                // let response = ui.interact(response.rect, id, Sense::click());
                let response = gg.interact(u.response.rect, id, Sense::drag());

                if response.hovered() {
                    gg.output().cursor_icon = CursorIcon::Grab;
                }

                if response.drag_started() {
                    self.drag_delta = Some(u.response.rect.min.to_vec2() - response.interact_pointer_pos().unwrap_or(Pos2::default()).to_vec2());
                }

                drag_body(gg)
            });

            return row_resp.response.rect;

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
            //
            //     let delta = pointer_pos - r;
            //     ui.ctx().translate_layer(layer_id, delta);
            // }

            let pointer_pos = ui.ctx().pointer_interact_pos().unwrap_or(ui.next_widget_position());

            dbg!(self.drag_delta);

            let u = egui::Area::new("draggable_item")
                .interactable(false)
                .fixed_pos(pointer_pos + self.drag_delta.unwrap_or(Vec2::default()))
                .show(ui.ctx(), |x| {
                    let rect = x.horizontal(|gg| {
                        //gg.label("dragging meeeee yayyyy")

                        drag_handle(gg);
                        drag_body(gg)
                    }).response.rect;

                    // allocate space where the item would be
                    return rect;
                });


            return ui.allocate_space(u.inner.size()).1;
        }
    }
}

pub fn shift_vec<T>(source_idx: usize, target_idx: usize, vec: &mut Vec<T>) {
    let target_idx = if source_idx >= target_idx {
        target_idx
    } else {
        target_idx - 1
    };

    let item = vec.remove(source_idx);
    vec.insert(target_idx, item);
}
