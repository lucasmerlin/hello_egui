use std::hash::{Hash, Hasher};

use eframe::{App, egui, Frame};
use eframe::egui::{Color32, Context};
use eframe::emath::lerp;
use egui::{Id, Rounding, Sense, Ui, Vec2};
use egui_extras::{Size, StripBuilder};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use egui_dnd::{DragDropItem, DragDropUi};
use egui_dnd::utils::shift_vec;

#[derive(Clone)]
struct Color {
    color: Color32,
    name: String,
    rounded: bool,
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

struct DnDApp {
    dnd: DragDropUi,
    items: Vec<Color>,
}

// ff36ab abff36 36abff

// 9742ff 42ff97 ff9742

impl Default for DnDApp {
    fn default() -> Self {
        DnDApp {
            dnd: DragDropUi::default(),
            items: vec![
                Color {
                    name: "Panic Purple".to_string(),
                    color: egui::hex_color!("642CA9"),
                    rounded: false,
                },
                Color {
                    name: "Generic Green".to_string(),
                    color: egui::hex_color!("2A9D8F"),
                    rounded: false,
                },
                Color {
                    name: "Ownership Orange*".to_string(),
                    color: egui::hex_color!("E9C46A"),
                    rounded: false,
                },
            ],
        }
    }
}

impl DnDApp {
    fn dnd_ui(&mut self, ui: &mut Ui) {
        let response = self
            .dnd
            .ui(ui, self.items.iter_mut(), |item: &mut Color, ui, handle, pressed| {
                ui.horizontal(|ui| {
                    if handle
                        .sense(Sense::click())
                        .ui(ui, |ui| {
                            let (_id, rect) = ui.allocate_space(Vec2::new(32.0, 32.0));

                            let x = ui.ctx().animate_bool(item.id(), item.rounded);
                            let rounding = x * 16.0 + 1.0;

                            ui.painter().rect_filled(
                                rect.shrink(x * 4.0),
                                Rounding::same(rounding),
                                item.color,
                            );

                            ui.heading(&item.name);
                        })
                        .clicked()
                    {
                        item.rounded = !item.rounded;
                    }
                });
            });

        response.update_vec(&mut self.items);
    }
}

impl App for DnDApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            vertex_gradient(
                ui,
                &Gradient(
                        self.items
                        .iter()
                        .map(|c| c.color)
                        .collect(),
                ),
            );

            StripBuilder::new(ui)
                .size(Size::remainder())
                .size(Size::exact(260.0))
                .size(Size::remainder())
                .horizontal(|mut strip| {
                    strip.empty();

                    strip.strip(|builder| {
                        builder
                            .size(Size::remainder())
                            .size(Size::exact(300.0))
                            .size(Size::remainder())
                            .vertical(|mut strip| {
                                strip.empty();

                                strip.cell(|ui| {
                                    ui.painter().rect_filled(
                                        ui.available_rect_before_wrap(),
                                        Rounding::same(4.0),
                                        ui.style().visuals.panel_fill,
                                    );

                                    egui::Frame::none().outer_margin(20.0).show(ui, |ui| {
                                        ui.heading("Color Sort");
                                        self.dnd_ui(ui);

                                        ui.add_space(5.0);
                                        ui.small("* it's actually yellow");

                                        ui.add_space(15.0);
                                        ui.separator();
                                        ui.add_space(15.0);

                                        ui.label("This is a demo for egui_dnd, a drag and drop sorting library for egui.");

                                        ui.hyperlink_to("View on GitHub", "https://github.com/lucasmerlin/egui_dnd");
                                        ui.hyperlink_to("View on Crates.io", "https://crates.io/crates/egui_dnd")
                                    });
                                });
                                strip.empty();
                            });
                    });

                    strip.empty();
                });
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "DnD Example App",
        options,
        Box::new(|_a| Box::new(DnDApp::default())),
    )
        .unwrap();
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let runner = eframe::WebRunner::new();
        runner
            .start(
                "canvas",
                web_options,
                Box::new(|_a| Box::new(DnDApp::default())),
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
        mesh.colored_vertex(pos2(rect.left(), y), animate_color(ui, color, Id::new("a").with(i), animation_time));
        mesh.colored_vertex(pos2(rect.right(), y), animate_color(ui, color, Id::new("b").with(i), animation_time));
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
        ui.ctx().animate_value_with_time(id.with(0), color[0] as f32, duration) as u8,
        ui.ctx().animate_value_with_time(id.with(1), color[1] as f32, duration) as u8,
        ui.ctx().animate_value_with_time(id.with(2), color[2] as f32, duration) as u8,
        color[3],
    )
}
