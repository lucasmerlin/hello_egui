/// This example demonstrates a very simple multi-page application with a simple "router" and a login dialog,
/// utilizing the `TypeInbox` as a message passing system between independent components.
///
/// The `TypeInbox` can be utilized as a application wide message bus, where components can send messages
/// to each other, without the `AppState` even having to know about any of the message types.
///
/// The individual components could be in separate modules or even separate crates, with the messages and
/// events being defined in a shared crate.
use std::sync::Arc;

use derive_new::new;
use eframe::{egui, NativeOptions};
use egui::{vec2, Area, CentralPanel, Context, Id, Ui, Window};
use parking_lot::Mutex;

use egui_inbox::broadcast::BroadcastReceiver;
use egui_inbox::type_broadcast::TypeBroadcast;
use egui_inbox::type_inbox::TypeInbox;
use egui_inbox::UiInbox;

use crate::dashboard::DashboardUi;
use crate::home::HomeUi;

#[derive(Clone, Debug)]
enum AuthMessage {
    ShowLoginDialog {
        message: String,
        navigate_to_when_finished: Option<RouterMessage>,
    },
    Logout,
}

#[derive(Clone, Copy, Debug)]
enum RouterMessage {
    Home,
    Dashboard,
    ForgotPassword,
}

#[derive(Clone, Debug)]
enum AuthEvent {
    LoggedIn { user: String },
    LoggedOut,
}

#[derive(Clone)]
struct AppState {
    inbox: TypeInbox,
    broadcast: TypeBroadcast,
    auth: Arc<Mutex<Option<String>>>,
}

// The components could be in separate modules or even crates
mod home {
    use derive_new::new;

    use crate::{AppState, AuthMessage, RouterMessage, Ui};

    #[derive(new)]
    pub struct HomeUi {
        app_state: AppState,
    }

    impl HomeUi {
        pub fn ui(&mut self, ui: &mut Ui) {
            ui.heading("Home");

            if ui.button("Go to dashboard").clicked() {
                let auth = self.app_state.auth.lock();
                if auth.is_some() {
                    self.app_state.inbox.send(RouterMessage::Dashboard);
                } else {
                    self.app_state.inbox.send(AuthMessage::ShowLoginDialog {
                        message: "You need to log in to see your personal dashboard".to_string(),
                        navigate_to_when_finished: Some(RouterMessage::Dashboard),
                    });
                }
            }
        }
    }
}

mod dashboard {
    use derive_new::new;

    use crate::{
        AppState, AuthEvent, AuthMessage, BroadcastReceiver, Context, RouterMessage, Ui, UiInbox,
    };

    #[derive(new)]
    pub struct DashboardUi {
        #[new(default)]
        lucky_number: Option<u32>,
        #[new(default)]
        inbox: UiInbox<u32>,
        #[new(value = "app_state.broadcast.subscribe::<AuthEvent>()")]
        auth_inbox: BroadcastReceiver<AuthEvent>,
        app_state: AppState,
    }

    impl DashboardUi {
        pub fn update(&mut self, ctx: &Context, active: bool) {
            self.inbox.replace_option(ctx, &mut self.lucky_number);
            self.auth_inbox.read(ctx).for_each(|event| match event {
                AuthEvent::LoggedIn { .. } => {
                    let tx = self.inbox.sender();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        let lucky_number = rand::random::<u32>() % 100;
                        tx.send(lucky_number).ok();
                    });
                }
                AuthEvent::LoggedOut => {
                    self.lucky_number = None;
                    if active {
                        self.app_state.inbox.send(RouterMessage::Home);
                    }
                }
            });
        }

        pub fn ui(&mut self, ui: &mut Ui) {
            ui.heading("Dashboard");

            let user = self.app_state.auth.lock();

            if let Some(user) = user.as_ref() {
                ui.label(format!("Welcome, {user}!"));

                if let Some(lucky_number) = self.lucky_number {
                    ui.label(format!("Your lucky number is: {lucky_number}"));
                } else {
                    ui.label("Our magicians are still calculating your lucky number...");
                    ui.spinner();
                }

                ui.label("Imagine some fancy graphs and data here 📊");
            } else {
                ui.label("You need to log in to see your personal dashboard");
            }

            if ui.button("Logout").clicked() {
                self.app_state.inbox.send(AuthMessage::Logout);
            }
        }
    }
}

mod auth {
    use derive_new::new;

    use crate::{egui, AppState, AuthEvent, AuthMessage, Context, RouterMessage, Window};

    #[derive(new)]
    pub struct AuthDialog {
        pub(crate) app_state: AppState,
        #[new(default)]
        username_input: String,
        #[new(default)]
        open_with_reason: Option<(String, Option<RouterMessage>)>,
    }

    impl AuthDialog {
        pub fn dialog_ui(&mut self, ctx: &Context) {
            self.app_state.inbox.read().for_each(|msg| match msg {
                AuthMessage::ShowLoginDialog {
                    message,
                    navigate_to_when_finished,
                } => {
                    self.open_with_reason = Some((message, navigate_to_when_finished));
                }
                AuthMessage::Logout => {
                    self.app_state.broadcast.send(AuthEvent::LoggedOut);
                }
            });

            let mut open = true;

            if let Some((reason, navigate_to_when_finished)) = self.open_with_reason.clone() {
                Window::new("Login")
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
                    .open(&mut open)
                    .show(ctx, |ui| {
                        ui.label(reason);
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut self.username_input);
                        if ui.button("Login").clicked() {
                            self.app_state.broadcast.send(AuthEvent::LoggedIn {
                                user: self.username_input.clone(),
                            });
                            if let Some(msg) = navigate_to_when_finished {
                                self.app_state.inbox.send(msg);
                            }
                            self.open_with_reason = None;
                        }
                        if ui.button("Forgot password").clicked() {
                            self.app_state.inbox.send(RouterMessage::ForgotPassword);
                            self.open_with_reason = None;
                        }
                    });
            }

            if !open {
                self.open_with_reason = None;
            }
        }
    }
}

#[derive(new)]
struct Router {
    app_state: AppState,
    #[new(value = "RouterMessage::Home")]
    page: RouterMessage,
    home: HomeUi,
    dashboard: DashboardUi,
}

impl Router {
    pub fn ui(&mut self, ui: &mut Ui) {
        // Read the router's inbox to see if we should open any new pages
        self.app_state
            .inbox
            .read()
            .for_each(|msg: RouterMessage| self.page = msg);

        // If we read a component's inbox only when it's rendered, it could cause a memory leak
        // so it's better to have a update function on our components, if they are not always rendered
        // Although for things as rare as login events it would be fine to ignore this
        self.dashboard
            .update(ui.ctx(), matches!(self.page, RouterMessage::Dashboard));

        match self.page {
            RouterMessage::Home => self.home.ui(ui),
            RouterMessage::Dashboard => self.dashboard.ui(ui),
            RouterMessage::ForgotPassword => {
                ui.heading("Forgot password");

                ui.label("Imagine some password recovery form here ✨");

                if ui.button("Go back").clicked() {
                    self.app_state.inbox.send(RouterMessage::Home);
                }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let mut state = None;

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            let (state, auth, router, auth_rx) = state.get_or_insert_with(|| {
                let state = AppState {
                    inbox: TypeInbox::new(ctx),
                    auth: Arc::new(Mutex::new(None)),
                    broadcast: TypeBroadcast::new(),
                };
                let auth = auth::AuthDialog::new(state.clone());
                let home = HomeUi::new(state.clone());
                let dashboard = DashboardUi::new(state.clone());

                let router = Router::new(state.clone(), home, dashboard);

                let auth_rx = state.broadcast.subscribe::<AuthEvent>();

                (state, auth, router, auth_rx)
            });

            // Update our global auth state, based on the auth events
            auth_rx.read(ctx).for_each(|event| match event {
                AuthEvent::LoggedIn { user } => {
                    *state.auth.lock() = Some(user);
                }
                AuthEvent::LoggedOut => {
                    *state.auth.lock() = None;
                }
            });

            // Show the login dialog (Since it's a popup window, it is not part of the router)
            auth.dialog_ui(ctx);

            CentralPanel::default().show(ctx, |ui| {
                Area::new(Id::new("Centered"))
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
                    .show(ui.ctx(), |ui| {
                        ui.group(|ui| {
                            ui.set_min_size(vec2(400.0, 300.0));
                            router.ui(ui);
                        });
                    });
            });
        },
    )
}
