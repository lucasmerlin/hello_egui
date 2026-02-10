/// Example demonstrating the Route lifecycle hooks:
/// `on_showing`, `on_shown`, `on_hiding`, `on_hide`.
///
/// Each route tracks its visibility state and displays lifecycle event history.
use eframe::NativeOptions;
use egui::{CentralPanel, Color32, Frame, Ui};
use egui_inbox::UiInbox;
use egui_router::{EguiRouter, Route, TransitionConfig};

struct AppState {
    inbox: UiInbox<RouterMessage>,
}

enum RouterMessage {
    Navigate(String),
    Back,
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let mut router: Option<(EguiRouter<AppState>, AppState)> = None;

    eframe::run_simple_native(
        "Route Lifecycle Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            let (router, state) = router.get_or_insert_with(|| {
                let mut state = AppState {
                    inbox: UiInbox::new(),
                };

                let router = EguiRouter::builder()
                    .transition(TransitionConfig::slide().with_duration(0.5))
                    .route("/", || HomePage::new())
                    .route("/detail", || DetailPage::new())
                    .default_path("/")
                    .build(&mut state);

                (router, state)
            });

            state.inbox.read(ctx).for_each(|msg| match msg {
                RouterMessage::Navigate(route) => {
                    router.navigate(state, route).ok();
                }
                RouterMessage::Back => {
                    router.back(state).ok();
                }
            });

            CentralPanel::default().show(ctx, |ui| {
                router.ui(ui, state);
            });
        },
    )
}

/// A page that tracks and displays its lifecycle events.
struct HomePage {
    events: Vec<&'static str>,
}

impl HomePage {
    fn new() -> Self {
        Self {
            events: vec!["created"],
        }
    }
}

impl Route<AppState> for HomePage {
    fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        background(ui, ui.style().visuals.faint_bg_color, |ui| {
            ui.heading("Home Page");

            if ui.link("Go to Detail →").clicked() {
                state
                    .inbox
                    .sender()
                    .send(RouterMessage::Navigate("/detail".to_string()))
                    .ok();
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            ui.strong("Lifecycle events:");
            for event in &self.events {
                ui.label(format!("• {event}"));
            }
        });
    }

    fn on_showing(&mut self, _state: &mut AppState) {
        self.events.push("on_showing (transition back started)");
    }

    fn on_shown(&mut self, _state: &mut AppState) {
        self.events.push("on_shown (fully visible)");
    }

    fn on_hiding(&mut self, _state: &mut AppState) {
        self.events.push("on_hiding (transition away started)");
    }

    fn on_hide(&mut self, _state: &mut AppState) {
        self.events.push("on_hide (fully hidden)");
    }
}

/// A detail page that also tracks lifecycle events.
struct DetailPage {
    events: Vec<&'static str>,
}

impl DetailPage {
    fn new() -> Self {
        Self {
            events: vec!["created"],
        }
    }
}

impl Route<AppState> for DetailPage {
    fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        background(ui, ui.style().visuals.extreme_bg_color, |ui| {
            ui.heading("Detail Page");

            if ui.button("← Back").clicked() {
                state
                    .inbox
                    .sender()
                    .send(RouterMessage::Back)
                    .ok();
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            ui.strong("Lifecycle events:");
            for event in &self.events {
                ui.label(format!("• {event}"));
            }
        });
    }

    fn on_showing(&mut self, _state: &mut AppState) {
        self.events.push("on_showing (transition back started)");
    }

    fn on_shown(&mut self, _state: &mut AppState) {
        self.events.push("on_shown (fully visible)");
    }

    fn on_hiding(&mut self, _state: &mut AppState) {
        self.events.push("on_hiding (transition away started)");
    }

    fn on_hide(&mut self, _state: &mut AppState) {
        self.events.push("on_hide (fully hidden)");
    }
}

fn background(ui: &mut Ui, color: Color32, content: impl FnOnce(&mut Ui)) {
    Frame::NONE.fill(color).inner_margin(16.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());
        content(ui);
    });
}
