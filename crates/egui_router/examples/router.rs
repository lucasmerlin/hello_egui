use eframe::NativeOptions;
use egui::{CentralPanel, Color32, Context, Frame, ScrollArea, Ui, Window};
use egui_inbox::type_inbox::TypeInbox;
use egui_router::{EguiRouter, Request, Route, TransitionConfig};
use std::borrow::Cow;

#[derive(Debug, Clone)]
struct AppState {
    message: String,
    inbox: TypeInbox,
}

enum RouterMessage {
    Navigate(String),
    Back,
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let init = |ctx: &Context| {
        let mut app_state = AppState {
            message: "Hello, World!".to_string(),
            inbox: TypeInbox::new(ctx),
        };
        let mut router = EguiRouter::builder()
            .transition(TransitionConfig::fade_up().with_easing(egui_animation::easing::quad_out))
            .default_duration(0.2)
            .default_path("/");

        router = router
            .route("/", home)
            .route("/edit", edit_message)
            .route("/post/{id}", post)
            .async_route("/async", async_route);

        (router.build(&mut app_state), app_state)
    };

    let mut router: Option<(EguiRouter<AppState>, AppState)> = None;
    let mut window_router: Option<(EguiRouter<AppState>, AppState)> = None;

    eframe::run_simple_native(
        "Router Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            let mut router = router.get_or_insert_with(|| init(ctx));
            let mut window_router = window_router.get_or_insert_with(|| init(ctx));

            for state in &mut [&mut router, &mut window_router] {
                state
                    .1
                    .inbox
                    .read()
                    .for_each(|msg: RouterMessage| match msg {
                        RouterMessage::Navigate(route) => {
                            state.0.navigate(&mut state.1, route).unwrap();
                        }
                        RouterMessage::Back => {
                            state.0.back().unwrap();
                        }
                    });
            }

            CentralPanel::default().show(ctx, |ui| {
                router.0.ui(ui, &mut router.1);
            });

            Window::new("Router Window")
                .frame(Frame::window(&ctx.style()).inner_margin(0.0))
                .show(ctx, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height());
                    window_router.0.ui(ui, &mut window_router.1);
                });
        },
    )
}

fn home() -> impl Route<AppState> {
    |ui: &mut Ui, state: &mut AppState| {
        background(ui, ui.style().visuals.faint_bg_color, |ui| {
            ui.heading("Home!");

            ui.label(format!("Message: {}", state.message));

            ui.label(format!("Id: {:?}", ui.next_auto_id()));

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

            if ui.link("Post With Query").clicked() {
                state
                    .inbox
                    .send(RouterMessage::Navigate("/post/3?search=test".to_string()));
            }

            if ui.link("Invalid Post").clicked() {
                state
                    .inbox
                    .send(RouterMessage::Navigate("/post/".to_string()));
            }

            if ui.link("Async Route").clicked() {
                state
                    .inbox
                    .send(RouterMessage::Navigate("/async".to_string()));
            }
        });
    }
}

fn edit_message() -> impl Route<AppState> {
    |ui: &mut Ui, state: &mut AppState| {
        background(ui, ui.style().visuals.window_fill, |ui| {
            ui.heading("Edit Message");
            ui.text_edit_singleline(&mut state.message);

            if ui.button("Save").clicked() {
                state.inbox.send(RouterMessage::Back);
            }
        });
    }
}

fn post(mut request: Request<AppState>) -> impl Route<AppState> {
    let id = request.params.get("id").map(ToOwned::to_owned);

    let search: Option<String> = request.query.remove("search").map(Cow::into_owned);

    move |ui: &mut Ui, state: &mut AppState| {
        background(ui, ui.style().visuals.extreme_bg_color, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                if let Some(id) = &id {
                    ui.label(format!("Post: {id}"));

                    ui.label(format!("Id: {:?}", ui.next_auto_id()));

                    if let Some(search) = &search {
                        ui.label(format!("Search: {search}"));
                    }

                    if ui.button("back").clicked() {
                        state.inbox.send(RouterMessage::Back);
                    }

                    ui.label(include_str!("../../../README.md"));
                } else {
                    ui.label("Post not found");
                    if ui.button("back").clicked() {
                        state.inbox.send(RouterMessage::Back);
                    }
                }
            });
        });
    }
}

async fn async_route() -> impl Route<AppState> {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    move |ui: &mut Ui, state: &mut AppState| {
        background(ui, ui.style().visuals.extreme_bg_color, |ui| {
            ui.heading("Async Route Loaded");

            if ui.button("back").clicked() {
                state.inbox.send(RouterMessage::Back);
            }
        });
    }
}

fn background(ui: &mut Ui, color: Color32, content: impl FnOnce(&mut Ui)) {
    Frame::NONE.fill(color).inner_margin(16.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());
        content(ui);
    });
}
