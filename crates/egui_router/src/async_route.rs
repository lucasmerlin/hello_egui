use crate::handler::HandlerError;
use crate::Route;
use egui::Ui;
use egui_suspense::EguiSuspense;

pub struct AsyncRoute<State> {
    pub suspense: EguiSuspense<Box<dyn Route<State> + Send + Sync>, HandlerError>,
}

impl<State: 'static> AsyncRoute<State> {
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        self.suspense.ui(ui, |ui, data, _state| {
            data.ui(ui, state);
        });
    }
}

impl<State: 'static> Route<State> for AsyncRoute<State> {
    fn ui(&mut self, ui: &mut Ui, state: &mut State) {
        self.ui(ui, state);
    }
}
