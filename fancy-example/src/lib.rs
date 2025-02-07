use eframe::egui::Color32;
use eframe::emath::lerp;
use eframe::{egui, Frame};
use egui::{Context, Id, SidePanel, Ui};
use std::hash::Hash;
use std::num::NonZeroUsize;

use egui_inbox::UiInbox;
use egui_router::EguiRouter;
use hello_egui_utils::center::Center;
use shared_state::SharedState;
use sidebar::SideBar;

use crate::routes::router;

pub mod chat;
mod color_sort;
mod crate_ui;
pub mod example;
mod flex;
mod futures;
pub mod gallery;
mod routes;
mod shared_state;
mod sidebar;
mod signup_form;
pub mod stargazers;

pub enum FancyMessage {
    Navigate(String),
}

pub struct App {
    sidebar_expanded: bool,
    shared_state: SharedState,
    inbox: UiInbox<FancyMessage>,
    router: EguiRouter<SharedState>,
}

impl App {
    pub fn new(ctx: &Context) -> Self {
        let (tx, inbox) = UiInbox::channel();
        let mut state = SharedState::new(tx);

        let router = router(&mut state);

        ctx.options_mut(|opts| {
            opts.max_passes = NonZeroUsize::new(4).unwrap();
        });

        egui_extras::install_image_loaders(ctx);
        egui_thumbhash::register(ctx);

        Self {
            inbox,
            shared_state: state,
            sidebar_expanded: false,
            router,
        }
    }

    pub fn show(&mut self, ctx: &Context) {
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
                if SideBar::ui(ui, &mut self.shared_state) {
                    self.sidebar_expanded = false;
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(ctx.style().visuals.panel_fill.gamma_multiply(0.7)))
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

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.show(ctx);
    }
}

pub fn demo_area(ui: &mut Ui, title: &'static str, width: f32, content: impl FnOnce(&mut Ui)) {
    Center::new(title).ui(ui, |ui| {
        let width = f32::min(ui.available_width() - 20.0, width);
        ui.set_max_width(width);
        ui.set_max_height(ui.available_height() - 20.0);

        egui::Frame::NONE
            .fill(ui.style().visuals.panel_fill)
            .corner_radius(4.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.heading(title);
                ui.add_space(5.0);

                content(ui);
            });
    });
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Gradient(pub Vec<Color32>);

// taken from the egui demo
fn vertex_gradient(ui: &mut Ui, gradient: &Gradient) {
    use egui::epaint::{pos2, Mesh, Shape};

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
            .animate_value_with_time(id.with(0), f32::from(color[0]), duration) as u8,
        ui.ctx()
            .animate_value_with_time(id.with(1), f32::from(color[1]), duration) as u8,
        ui.ctx()
            .animate_value_with_time(id.with(2), f32::from(color[2]), duration) as u8,
        color[3],
    )
}
