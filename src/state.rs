use std::borrow::BorrowMut;
use std::hash::Hash;
use std::time::SystemTime;

use egui::{Align, CursorIcon, Id, InnerResponse, LayerId, Order, Rect, Sense, Ui, Vec2};

use crate::utils::shift_vec;

/// Item that can be reodered using drag and drop
pub trait DragDropItem {
    /// Unique id for the item, to allow egui to keep track of its dragged state between frames
    fn id(&self) -> Id;
}

impl<T: Hash> DragDropItem for T {
    fn id(&self) -> Id {
        Id::new(self)
    }
}

pub struct Response {
    pub from: usize,
    pub to: usize,
}

/// Response containing the potential list updates during and after a drag & drop event
/// `current_drag` will contain a [Response] when something is being dragged right now and can be
/// used update some state while the drag is in progress.
/// `completed` contains a [Response] after a successful drag & drop event. It should be used to
/// update positions of the affected items. If the source is a vec, [shift_vec] can be used.
pub struct DragDropResponse {
    pub current_drag: Option<Response>,
    pub completed: Option<Response>,
}

/// Holds the data needed to draw the floating item while it is being dragged
#[derive(Default, Clone, Debug)]
pub struct DragDropUi {
    detection_state: DragDetectionState,
}

/// [Handle::ui] is used to draw the drag handle
pub struct Handle<'a> {
    id: Id,
    state: &'a mut DragDropUi,
}

#[derive(Debug, Default, Clone)]
enum DragDetectionState {
    #[default]
    None,
    PressedWaitingForDelay {
        pressed_at: SystemTime,
    },
    WaitingForClickThreshold,
    CouldBeValidDrag,
    Cancelled,
    Dragging { id: Id, offset: Vec2, hovering_idx: Option<usize>, source_idx: Option<usize>, dragged_item_size: Option<Vec2> },
}

impl<'a> Handle<'a> {
    pub fn ui_impl(self, ui: &mut Ui, sense: Option<Sense>, contents: impl FnOnce(&mut Ui)) -> egui::Response {
        let u = ui.scope(contents);

        let response = ui.interact(u.response.rect, self.id, sense.unwrap_or(Sense::hover()));

        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = CursorIcon::Grab);
        }

        let offset = response.rect.min.to_vec2()
            - response
            .hover_pos()
            .unwrap_or_default()
            .to_vec2();

        let drag_distance = ui.input(|i| {
            (i.pointer.hover_pos().unwrap_or_default() - i.pointer.press_origin().unwrap_or_default()).length()
        });

        let click_threshold = 1.0;
        let is_above_click_threshold = drag_distance > click_threshold;

        if response.hovered() && matches!(self.state.detection_state, DragDetectionState::WaitingForClickThreshold) {
            // It should be save to stop anything else being dragged here
            // This is important so any ScrollArea isn't being dragged while we wait for the click threshold
            ui.memory_mut(|mem| mem.stop_dragging());
            if is_above_click_threshold {
                self.state.detection_state = DragDetectionState::CouldBeValidDrag;
            }
        }

        if response.hovered() && matches!(self.state.detection_state, DragDetectionState::CouldBeValidDrag) {
            self.state.detection_state = DragDetectionState::Dragging {
                id: self.id,
                offset,
                hovering_idx: None,
                source_idx: None,
                dragged_item_size: None,
            };
            ui.memory_mut(|mem| mem.set_dragged_id(self.id));
        }

        return response;
    }

    /// Draw the drag handle
    pub fn ui_sense(self, ui: &mut Ui, sense: Sense, contents: impl FnOnce(&mut Ui)) -> egui::Response {
        self.ui_impl(ui, Some(sense), contents)
    }

    /// Draw the drag handle
    pub fn ui(self, ui: &mut Ui, contents: impl FnOnce(&mut Ui)) -> egui::Response {
        self.ui_impl(ui, None, contents)
    }
}

/// [DragDropUi] stores the state of the Drag & Drop list.
/// # Example
/// ```rust,no_run
/// use egui_dnd::DragDropUi;
/// use eframe::App;
/// use eframe::egui::Context;
/// use eframe::Frame;
/// use eframe::egui::CentralPanel;
/// use egui_dnd::utils::shift_vec;
///
/// struct DnDApp {
///     items: Vec<String>,
///     dnd: DragDropUi,
/// }
///
///
/// impl App for DnDApp {
///     fn update(&mut self, ctx: &Context, frame: &mut Frame) {
///         CentralPanel::default().show(ctx, |ui| {
///             let response = self.dnd.ui(ui, self.items.iter_mut(), |item, ui, handle| {
///                 ui.horizontal(|ui| {
///                     handle.ui(ui, |ui| {
///                         ui.label("grab");
///                     });
///                     ui.label(item.clone());
///                 });
///             });
///             if let Some(response) = response.completed {
///                 shift_vec(response.from, response.to, &mut self.items);
///             }
///         });
///     }
/// }
///
/// use eframe::NativeOptions;
/// let dnd = DragDropUi::default();
/// eframe::run_native("DnD Example", NativeOptions::default(), Box::new(|_| {
///     Box::new(DnDApp {
///         dnd: DragDropUi::default(),
///         items: vec!["a", "b", "c"].into_iter().map(|s| s.to_string()).collect(),
///     })
/// }));
/// ```
impl DragDropUi {
    /// Draw the dragged item and check if it has been dropped
    pub fn ui<'a, T: DragDropItem + 'a, B>(
        &mut self,
        ui: &mut Ui,
        values: impl Iterator<Item=B>,
        mut item_ui: impl FnMut(&mut T, &mut Ui, Handle),
    ) -> DragDropResponse
        where B: BorrowMut<T> {
        ui.input(|i| {
            if i.pointer.any_down() {
                let mobile_scroll = i.any_touches();
                let scroll_tolerance = 6.0;
                let drag_delay = std::time::Duration::from_millis(if mobile_scroll { 300 } else { 0 });


                if let DragDetectionState::None = self.detection_state {
                    self.detection_state = DragDetectionState::PressedWaitingForDelay {
                        pressed_at: SystemTime::now(),
                    };
                }

                let drag_distance = (i.pointer.hover_pos().unwrap_or_default() - i.pointer.press_origin().unwrap_or_default()).length();
                let is_below_scroll_threshold = drag_distance < scroll_tolerance;

                if let DragDetectionState::PressedWaitingForDelay { pressed_at } = self.detection_state {
                    if pressed_at.elapsed().unwrap_or(drag_delay) >= drag_delay {
                        if is_below_scroll_threshold || !mobile_scroll {
                            self.detection_state = DragDetectionState::WaitingForClickThreshold;
                        } else {
                            self.detection_state = DragDetectionState::Cancelled;
                        }
                    } else if !is_below_scroll_threshold {
                        self.detection_state = DragDetectionState::Cancelled;
                    }
                }
            }
        });

        let mut vec = values.enumerate().collect::<Vec<_>>();
        let len = vec.len();

        if let DragDetectionState::Dragging { hovering_idx, source_idx, .. } = &mut self.detection_state {
            if let (Some(hovering_idx), Some(source_idx)) = (hovering_idx, source_idx) {
                shift_vec(*source_idx, *hovering_idx, &mut vec);
            }
        }

        let mut rects = Vec::with_capacity(vec.len());

        DragDropUi::drop_target(ui, true, |ui| {
            vec.into_iter().for_each(|(idx, mut item)| {


                let rect = self.drag_source(ui, item.borrow_mut().id(), |ui, handle| {
                    item_ui(item.borrow_mut(), ui, handle);
                });
                rects.push((idx, rect));

                if let DragDetectionState::Dragging { id, source_idx, .. } = &mut self.detection_state {
                    if item.borrow().id() == *id {
                        *source_idx = Some(idx);
                    }
                }
            });
        });

        if let DragDetectionState::Dragging { id, offset, hovering_idx, source_idx, .. } = &mut self.detection_state {
            let pos = ui.input(|i| i.pointer.hover_pos());

            if let Some(pos) = pos {
                let pos = pos + *offset;

                let mut closest: Option<(f32, usize, usize, Rect)> = None;

                rects
                    .into_iter()
                    .enumerate()
                    .for_each(|(new_idx, (idx, rect))| {
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

                if let Some((_dist, new_idx, _original_idx, rect)) = closest {
                    let mut i = if pos.y > rect.center().y {
                        new_idx + 1
                    } else {
                        new_idx
                    };

                    if let Some(idx) = *source_idx {
                        if i > idx && i < len {
                            i += 1;
                        }
                    }

                    *hovering_idx = Some(i);
                }
            }
        }

        let response = if let DragDetectionState::Dragging {
            source_idx: Some(source_idx), hovering_idx: Some(hovering_idx), ..
        } = &self.detection_state {
            if ui.input(|i| i.pointer.any_released()) {
                DragDropResponse {
                    completed: Some(Response {
                        from: *source_idx,
                        to: *hovering_idx,
                    }),
                    current_drag: None,
                }
            } else {
                DragDropResponse {
                    current_drag: Some(Response {
                        from: *source_idx,
                        to: *hovering_idx,
                    }),
                    completed: None,
                }
            }
        } else {
            DragDropResponse {
                current_drag: None,
                completed: None,
            }
        };

        ui.input(|input| {
            if !input.pointer.any_down() {
                self.detection_state = DragDetectionState::None;
            }
        });

        // We are not over any target, cancel the drag
        if let DragDetectionState::CouldBeValidDrag = self.detection_state {
            self.detection_state = DragDetectionState::Cancelled;
        }

        response
    }

    fn drag_source(
        &mut self,
        ui: &mut Ui,
        id: Id,
        drag_body: impl FnOnce(&mut Ui, Handle),
    ) -> Rect {
        if let DragDetectionState::Dragging { id: dragging_id, offset, dragged_item_size, .. } = &mut self.detection_state {
            if id == *dragging_id {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);

                // let response = ui.scope(body).response;

                // Paint the body to a new layer:
                let _layer_id = LayerId::new(Order::Tooltip, id);
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

                let pointer_pos = ui
                    .ctx()
                    .pointer_interact_pos()
                    .unwrap_or_else(|| ui.next_widget_position());

                ui.scroll_to_rect(Rect::from_center_size(pointer_pos, Vec2::splat(100.0)), None);

                let u = egui::Area::new("draggable_item")
                    .interactable(false)
                    .fixed_pos(pointer_pos + *offset)
                    .show(ui.ctx(), |x| {
                        // allocate space where the item would be
                        x.scope(|gg| {
                            //gg.label("dragging meeeee yayyyy")

                            drag_body(gg, Handle { state: self, id })
                        })
                            .response
                            .rect
                    });

                if let DragDetectionState::Dragging { dragged_item_size, .. } = &mut self.detection_state {
                    *dragged_item_size = Some(u.inner.size());
                }

                return ui.allocate_space(u.inner.size()).1;
            }
        }

        let scope = ui.scope(|ui| drag_body(ui, Handle { state: self, id }));
        scope.response.rect
    }

    fn drop_target<R>(
        ui: &mut Ui,
        _can_accept_what_is_being_dragged: bool,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let margin = Vec2::splat(4.0);

        let outer_rect_bounds = ui.available_rect_before_wrap();
        let inner_rect = outer_rect_bounds.shrink2(margin);

        let mut content_ui = ui.child_ui(inner_rect, *ui.layout());

        let ret = body(&mut content_ui);
        let outer_rect =
            Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
        let (_rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

        InnerResponse::new(ret, response)
    }
}
