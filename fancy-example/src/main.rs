use std::hash::Hash;
use std::time::Duration;

use eframe::egui::Color32;
use eframe::emath::lerp;
use eframe::{egui, Frame};
use egui::{Align2, Area, Context, Id, SidePanel, Ui, Vec2};

use color_sort::ColorSort;

use shared_state::SharedState;
use sidebar::{Category, SideBar};

use crate::chat::ChatExample;
use crate::futures::sleep;
use crate::stargazers::Stargazers;

mod chat;
mod color_sort;
mod futures;
mod shared_state;
mod sidebar;
mod stargazers;

pub struct App {
    sidebar: SideBar,
    shared_state: SharedState,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            sidebar: SideBar::new(vec![
                Category {
                    name: "Drag and Drop".to_string(),
                    examples: vec![
                        Box::new(ColorSort::vertical()),
                        Box::new(ColorSort::wrapped()),
                        Box::new(Stargazers::new()),
                    ],
                },
                Category {
                    name: "Infinite Scroll".to_string(),
                    examples: vec![Box::new(Stargazers::new()), Box::new(ChatExample::new())],
                },
            ]),
            shared_state: SharedState::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(110.0)
            .show(ctx, |ui| {
                self.sidebar.ui(ui);
            });

        let example = self.sidebar.active_example_mut();

        egui::CentralPanel::default().frame(egui::Frame::none()
            .fill(ctx.style().visuals.panel_fill.gamma_multiply(0.7))
        ).show(ctx, |ui| {
                vertex_gradient(
                    ui,
                    &Gradient(
                        self.shared_state.background_colors
                            .iter()
                            .map(|c| c.color)
                            .collect(),
                    ),
                );

            Area::new(Id::new(example.name()))
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.set_width(300.0);

                    egui::Frame::none()
                        .fill(ui.style().visuals.panel_fill)
                        .rounding(4.0)
                        .inner_margin(20.0)
                        .show(ui, |ui| {

                            ui.heading(example.name());

                            ui.add_space(5.0);

                            example.ui(ui, &mut self.shared_state);

                            ui.separator();

                            ui.label("This is a demo for egui_dnd, a drag and drop sorting library for egui.");

                            ui.hyperlink_to("View on GitHub", "https://github.com/lucasmerlin/hello_egui/tree/main/crates/egui_dnd");
                            ui.hyperlink_to("View on Crates.io", "https://crates.io/crates/egui_dnd");
                            ui.hyperlink_to("View on docs.rs", "https://docs.rs/egui_dnd");
                        });
                });
        });
    }
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
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    tokio::spawn(async move {
        sleep(std::time::Duration::from_secs(1)).await;
        println!("Hello from tokio!");
    });

    eframe::run_native(
        "Dnd Example App",
        Default::default(),
        Box::new(move |ctx| {
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            Box::new(App::new()) as Box<dyn eframe::App>
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
