use crate::flex_widget::FlexWidget;
use crate::{FlexContainerResponse, FlexContainerUi};
use egui::{Frame, Label, Sense, Ui, Widget, WidgetInfo, WidgetText, WidgetType};

pub struct FlexButton {
    pub text: WidgetText,
    pub selected: bool,
    pub sense: Sense,
}

impl FlexButton {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            selected: false,
            sense: Sense::click(),
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }
}

impl FlexWidget for FlexButton {
    type Response = egui::Response;

    fn ui(self, ui: &mut Ui, container: FlexContainerUi) -> FlexContainerResponse<Self::Response> {
        let id = ui.next_auto_id();

        let response = ui.ctx().read_response(id);

        let style = if let Some(response) = &response {
            ui.style().interact_selectable(response, self.selected)
        } else {
            ui.style().visuals.widgets.inactive
        };

        let frame = Frame::none()
            .fill(style.bg_fill)
            .rounding(style.rounding)
            .stroke(style.bg_stroke)
            .inner_margin(ui.spacing().button_padding);

        let text = self.text.text().to_string();

        let res = frame.show(ui, |ui| {
            container.content(ui, |ui| {
                Label::new(self.text.color(style.fg_stroke.color))
                    .selectable(false)
                    .sense(Sense::hover())
                    .wrap()
                    .ui(ui);
            })
        });

        let response = ui.interact(res.response.rect, id, self.sense);

        response
            .widget_info(move || WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), &text));

        res.inner.map(|_| response)
    }
}
