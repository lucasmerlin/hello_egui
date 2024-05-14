use eframe::NativeOptions;
use egui::{CentralPanel, Context, Ui};
use egui_inbox::type_inbox::TypeInbox;
use egui_router::{EguiRouter, Request, Route};

struct AppState {
    message: String,
    inbox: TypeInbox,
}

enum RouterMessage {
    Navigate(String),
    Back,
}

fn main() -> eframe::Result<()> {
    let init = |ctx: &Context| {
        let mut router = EguiRouter::new(AppState {
            message: "Hello, World!".to_string(),
            inbox: TypeInbox::new(ctx.clone()),
        });

        router.route("/", home);
        router.route("/edit", edit_message);
        router.route("/post/{id}", post);

        router.navigate("/");

        router
    };

    let mut router: Option<EguiRouter<AppState>> = None;

    eframe::run_simple_native(
        "Router Example",
        NativeOptions::default(),
        move |ctx, frame| {
            let router = router.get_or_insert_with(|| init(ctx));

            router
                .state
                .inbox
                .read()
                .for_each(|msg: RouterMessage| match msg {
                    RouterMessage::Navigate(route) => {
                        router.navigate(route);
                    }
                    RouterMessage::Back => {
                        router.back();
                    }
                });

            CentralPanel::default().show(ctx, |ui| {
                router.ui(ui);
            });
        },
    )
}

fn home(request: &mut Request<AppState>) -> impl Route<AppState> {
    |ui: &mut Ui, state: &mut AppState| {
        ui.heading("Home!");

        ui.label(format!("Message: {}", state.message));

        if ui.link("Edit Message").clicked() {
            state
                .inbox
                .send(RouterMessage::Navigate("/edit".to_string()));
        }

        ui.label("Navigate to post:");

        if ui.link("Post 1").clicked() {
            state
                .inbox
                .send(RouterMessage::Navigate("/post/1".to_string()));
        }

        if ui.link("Post 2").clicked() {
            state
                .inbox
                .send(RouterMessage::Navigate("/post/2".to_string()));
        }

        if ui.link("Invalid Post").clicked() {
            state
                .inbox
                .send(RouterMessage::Navigate("/post/".to_string()));
        }
    }
}

fn edit_message(request: &mut Request<AppState>) -> impl Route<AppState> {
    |ui: &mut Ui, state: &mut AppState| {
        ui.heading("Edit Message");
        ui.text_edit_singleline(&mut state.message);

        if ui.button("Save").clicked() {
            state.inbox.send(RouterMessage::Back);
        }
    }
}

fn post(request: &mut Request<AppState>) -> impl Route<AppState> {
    let id = request.params.get("id").map(ToOwned::to_owned);

    move |ui: &mut Ui, state: &mut AppState| {
        if let Some(id) = &id {
            ui.label(format!("Post: {}", id));
        } else {
            ui.label("Post not found");
        }
    }
}
