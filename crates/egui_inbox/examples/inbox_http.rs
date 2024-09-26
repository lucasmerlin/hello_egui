use eframe::{egui, NativeOptions};
use egui::CentralPanel;
use egui_inbox::UiInbox;
use ehttp::Request;

struct MyComponent {
    inbox: UiInbox<String>,
    state: Option<String>,
}

impl MyComponent {
    pub fn new() -> Self {
        Self {
            inbox: UiInbox::new(),
            state: None,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Check if there is a new value, and if so, place it in our Option
        // We pass the ui to the inbox, so it can grab a reference to egui's context and
        // later call request_repaint() when sending a message
        self.inbox.replace_option(ui, &mut self.state);

        if ui.button("Http Request").clicked() {
            let tx = self.inbox.sender();

            ehttp::fetch(
                Request::get("http://worldtimeapi.org/api/ip"),
                move |result| {
                    let time = match result {
                        Ok(response) => {
                            let json: serde_json::Value = response.json().unwrap();
                            json["datetime"].as_str().unwrap().to_string()
                        }
                        Err(err) => format!("Error: {err:?}"),
                    };

                    // Queues the message in the inbox and calls request_repaint so the ui will be updated immediately
                    tx.send(time).ok();
                },
            );
        }

        if let Some(time) = &self.state {
            ui.strong(format!("Time: {time:?}"));
        } else {
            ui.label("No time yet");
        }
    }
}

pub fn main() -> eframe::Result<()> {
    let mut my_component = MyComponent::new();

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                my_component.ui(ui);
            });
        },
    )
}
