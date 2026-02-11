use crate::handler::HandlerError;
use crate::Route;
use egui::Ui;
use egui_suspense::EguiSuspense;
#[cfg(not(feature = "subsecond"))]
use crate::SubsecondMockRoute;

pub(crate) struct AsyncRoute<State> {
    pub suspense: EguiSuspense<Box<dyn Route<State> + Send + Sync>, HandlerError>,
}

impl<State: 'static> AsyncRoute<State> {
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        self.suspense.ui(ui, |ui, data, _state| {
            data.ui_subsecond(ui, state);
        });
    }
}

impl<State: 'static> Route<State> for AsyncRoute<State> {
    fn ui(&mut self, ui: &mut Ui, state: &mut State) {
        self.ui(ui, state);
    }

    fn on_showing(&mut self) {
        if let Some(route) = self.suspense.data_mut() {
            route.on_showing();
        }
    }

    fn on_shown(&mut self) {
        if let Some(route) = self.suspense.data_mut() {
            route.on_shown();
        }
    }

    fn on_hiding(&mut self) {
        if let Some(route) = self.suspense.data_mut() {
            route.on_hiding();
        }
    }

    fn on_hide(&mut self) {
        if let Some(route) = self.suspense.data_mut() {
            route.on_hide();
        }
    }

    fn enable_swipe(&self) -> Option<bool> {
        self.suspense.data().and_then(|route| route.enable_swipe())
    }
}
