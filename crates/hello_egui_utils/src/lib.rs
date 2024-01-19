#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use egui::{Align, Label, Layout, Ui, Vec2, WidgetText};

/// Returns the size of the text in the current ui (based on the max width of the ui)
pub fn measure_text(ui: &mut Ui, text: impl Into<WidgetText>) -> Vec2 {
    // There might be a more elegant way but this is enough for now
    let res = Label::new(text).layout_in_ui(&mut ui.child_ui(
        ui.available_rect_before_wrap(),
        Layout::left_to_right(Align::Center),
    ));

    // There seem to be rounding errors in egui's text rendering
    // so we add a little bit of padding
    res.2.rect.size() + Vec2::new(0.1, 0.0)
}

/// Returns the approximate current scroll delta of the ui
pub fn current_scroll_delta(ui: &Ui) -> Vec2 {
    -ui.min_rect().min.to_vec2()
}
