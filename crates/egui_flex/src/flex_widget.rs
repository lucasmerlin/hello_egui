use crate::{FlexContainerResponse, FlexContainerUi};
use egui::Ui;

pub trait FlexWidget {
    type Response;
    fn ui(self, ui: &mut Ui, container: FlexContainerUi) -> FlexContainerResponse<Self::Response>;
}
