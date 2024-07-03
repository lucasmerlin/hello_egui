use std::hash::Hash;

use eframe::egui::Color32;
use eframe::emath::lerp;
use eframe::{egui, Frame};
use egui::{Context, Id, SidePanel, Ui};

use egui_inbox::UiInbox;
use egui_router::EguiRouter;
use hello_egui_utils::center::Center;
use shared_state::SharedState;
use sidebar::SideBar;

use crate::routes::router;

mod chat;
mod color_sort;
mod crate_ui;
mod example;
mod futures;
mod gallery;
mod routes;
mod shared_state;
mod sidebar;
mod signup_form;
mod stargazers;

pub enum FancyMessage {
    Navigate(String),
}

pub struct App {
    sidebar: SideBar,
    sidebar_expanded: bool,
    shared_state: SharedState,
    inbox: UiInbox<FancyMessage>,
    router: EguiRouter<SharedState>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let (tx, inbox) = UiInbox::channel();
        let mut state = SharedState::new(tx);

        let router = router(&mut state);
        Self {
            inbox,
            sidebar: SideBar::new(),
            shared_state: state,
            sidebar_expanded: false,
            router,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.inbox.set_ctx(ctx);
        self.inbox.read_without_ctx().for_each(|msg| match msg {
            FancyMessage::Navigate(route) => {
                self.router.navigate(&mut self.shared_state, route).unwrap();
            }
        });

        let width = ctx.screen_rect().width();
        let collapsible_sidebar = width < 800.0;
        let is_expanded = !collapsible_sidebar || self.sidebar_expanded;

        SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(170.0)
            .show_animated(ctx, is_expanded, |ui| {
                if self.sidebar.ui(ui, &mut self.shared_state) {
                    self.sidebar_expanded = false;
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(ctx.style().visuals.panel_fill.gamma_multiply(0.7)))
            .show(ctx, |ui| {
                vertex_gradient(
                    ui,
                    &Gradient(
                        self.shared_state
                            .background_colors
                            .iter()
                            .map(|c| c.color)
                            .collect(),
                    ),
                );

                if collapsible_sidebar {
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        if ui.add(egui::Button::new("☰")).clicked() {
                            self.sidebar_expanded = !self.sidebar_expanded;
                        }
                    });
                }

                if !(collapsible_sidebar && is_expanded) {
                    self.router.ui(ui, &mut self.shared_state);
                }
            });
    }
}

pub fn demo_area(ui: &mut Ui, title: &'static str, width: f32, content: impl FnOnce(&mut Ui)) {
    Center::new(title).ui(ui, |ui| {
        let width = f32::min(ui.available_width() - 20.0, width);
        ui.set_max_width(width);
        ui.set_max_height(ui.available_height() - 20.0);

        egui::Frame::none()
            .fill(ui.style().visuals.panel_fill)
            .rounding(4.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.heading(title);
                ui.add_space(5.0);

                content(ui);
            });
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        })
    });

    eframe::run_native(
        "Dnd Example App",
        Default::default(),
        Box::new(move |ctx| {
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            egui_thumbhash::register(&ctx.egui_ctx);
            Ok(Box::new(App::new()) as Box<dyn eframe::App>)
        }),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "canvas",
                web_options,
                Box::new(|a| {
                    egui_extras::install_image_loaders(&a.egui_ctx);
                    egui_thumbhash::register(&a.egui_ctx);
                    Box::new(App::new()) as Box<dyn eframe::App>
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Gradient(pub Vec<Color32>);

// taken from the egui demo
fn vertex_gradient(ui: &mut Ui, gradient: &Gradient) {
    use egui::epaint::*;

    let rect = ui.max_rect();

    let n = gradient.0.len();
    let animation_time = 0.4;
    assert!(n >= 2);
    let mut mesh = Mesh::default();
    for (i, &color) in gradient.0.iter().enumerate() {
        let t = i as f32 / (n as f32 - 1.0);
        let y = lerp(rect.y_range(), t);
        mesh.colored_vertex(
            pos2(rect.left(), y),
            animate_color(ui, color, Id::new("a").with(i), animation_time),
        );
        mesh.colored_vertex(
            pos2(rect.right(), y),
            animate_color(ui, color, Id::new("b").with(i), animation_time),
        );
        if i < n - 1 {
            let i = i as u32;
            mesh.add_triangle(2 * i, 2 * i + 1, 2 * i + 2);
            mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
        }
    }
    ui.painter().add(Shape::mesh(mesh));
}

fn animate_color(ui: &mut Ui, color: Color32, id: Id, duration: f32) -> Color32 {
    Color32::from_rgba_premultiplied(
        ui.ctx()
            .animate_value_with_time(id.with(0), color[0] as f32, duration) as u8,
        ui.ctx()
            .animate_value_with_time(id.with(1), color[1] as f32, duration) as u8,
        ui.ctx()
            .animate_value_with_time(id.with(2), color[2] as f32, duration) as u8,
        color[3],
    )
}
