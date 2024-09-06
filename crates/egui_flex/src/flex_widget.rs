use crate::{FlexContainerResponse, FlexContainerUi};
use egui::{Ui, Widget};

pub trait FlexWidget {
    type Response;
    fn ui(self, ui: &mut Ui, container: FlexContainerUi) -> FlexContainerResponse<Self::Response>;
}

impl<T: Widget> FlexWidget for T {
    type Response = egui::Response;

    fn ui(self, ui: &mut Ui, container: FlexContainerUi) -> FlexContainerResponse<Self::Response> {
        container.content_widget(ui, self)
    }
}
