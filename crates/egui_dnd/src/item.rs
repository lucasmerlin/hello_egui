use egui::{CursorIcon, Id, InnerResponse, LayerId, Layout, Order, Pos2, Rect, Sense, Ui, Vec2};
use egui_animation::animate_position;

use crate::state::DragDetectionState;
use crate::{DragDropUi, Handle, ItemState};

pub struct Item<'a> {
    id: Id,
    pub state: ItemState,
    dnd_state: &'a mut DragDropUi,
    hovering_over_any_handle: &'a mut bool,
    easing: fn(f32) -> f32,
}

impl<'a> Item<'a> {
    pub fn new(
        id: Id,
        state: ItemState,
        dnd_state: &'a mut DragDropUi,
        hovering_over_any_handle: &'a mut bool,
    ) -> Self {
        Self {
            id,
            state,
            dnd_state,
            hovering_over_any_handle,
            easing: simple_easing::linear,
        }
    }

    pub fn ui(
        self,
        ui: &mut Ui,
        add_content: impl FnOnce(&mut Ui, Handle, ItemState),
    ) -> ItemResponse {
        self.drag_source(None, ui, add_content)
    }

    pub fn ui_sized(
        self,
        ui: &mut Ui,
        size: Vec2,
        add_content: impl FnOnce(&mut Ui, Handle, ItemState),
    ) -> ItemResponse {
        self.drag_source(Some(size), ui, add_content)
    }

    fn drag_source(
        self,
        size: Option<Vec2>,
        ui: &mut Ui,
        drag_body: impl FnOnce(&mut Ui, Handle, ItemState),
    ) -> ItemResponse {
        let hovering_over_any_handle = self.hovering_over_any_handle;
        let id = self.id;
        let index = self.state.index;
        let last_pointer_pos = self.dnd_state.detection_state.last_pointer_pos();
        if let DragDetectionState::Dragging {
            id: dragging_id,
            offset,
            ..
        } = &mut self.dnd_state.detection_state
        {
            // Draw the item item in it's original position in the first frame to avoid flickering
            if id == *dragging_id {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);

                let _layer_id = LayerId::new(Order::Tooltip, id);

                let pointer_pos = ui
                    .ctx()
                    .pointer_hover_pos()
                    .or(last_pointer_pos)
                    .unwrap_or_else(|| ui.next_widget_position());
                let position = pointer_pos + *offset;

                // We animate so the animated position is updated, even though we don't use it here.
                animate_position(
                    ui,
                    id,
                    position,
                    self.dnd_state.swap_animation_time,
                    self.easing,
                    false,
                );

                let InnerResponse { inner: rect, .. } = Self::draw_floating_at_position(
                    self.state,
                    self.dnd_state,
                    ui,
                    id,
                    position,
                    hovering_over_any_handle,
                    size,
                    *ui.layout(),
                    drag_body,
                );

                ui.allocate_space(rect.size());

                let rect = Rect::from_min_size(ui.next_widget_position(), rect.size());
                return ItemResponse(rect);
            }
        } else if let DragDetectionState::TransitioningBackAfterDragFinished {
            id: transitioning_id,
            dragged_item_size: _,
        } = &mut self.dnd_state.detection_state
        {
            if id == *transitioning_id {
                let (end_pos, did_allocate_size) = if let Some(size) = size {
                    let (_, rect) = ui.allocate_space(size);
                    (rect.min, Some(rect))
                } else {
                    (ui.next_widget_position(), None)
                };

                let position = animate_position(
                    ui,
                    id,
                    end_pos,
                    self.dnd_state.return_animation_time,
                    self.easing,
                    false,
                );

                let InnerResponse { inner: rect, .. } = Self::draw_floating_at_position(
                    self.state,
                    self.dnd_state,
                    ui,
                    id,
                    position,
                    hovering_over_any_handle,
                    size,
                    *ui.layout(),
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

                return ItemResponse(rect);
            }
        }

        let was_dragging = self.dnd_state.detection_state.is_dragging();

        let rect = if let Some(size) = size {
            // We need to do it like this because in some layouts
            // ui.next_widget_position() will return the vertical center instead
            // of the top left corner
            let (_, rect) = ui.allocate_space(size);

            let animated_position = animate_position(
                ui,
                id,
                rect.min,
                self.dnd_state.swap_animation_time,
                self.easing,
                true,
            );

            let position = if self.dnd_state.detection_state.is_dragging() {
                animated_position
            } else {
                rect.min
            };

            let mut child = ui.child_ui(rect, *ui.layout(), None);

            child.allocate_ui_at_rect(Rect::from_min_size(position, rect.size()), |ui| {
                drag_body(
                    ui,
                    Handle::new(
                        id,
                        index,
                        self.dnd_state,
                        hovering_over_any_handle,
                        rect.min,
                    ),
                    self.state,
                )
            });

            rect
        } else {
            let position = ui.cursor().min;

            let animated_position = animate_position(
                ui,
                id,
                position,
                self.dnd_state.swap_animation_time,
                self.easing,
                true,
            );

            let position = if self.dnd_state.detection_state.is_dragging() {
                animated_position
            } else {
                position
            };

            let size = ui.available_size();

            let mut child = ui.child_ui(ui.max_rect(), *ui.layout(), None);
            let response = child.allocate_ui_at_rect(Rect::from_min_size(position, size), |ui| {
                drag_body(
                    ui,
                    Handle::new(
                        id,
                        index,
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

        ItemResponse(rect)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_floating_at_position(
        state: ItemState,
        dnd_state: &mut DragDropUi,
        ui: &mut Ui,
        id: Id,
        pos: Pos2,
        hovering_over_any_handle: &mut bool,
        size: Option<Vec2>,
        layout: Layout,
        body: impl FnOnce(&mut Ui, Handle, ItemState),
    ) -> InnerResponse<Rect> {
        egui::Area::new(Id::new("draggable_item"))
            .interactable(false)
            .fixed_pos(pos)
            .order(Order::Tooltip)
            .show(ui.ctx(), |ui| {
                ui.with_layout(layout, |ui| {
                    if let Some(size) = size.or(dnd_state.detection_state.dragged_item_size()) {
                        ui.set_max_size(size);
                    }
                    body(
                        ui,
                        Handle::new(id, state.index, dnd_state, hovering_over_any_handle, pos),
                        state,
                    )
                })
                .response
                .rect
            })
    }
}

pub struct ItemResponse(pub(crate) Rect);
