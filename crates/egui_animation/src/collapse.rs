use crate::animate_bool_eased;
use egui::{Id, Rect, Ui, Vec2};

/// Collapse animation to hide/show content.
/// Currently only vertical collapse is supported.
pub struct Collapse {
    // TODO: Implement horizontal collapse
    #[allow(dead_code)]
    horizontal: bool,
    visible: bool,
    id: Id,
    duration: f32,
}

impl Collapse {
    // pub fn horizontal(id: impl Into<Id>, visible: bool) -> Self {
    //     Self {
    //         horizontal: true,
    //         visible,
    //         id: id.into(),
    //         duration: 0.2,
    //     }
    // }

    /// Creates a new vertical collapse animation.
    pub fn vertical(id: impl Into<Id>, visible: bool) -> Self {
        Self {
            horizontal: false,
            visible,
            id: id.into(),
            duration: 0.2,
        }
    }

    /// Show the content.
    pub fn ui(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let id = self.id;
        let visible = self.visible;

        let x = animate_bool_eased(
            ui.ctx(),
            id,
            visible,
            simple_easing::cubic_in_out,
            self.duration,
        );

        let last_size = ui
            .ctx()
            .memory_mut(|mem| *mem.data.get_temp_mut_or(id, 0.0));

        let mut child = ui.child_ui(
            Rect::from_min_size(ui.next_widget_position(), ui.available_size()),
            *ui.layout(),
            None,
        );

        let current_size = last_size * x;

        child.set_clip_rect(Rect::from_min_size(
            child.next_widget_position(),
            Vec2::new(child.available_size().x, current_size),
        ));

        content(&mut child);

        let size = child.min_size().min(ui.available_size());

        ui.memory_mut(|mem| {
            mem.data.insert_temp(id, size.y);
        });

        ui.allocate_rect(
            Rect::from_min_size(ui.next_widget_position(), Vec2::new(size.x, current_size)),
            egui::Sense::hover(),
        );
    }
}
