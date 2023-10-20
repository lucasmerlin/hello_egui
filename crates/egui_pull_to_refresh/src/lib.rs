use egui::scroll_area::ScrollAreaOutput;
use egui::{Align2, Area, Color32, Id, Rect, Sense, Ui, Vec2};

use crate::progress_spinner::ProgressSpinner;

mod progress_spinner;

#[derive(Debug, Clone)]
pub enum PullToRefreshState {
    Idle,
    Dragging { distance: f32, far_enough: bool },
    DoRefresh,
    Refreshing,
    Done,
}

impl PullToRefreshState {
    fn progress(&self, min_distance: f32) -> Option<f64> {
        match self {
            PullToRefreshState::Idle => Some(0.0),
            PullToRefreshState::Dragging { distance, .. } => {
                Some((distance / min_distance).min(1.0).max(0.0) as f64)
            }
            PullToRefreshState::DoRefresh => Some(1.0),
            PullToRefreshState::Refreshing => None,
            PullToRefreshState::Done => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PullToRefreshResponse<T> {
    pub state: PullToRefreshState,
    pub inner: T,
}

impl<T> PullToRefreshResponse<T> {
    pub fn should_refresh(&self) -> bool {
        matches!(self.state, PullToRefreshState::DoRefresh)
    }
}

pub struct PullToRefresh {
    id: Id,
    loading: bool,
    min_refresh_distance: f32,
    can_refresh: bool,
}

impl PullToRefresh {
    /// Creates a new pull to refresh widget.
    /// If `loading` is true, the widget will show the loading indicator.
    pub fn new(loading: bool) -> Self {
        Self {
            id: Id::new("pull_to_refresh"),
            loading,
            min_refresh_distance: 100.0,
            can_refresh: true,
        }
    }

    /// Sets the minimum distance the user needs to drag to trigger a refresh.
    pub fn min_refresh_distance(mut self, min_refresh_distance: f32) -> Self {
        self.min_refresh_distance = min_refresh_distance;
        self
    }

    /// You need to provide a id if you use multiple pull to refresh widgets at once.
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// If `can_refresh` is false, pulling will not trigger a refresh.
    pub fn can_refresh(mut self, can_refresh: bool) -> Self {
        self.can_refresh = can_refresh;
        self
    }

    /// Shows the pull to refresh widget.
    /// Note: If you want to use the pull to refresh widget in a scroll area, use [`Self::scroll_area_ui`].
    pub fn ui<T>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> T,
    ) -> PullToRefreshResponse<T> {
        let mut child = ui.child_ui(ui.available_rect_before_wrap(), *ui.layout());

        let output = content(&mut child);

        let can_refresh = self.can_refresh;
        let state = self.internal_ui(ui, can_refresh, None, child.min_rect());

        PullToRefreshResponse {
            state,
            inner: output,
        }
    }

    pub fn scroll_area_ui<T>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> ScrollAreaOutput<T>,
    ) -> PullToRefreshResponse<ScrollAreaOutput<T>> {
        let scroll_output = content(ui);
        let content_rect = scroll_output.inner_rect;
        let can_refresh = scroll_output.state.offset.y == 0.0 && self.can_refresh;
        // This is the id used in the Sense of the scroll area
        // I hope this id is stable across egui patches...
        let allow_dragged_id = scroll_output.id.with("area");
        let state = self.internal_ui(ui, can_refresh, Some(allow_dragged_id), content_rect);
        PullToRefreshResponse {
            state,
            inner: scroll_output,
        }
    }

    fn internal_ui(
        self,
        ui: &mut Ui,
        can_refresh: bool,
        allow_dragged_id: Option<Id>,
        content_rect: Rect,
    ) -> PullToRefreshState {
        let last_state = ui.data_mut(|data| {
            data.get_temp_mut_or(self.id, PullToRefreshState::Idle)
                .clone()
        });

        let mut state = last_state;
        if self.loading {
            state = PullToRefreshState::Refreshing;
        }

        if !self.loading && matches!(state, PullToRefreshState::Refreshing) {
            state = PullToRefreshState::Done;
        }

        if can_refresh && !self.loading {
            let sense = ui.interact(content_rect, self.id, Sense::hover());

            let is_something_blocking_drag = ui.memory(|memory| {
                memory.is_anything_being_dragged()
                    && !allow_dragged_id.map_or(false, |id| memory.is_being_dragged(id))
            });

            if sense.hovered() && !is_something_blocking_drag {
                let (delta, any_released) = ui.input(|input| {
                    (
                        if input.pointer.is_decidedly_dragging() {
                            Some(input.pointer.delta())
                        } else {
                            None
                        },
                        input.pointer.any_released(),
                    )
                });
                if let Some(delta) = delta {
                    if matches!(state, PullToRefreshState::Idle) {
                        state = PullToRefreshState::Dragging {
                            distance: 0.0,
                            far_enough: false,
                        };
                    }
                    if let PullToRefreshState::Dragging { distance: drag, .. } = state.clone() {
                        let dist = drag + delta.y;
                        state = PullToRefreshState::Dragging {
                            distance: dist,
                            far_enough: dist > self.min_refresh_distance,
                        };
                    }
                } else {
                    state = PullToRefreshState::Idle;
                }
                if any_released {
                    if let PullToRefreshState::Dragging {
                        far_enough: enough, ..
                    } = state.clone()
                    {
                        if enough {
                            state = PullToRefreshState::DoRefresh;
                        } else {
                            state = PullToRefreshState::Idle;
                        }
                    } else {
                        state = PullToRefreshState::Idle;
                    }
                }
            } else {
                state = PullToRefreshState::Idle;
            }
        } else {
            state = PullToRefreshState::Idle;
        }

        if self.loading {
            state = PullToRefreshState::Refreshing;
        }

        let spinner_size = Vec2::splat(24.0);

        let progress_for_offset = match &state {
            PullToRefreshState::Idle => 0.0,
            PullToRefreshState::Dragging { .. } => {
                state.progress(self.min_refresh_distance).unwrap_or(1.0)
            }
            PullToRefreshState::DoRefresh => 1.0,
            PullToRefreshState::Refreshing => 1.0,
            PullToRefreshState::Done => 0.0,
        } as f32;

        let anim_progress = ui.ctx().animate_value_with_time(
            self.id.with("offset_top"),
            progress_for_offset,
            ui.style().animation_time,
        );

        let offset_top = -spinner_size.y + spinner_size.y * anim_progress * 2.0;

        if anim_progress > 0.0 {
            Area::new("Pull to refresh indicator")
                .fixed_pos(content_rect.center_top())
                .pivot(Align2::CENTER_TOP)
                .show(ui.ctx(), |ui| {
                    let (rect, _) = ui.allocate_exact_size(spinner_size, Sense::hover());

                    ui.set_clip_rect(Rect::everything_below(rect.min.y));

                    let rect = rect.translate(Vec2::new(0.0, offset_top));

                    ui.painter().circle(
                        rect.center(),
                        spinner_size.x / 1.5,
                        ui.style().visuals.widgets.inactive.bg_fill,
                        ui.visuals().widgets.inactive.bg_stroke,
                    );

                    let mut spinner_color = ui.style().visuals.widgets.inactive.fg_stroke.color;
                    if anim_progress < 1.0 {
                        spinner_color = Color32::from_rgba_unmultiplied(
                            spinner_color.r(),
                            spinner_color.g(),
                            spinner_color.b(),
                            (spinner_color.a() as f32 * 0.7).round() as u8,
                        );
                    }
                    ProgressSpinner::new()
                        .color(spinner_color)
                        .progress(state.progress(self.min_refresh_distance))
                        .paint_at(ui, rect);
                });
        }

        ui.data_mut(|data| {
            data.insert_temp(self.id, state.clone());
        });

        state
    }
}
