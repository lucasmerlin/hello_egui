use crate::state::DragDetectionState;
use crate::utils::animate_position;
use crate::{DragDropUi, Handle, ItemState};
use egui::{CursorIcon, Id, InnerResponse, LayerId, Order, Pos2, Rect, Sense, Ui, Vec2};

pub struct Item<'a, T> {
    id: Id,
    pub item: T,
    pub state: ItemState,
    dnd_state: &'a mut DragDropUi,
    hovering_over_any_handle: &'a mut bool,
}

impl<'a, T> Item<'a, T> {
    pub fn new(
        id: Id,
        item: T,
        state: ItemState,
        dnd_state: &'a mut DragDropUi,
        hovering_over_any_handle: &'a mut bool,
    ) -> Self {
        Self {
            id,
            item,
            state,
            dnd_state,
            hovering_over_any_handle,
        }
    }

    pub fn ui(
        mut self,
        ui: &mut Ui,
        add_content: impl FnMut(&mut Ui, T, Handle, ItemState),
    ) -> ItemResponse {
        self.drag_source(None, ui, add_content)
    }

    pub fn ui_sized(
        mut self,
        ui: &mut Ui,
        size: Vec2,
        add_content: impl FnMut(&mut Ui, T, Handle, ItemState),
    ) -> ItemResponse {
        self.drag_source(Some(size), ui, add_content)
    }

    fn drag_source(
        self,
        size: Option<Vec2>,
        ui: &mut Ui,
        drag_body: impl FnOnce(&mut Ui, T, Handle, ItemState),
    ) -> ItemResponse {
        let hovering_over_any_handle = self.hovering_over_any_handle;
        let id = self.id;
        let index = self.state.index;
        let last_pointer_pos = self.dnd_state.detection_state.last_pointer_pos();
        if let DragDetectionState::Dragging {
            id: dragging_id,
            offset,
            phase,
            ..
        } = &mut self.dnd_state.detection_state
        {
            // Draw the item item in it's original position in the first frame to avoid flickering
            if id == *dragging_id && !phase.is_first_frame() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);

                let _layer_id = LayerId::new(Order::Tooltip, id);

                let pointer_pos = ui
                    .ctx()
                    .pointer_hover_pos()
                    .or(last_pointer_pos)
                    .unwrap_or_else(|| ui.next_widget_position());
                let position = pointer_pos + *offset;

                // We animate so the animated position is updated, even though we don't use it here.
                animate_position(ui, id, position);

                // If we are in a ScrollArea, allow for scrolling while dragging
                ui.scroll_to_rect(Rect::from_center_size(pointer_pos, Vec2::splat(50.0)), None);

                let InnerResponse { inner: rect, .. } = Self::draw_floating_at_position(
                    self.item,
                    self.state,
                    self.dnd_state,
                    ui,
                    id,
                    position,
                    hovering_over_any_handle,
                    size,
                    drag_body,
                );

                ui.allocate_space(rect.size());

                let response = Rect::from_min_size(ui.next_widget_position(), rect.size());
                return ItemResponse {
                    rect: response,
                    idx: index,
                    id,
                };
            }
        } else if let DragDetectionState::TransitioningBackAfterDragFinished {
            from,
            id: transitioning_id,
            dragged_item_size,
        } = &mut self.dnd_state.detection_state
        {
            if id == *transitioning_id {
                let (end_pos, did_allocate_size) = if let Some(size) = size {
                    let (_, rect) = ui.allocate_space(size);
                    (rect.min, Some(rect))
                } else {
                    (ui.next_widget_position(), None)
                };

                // This ensures that the first frame of the animation gets the correct position
                let value = std::mem::take(from).unwrap_or(end_pos);

                let position = animate_position(ui, id, value);

                let InnerResponse { inner: rect, .. } = Self::draw_floating_at_position(
                    self.item,
                    self.state,
                    self.dnd_state,
                    ui,
                    id,
                    position,
                    hovering_over_any_handle,
                    size,
                    drag_body,
                );

                let rect = if let Some(rect) = did_allocate_size {
                    rect
                } else {
                    ui.allocate_exact_size(rect.size(), Sense::hover()).0
                };

                if position == end_pos {
                    // Animation finished
                    self.dnd_state.detection_state = DragDetectionState::None;
                }

                return ItemResponse {
                    rect,
                    idx: index,
                    id,
                };
            }
        }

        let was_dragging = self.dnd_state.detection_state.is_dragging();

        let rect = if let Some(size) = size {
            // We need to do it like this because in some layouts
            // ui.next_widget_position() will return the vertical center instead
            // of the top left corner
            let (_, rect) = ui.allocate_space(size);

            let animated_position = animate_position(ui, id, rect.min);

            let position = if self.dnd_state.detection_state.is_dragging() {
                animated_position
            } else {
                rect.min
            };

            let mut child = ui.child_ui(rect, *ui.layout());

            child.allocate_ui_at_rect(Rect::from_min_size(position, rect.size()), |ui| {
                drag_body(
                    ui,
                    self.item,
                    Handle::new(id, self.dnd_state, hovering_over_any_handle, rect.min),
                    self.state,
                )
            });

            rect
        } else {
            let position = ui.next_widget_position();
            let animated_position = animate_position(ui, id, position);

            let position = if self.dnd_state.detection_state.is_dragging() {
                animated_position
            } else {
                position
            };

            let size = ui.available_size();

            let mut child = ui.child_ui(ui.max_rect(), *ui.layout());
            let response = child.allocate_ui_at_rect(Rect::from_min_size(position, size), |ui| {
                drag_body(
                    ui,
                    self.item,
                    Handle::new(
                        id,
                        self.dnd_state,
                        hovering_over_any_handle,
                        animated_position,
                    ),
                    self.state,
                )
            });

            ui.allocate_space(response.response.rect.size()).1
        };

        if !was_dragging && self.dnd_state.detection_state.is_dragging() {
            if let DragDetectionState::Dragging {
                dragged_item_size, ..
            } = &mut self.dnd_state.detection_state
            {
                // We set this here because we don't know the size in the handle
                *dragged_item_size = rect.size();
            }
        }

        ItemResponse {
            rect,
            idx: index,
            id,
        }
    }

    fn draw_floating_at_position(
        item: T,
        state: ItemState,
        dnd_state: &mut DragDropUi,
        ui: &mut Ui,
        id: Id,
        pos: Pos2,
        hovering_over_any_handle: &mut bool,
        size: Option<Vec2>,
        body: impl FnOnce(&mut Ui, T, Handle, ItemState),
    ) -> InnerResponse<Rect> {
        let _layer_id = LayerId::new(Order::Tooltip, id);

        egui::Area::new("draggable_item")
            .interactable(false)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                ui.scope(|ui| {
                    if let Some(size) = size.or(dnd_state.detection_state.dragged_item_size()) {
                        ui.set_max_size(size);
                    }
                    body(
                        ui,
                        item,
                        Handle::new(id, dnd_state, hovering_over_any_handle, pos),
                        state,
                    )
                })
                .response
                .rect
            })
    }
}

pub struct ItemResponse {
    pub(crate) rect: Rect,
    id: Id,
    idx: usize,
}
