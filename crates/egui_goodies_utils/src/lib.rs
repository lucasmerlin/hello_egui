use egui::{Align, Label, Layout, Ui, Vec2, WidgetText};

pub fn measure_text(ui: &mut Ui, text: impl Into<WidgetText>) -> Vec2 {
    // There might be a more elegant way but this is enough for now
    let res = Label::new(text).layout_in_ui(&mut ui.child_ui(
        ui.available_rect_before_wrap(),
        Layout::left_to_right(Align::Center),
    ));
    // There seem to be rounding errors in egui's text rendering
    // so we add a little bit of padding
    let size = res.2.rect.size() + Vec2::new(0.1, 0.0);

    size
}
