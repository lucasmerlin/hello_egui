use std::borrow::BorrowMut;
use std::hash::Hash;
use std::time::SystemTime;

use egui::{CursorIcon, Id, InnerResponse, LayerId, Order, Pos2, Rect, Sense, Ui, Vec2};

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

#[derive(Debug, Clone, Copy)]
pub struct Response {
    pub from: usize,
    pub to: usize,
}

/// Response containing the potential list updates during and after a drag & drop event
/// `current_drag` will contain a [Response] when something is being dragged right now and can be
/// used update some state while the drag is in progress.
/// `completed` contains a [Response] after a successful drag & drop event. It should be used to
/// update positions of the affected items. If the source is a vec, [shift_vec] can be used.
#[derive(Debug, Default, Clone)]
pub struct DragDropResponse {
    pub current_drag: Option<Response>,
    pub completed: Option<Response>,
}

/// Holds the data needed to draw the floating item while it is being dragged
#[derive(Default, Clone, Debug)]
pub struct DragDropUi {
    detection_state: DragDetectionState,
    drag_count: usize,
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
    Dragging { id: Id, offset: Vec2, phase: DragPhase },
    TransitioningBackAfterDragFinished {
        from: Pos2,
    },
}

#[derive(Debug, Clone)]
enum DragPhase {
    FirstFrame,
    Rest {
        hovering_idx: usize,
        source_idx: usize,
        dragged_item_size: Vec2,
    },
}

impl DragPhase {
    fn is_first_frame(&self) -> bool {
        matches!(self, DragPhase::FirstFrame)
    }
}

impl DragDetectionState {
    fn is_dragging(&self) -> bool {
        matches!(self, DragDetectionState::Dragging { .. })
    }

    fn dragged_item(&self) -> Option<Id> {
        match self {
            DragDetectionState::Dragging { id, .. } => Some(*id),
            _ => None,
        }
    }

    fn is_dragging_item(&self, id: Id) -> bool {
        self.dragged_item() == Some(id)
    }

    fn offset(&self) -> Option<Vec2> {
        match self {
            DragDetectionState::Dragging { offset, .. } => Some(*offset),
            _ => None,
        }
    }

    fn dragged_item_size(&self) -> Option<Vec2> {
        match self {
            DragDetectionState::Dragging { phase: DragPhase::Rest { dragged_item_size, .. }, .. } => Some(*dragged_item_size),
            _ => None,
        }
    }

    fn hovering_index(&self) -> Option<usize> {
        match self {
            DragDetectionState::Dragging { phase: DragPhase::Rest { hovering_idx, .. }, .. } => Some(*hovering_idx),
            _ => None,
        }
    }
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
                phase: DragPhase::FirstFrame,
            };
            self.state.drag_count += 1;
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
        mut item_ui: impl FnMut(&mut T, &mut Ui, Handle, bool),
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

        // if let DragDetectionState::Dragging { hovering_idx, source_idx, .. } = &mut self.detection_state {
        //     if let (Some(hovering_idx), Some(source_idx)) = (hovering_idx, source_idx) {
        //         shift_vec(*source_idx, *hovering_idx, &mut vec);
        //     }
        // }

        let dragged_item_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default() + self.detection_state.offset().unwrap_or_default();
        let dragged_item_rect = Rect::from_min_size(dragged_item_pos, self.detection_state.dragged_item_size().unwrap_or_default());
        let dragged_item_center = dragged_item_rect.center();
        let mut above_item = None;
        let mut below_item = None;

        let mut should_add_space_at_end = true;

        let mut source_idx = None;
        let mut dragged_item_size = None;
        let mut add_space_for_previous_item = false;

        ui.scope(|ui| {
            let item_spacing = ui.spacing().item_spacing.y;
            ui.spacing_mut().item_spacing.y = 0.0;

            DragDropUi::drop_target(ui, true, |ui| {
                vec.into_iter().for_each(|(idx, mut item)| {
                    let item_id = item.borrow().id();
                    let dragging = self.detection_state.is_dragging_item(item_id);

                    let hovering_this_item = self.detection_state.hovering_index() == Some(idx);
                    let mut add_space = hovering_this_item;
                    if add_space_for_previous_item {
                        add_space = true;
                        add_space_for_previous_item = false;
                    }
                    if hovering_this_item && self.detection_state.is_dragging_item(item_id) {
                        add_space = false;
                        add_space_for_previous_item = true;
                        println!("add space for previous item, idx: {}", idx);
                    }
                    if add_space {
                        should_add_space_at_end = false;
                    }
                    let x = ui.ctx().animate_bool(ui.auto_id_with(item_id.with(self.drag_count).with(self.detection_state.hovering_index().is_some())), add_space);
                    if x > 0.0 {
                        let space = (dragged_item_rect.height() + item_spacing);
                        ui.add_space(space * x);
                    }

                    if !self.detection_state.is_dragging_item(item_id) {
                        ui.add_space(item_spacing);
                    }

                    let rect = self.drag_source(ui, item.borrow_mut().id(), |ui, handle| {
                        item_ui(item.borrow_mut(), ui, handle, dragging);
                    });

                    if dragged_item_center.y < rect.center().y && above_item.is_none() {
                        above_item = Some(idx);
                    }
                    if dragged_item_center.y > rect.center().y {
                        below_item = Some((idx, item_id));
                    }

                    if self.detection_state.is_dragging_item(item_id) {
                        source_idx = Some(idx);
                        dragged_item_size = Some(rect.size());
                    }
                });
            });

            let x = ui.ctx().animate_bool(ui.auto_id_with(self.drag_count).with("end_space").with(self.detection_state.hovering_index().is_some()), should_add_space_at_end && self.detection_state.hovering_index().is_some());
            if x > 0.0 {
                let mut spacing = item_spacing;
                if add_space_for_previous_item {
                    spacing = item_spacing;
                }
                let space = (dragged_item_rect.height() + item_spacing);
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.allocate_exact_size(Vec2::new(0.0, space * x), Sense::hover());
            }
        });


        if let DragDetectionState::Dragging { phase, id: dragging_id, .. } = &mut self.detection_state {
            let hovering_idx = above_item
                .or(below_item.map(|(i, id)| if id == *dragging_id { i } else { i + 1 }));
            if let Some(hovering_idx) = hovering_idx {
                if let Some(source_idx) = source_idx {
                    if let Some(dragged_item_size) = dragged_item_size {
                        *phase = DragPhase::Rest {
                            hovering_idx,
                            source_idx,
                            dragged_item_size,
                        }
                    }
                }
            }
        }

        let response = if let DragDetectionState::Dragging {
            phase: DragPhase::Rest {
                source_idx,
                hovering_idx,
                ..
            }, ..
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

        ui.label(format!("Above: {:?}", above_item));
        ui.label(format!("Below: {:?}", below_item));

        response
    }

    fn drag_source(
        &mut self,
        ui: &mut Ui,
        id: Id,
        drag_body: impl FnOnce(&mut Ui, Handle),
    ) -> Rect {
        if let DragDetectionState::Dragging { id: dragging_id, offset, phase, .. } = &mut self.detection_state {
            // Draw the item item in it's original position in the first frame to avoid flickering
            if id == *dragging_id && !phase.is_first_frame() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);

                let _layer_id = LayerId::new(Order::Tooltip, id);

                let pointer_pos = ui
                    .ctx()
                    .pointer_hover_pos()
                    .unwrap_or_else(|| ui.next_widget_position());

                // If we are in a ScrollArea, allow for scrolling while dragging
                ui.scroll_to_rect(Rect::from_center_size(pointer_pos, Vec2::splat(100.0)), None);

                let InnerResponse { inner: rect, .. } = egui::Area::new("draggable_item")
                    .interactable(false)
                    .fixed_pos(pointer_pos + *offset)
                    .show(ui.ctx(), |ui| {
                        ui.scope(|ui| {
                            drag_body(ui, Handle { state: self, id })
                        })
                            .response
                            .rect
                    });

                return Rect::from_min_size(ui.next_widget_position(), rect.size());
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
