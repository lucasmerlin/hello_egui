use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use eframe::egui;
use eframe::egui::Color32;
use eframe::emath::lerp;
use egui::ecolor::Hsva;
use egui::{Align2, Area, Context, Frame, Id, Rounding, ScrollArea, Sense, Ui, Vec2};

use egui_dnd::{dnd, DragDropItem};

use crate::load_stargazers::{load_stargazers, ImageState, StargazersState, StargazersType};

mod load_stargazers;

#[derive(Clone)]
struct Color {
    color: Color32,
    name: &'static str,
    rounded: bool,
    index: usize,
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

fn dnd_ui(items: &mut Vec<Color>, ui: &mut Ui, many: bool) {
    let item_size = if many {
        Vec2::splat(32.0)
    } else {
        Vec2::new(ui.available_width(), 32.0)
    };

    let response = dnd(ui, "fancy_dnd").show_custom(|ui, iter| {
        items.iter_mut().enumerate().for_each(|(index, item)| {
            iter.next(Id::new(item.index), item, index, |item| {
                item.ui_sized(ui, item_size, |ui, item, handle, state| {
                    ui.horizontal(|ui| {
                        handle.ui_sized(ui, item_size, |ui| {
                            let size_factor = ui.ctx().animate_value_with_time(
                                item.id().with("handle_anim"),
                                if state.dragged { 1.1 } else { 1.0 },
                                0.2,
                            );
                            let size = 32.0;

                            let (_id, response) =
                                ui.allocate_exact_size(Vec2::splat(size), Sense::click());

                            if response.clicked() {
                                item.rounded = !item.rounded;
                            }
                            let rect = response.rect;

                            let x = ui.ctx().animate_bool(item.id(), item.rounded);
                            let rounding = x * 16.0 + 1.0;

                            ui.painter().rect_filled(
                                rect.shrink(x * 4.0 * size_factor)
                                    .shrink(rect.width() * (1.0 - size_factor)),
                                Rounding::same(rounding),
                                item.color,
                            );

                            if !many {
                                ui.heading(item.name);
                            }
                        });
                    });
                })
            })
        });
    });

    response.update_vec(items);

    if let Some(reason) = response.cancellation_reason() {
        println!("Drag has been cancelled because of {:?}", reason);
    }
}

fn stargazers_ui(ui: &mut Ui, stargazers: StargazersType) {
    let clone = stargazers.clone();
    let mut guard = stargazers.lock().unwrap();

    ScrollArea::vertical()
        .max_height(250.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Like");
                ui.hyperlink_to(
                    "egui_dnd on GitHub",
                    "https://github.com/lucasmerlin/egui_dnd",
                );
                ui.label("to be listed here!");
            });

            match &mut *guard {
                StargazersState::None => {
                    load_stargazers(clone);
                }
                StargazersState::Loading => {
                    ui.spinner();
                }
                StargazersState::Data(data) => {
                    dnd(ui, "stargazers_dnd").show_vec(data, |ui, item, handle, state| {
                        ui.horizontal(|ui| {
                            handle.ui(ui, |ui| {
                                Frame::none()
                                    .fill(ui.visuals().faint_bg_color)
                                    .inner_margin(8.0)
                                    .outer_margin(2.0)
                                    .rounding(4.0)
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());

                                        let size = Vec2::new(32.0, 32.0);
                                        if ui.is_rect_visible(ui.min_rect().expand2(size)) {
                                            item.load_image();
                                        }

                                        let image = item.image.lock().unwrap();
                                        match &*image {
                                            ImageState::Data(image) => {
                                                image.show_size(ui, size);
                                            }
                                            ImageState::Loading => {
                                                ui.allocate_ui(size, |ui| {
                                                    ui.spinner();
                                                });
                                            }
                                            ImageState::Error(e) => {
                                                ui.allocate_ui(size, |ui| {
                                                    ui.label(&*e);
                                                });
                                            }
                                            _ => {
                                                ui.allocate_space(size);
                                            }
                                        }

                                        ui.hyperlink_to(
                                            item.login.as_str(),
                                            item.html_url.as_str(),
                                        );
                                    });
                            });
                        });
                    });
                }
                StargazersState::Error(e) => {
                    ui.label(&*e);
                }
            }
        });
}

fn colors() -> Vec<Color> {
    vec![
        Color {
            name: "Panic Purple",
            color: egui::hex_color!("642CA9"),
            rounded: false,
            index: 0,
        },
        Color {
            name: "Generic Green",
            color: egui::hex_color!("2A9D8F"),
            rounded: false,
            index: 1,
        },
        Color {
            name: "Ownership Orange*",
            color: egui::hex_color!("E9C46A"),
            rounded: false,
            index: 2,
        },
    ]
}

fn many_colors() -> Vec<Color> {
    let colors = 18;

    (0..colors)
        .map(|i| {
            let hue = i as f32 / colors as f32;
            let color = Color32::from(Hsva::new(hue, 0.8, 0.8, 1.0));
            Color {
                name: "Generic Green",
                color,
                rounded: false,
                index: i,
            }
        })
        .collect()
}

fn app(ctx: &Context, demo: &mut Demo, items: &mut Vec<Color>, stargazers: StargazersType) {
    egui::CentralPanel::default().frame(egui::Frame::none()
        .fill(ctx.style().visuals.panel_fill.gamma_multiply(0.7))
    ).show(ctx, |ui| {
        if items.len() == 3 {
            vertex_gradient(
                ui,
                &Gradient(
                    items
                        .iter()
                        .map(|c| c.color)
                        .collect(),
                ),
            );
        }


        Area::new("content")
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(300.0);

                egui::Frame::none()
                    .fill(ui.style().visuals.panel_fill)
                    .rounding(4.0)
                    .inner_margin(20.0).show(ui, |ui| {
                        ui.heading("Color Sort");

                        ui.horizontal(|ui| {
                            ui.selectable_value(demo, Demo::Vertical, "Vertical");
                            ui.selectable_value(demo, Demo::Horizontal, "Horizontal");
                            ui.selectable_value(demo, Demo::Stargazers, "Stargazers");
                        });

                        if demo == &Demo::Vertical && items.len() > 3 {
                            *items = colors();
                        }
                        if demo == &Demo::Horizontal && items.len() == 3 {
                            *items = many_colors();
                        }

                        if demo == &Demo::Stargazers {
                            stargazers_ui(ui, stargazers.clone());
                        } else {


                            // Done here again in case items changed
                            let many = items.len() > 3;

                                ui.spacing_mut().item_spacing.x = ui.spacing().item_spacing.y;
                                if many {
                                    ui.horizontal_wrapped(|ui| {
                                        dnd_ui(items, ui, many);
                                    });
                                } else {
                                    dnd_ui(items, ui, many);
                                }

                            ui.add_space(5.0);
                            if !many {
                                ui.small("* it's actually yellow");
                            } else {
                                ui.small(" ");
                            }
                        }

                        ui.separator();

                        ui.label("This is a demo for egui_dnd, a drag and drop sorting library for egui.");

                        ui.hyperlink_to("View on GitHub", "https://github.com/lucasmerlin/egui_dnd");
                        ui.hyperlink_to("View on Crates.io", "https://crates.io/crates/egui_dnd");
                        ui.hyperlink_to("View on docs.rs", "https://docs.rs/egui_dnd");
                    });
            });
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let mut items = colors();
    let stargazers: StargazersType = Arc::new(Mutex::new(StargazersState::None));
    let mut demo = Demo::Vertical;

    eframe::run_simple_native("Dnd Example App", Default::default(), move |ctx, _frame| {
        app(ctx, &mut demo, &mut items, stargazers.clone());
    })
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();
    let items = colors();
    let stargazers: StargazersType = Arc::new(Mutex::new(StargazersState::None));
    let mut demo = Demo::Vertical;

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "canvas",
                web_options,
                Box::new(|_a| Box::new(App(items, stargazers, demo))),
            )
            .await
            .expect("failed to start eframe");
    });

    struct App(Vec<Color>, StargazersType, Demo);

    impl eframe::App for App {
        fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
            app(ctx, &mut self.2, &mut self.0, self.1.clone());
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Gradient(pub Vec<Color32>);

#[derive(Clone, Hash, PartialEq, Eq)]
enum Demo {
    Horizontal,
    Vertical,
    Stargazers,
}

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
