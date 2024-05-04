use eframe::NativeOptions;
use std::mem;

#[derive(Debug, Clone)]
enum AuthBroadcastMessage {
    LoggedIn { user: String },
    LoggedOut,
}

#[derive(Debug, Clone)]
struct AppState {
    auth_broadcast: egui_inbox::broadcast::Broadcast<AuthBroadcastMessage>,
}

struct AuthUi {
    app_state: AppState,
    logged_in_as: Option<String>,

    username_input: String,
}

impl AuthUi {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut logout = false;
        if let Some(user) = &self.logged_in_as {
            ui.label(format!("Logged in as: {}", user));
            if ui.button("Log out").clicked() {
                self.app_state
                    .auth_broadcast
                    .send(AuthBroadcastMessage::LoggedOut);
                logout = true;
            }
        } else {
            ui.label("Not logged in");

            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username_input);

            if ui.button("Log in").clicked() {
                self.app_state
                    .auth_broadcast
                    .send(AuthBroadcastMessage::LoggedIn {
                        user: self.username_input.clone(),
                    });

                self.logged_in_as = Some(mem::take(&mut self.username_input));
                self.username_input.clear();
            }
        }

        if logout {
            self.logged_in_as = None;
        }
    }
}

struct UserRandomNumberUi {
    user: Option<String>,
    random_number: Option<u32>,
    auth_rx: egui_inbox::broadcast::BroadcastReceiver<AuthBroadcastMessage>,
}

impl UserRandomNumberUi {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.auth_rx.read(ui).for_each(|event| match event {
            AuthBroadcastMessage::LoggedIn { user } => {
                self.user = Some(user);
                self.random_number = Some(rand::random());
            }
            AuthBroadcastMessage::LoggedOut => {
                self.user = None;
                self.random_number = None;
            }
        });

        if let Some((user, number)) = self.user.as_ref().zip(self.random_number.as_ref()) {
            ui.label(format!("{user}'s random number: {number}"));
        } else {
            ui.label("Not logged in");
        }
    }
}

impl AppState {
    fn new() -> Self {
        Self {
            auth_broadcast: egui_inbox::broadcast::Broadcast::new(),
        }
    }
}

#[cfg(feature = "broadcast")]
fn main() {
    let state = AppState::new();

    let mut auth_ui = AuthUi {
        app_state: state.clone(),
        logged_in_as: None,
        username_input: String::new(),
    };

    let mut user_random_number_ui = UserRandomNumberUi {
        user: None,
        random_number: None,
        auth_rx: state.auth_broadcast.subscribe(),
    };

    eframe::run_simple_native(
        "Broadcast Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.heading("Auth");
                        auth_ui.ui(ui);
                    });

                    ui.group(|ui| {
                        ui.heading("User Random Number");
                        user_random_number_ui.ui(ui);
                    });
                });
            });
        },
    )
    .unwrap();
}

#[cfg(not(feature = "broadcast"))]
fn main() {
    panic!("This example requires the `broadcast` feature to be enabled.");
}
