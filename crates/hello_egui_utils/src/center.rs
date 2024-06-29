use egui::{Align2, Id, Rect, Ui, Vec2};

/// A widget that measures its content and centers it within the available space.
pub struct Center {
    id: Id,
    rect: Option<Rect>,
    size: Option<Vec2>,
    align2: Align2,
}

impl Center {
    /// Create a new center widget with the given id.
    pub fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            rect: None,
            size: None,
            align2: Align2::CENTER_CENTER,
        }
    }

    /// Set the alignment
    pub fn align2(mut self, align2: Align2) -> Self {
        self.align2 = align2;
        self
    }

    /// Show the widget
    pub fn ui<T>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> T) -> T {
        let id = ui.id().with(self.id);
        let data_id = id.with("center");

        let rect = if let Some(rect) = self.rect {
            rect
        } else if let Some(size) = self.size {
            let pos = ui.next_widget_position();
            Rect::from_min_size(pos, size)
        } else {
            ui.available_rect_before_wrap()
        };

        let last_size = ui.ctx().data(|mem| mem.get_temp(data_id));

        let content_rect = if let Some(size) = last_size {
            let left_top = self.align2.align_size_within_rect(size, rect).left_top();
            Rect::from_min_size(left_top, rect.size())
        } else {
            rect
        };

        let mut ui = ui.child_ui(content_rect, *ui.layout());

        if last_size.is_none() {
            ui.set_visible(false);
        }

        let result = content(&mut ui);

        let size = ui.min_size();

        ui.ctx().data_mut(|mem| {
            mem.insert_temp(data_id, size);
        });

        result
    }
}
