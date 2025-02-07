use crate::crate_ui::{crate_usage_ui, Crate, CrateUsage};
use crate::demo_area;
use crate::example::{Example, ExampleTrait};
use crate::shared_state::SharedState;
use eframe::emath::Vec2;
use eframe::epaint::{Color32, Hsva};
use egui::{CornerRadius, Id, Sense, Ui};
use egui_dnd::{dnd, DragDropItem};
use std::hash::{Hash, Hasher};

pub const COLOR_SORT_EXAMPLE: Example = Example {
    name: "Color Sort",
    slug: "color_sort_vertical",
    crates: &[CrateUsage::simple(Crate::EguiDnd)],
    get: || Box::new(ColorSort::vertical()),
};

pub const COLOR_SORT_WRAPPED_EXAMPLE: Example = Example {
    name: "Color Sort (wrapped)",
    slug: "color_sort_wrapped",
    crates: &[CrateUsage::simple(Crate::EguiDnd)],
    get: || Box::new(ColorSort::wrapped()),
};

#[derive(Clone)]
pub struct Color {
    #[allow(clippy::struct_field_names)]
    pub color: Color32,
    pub name: &'static str,
    pub rounded: bool,
    pub index: usize,
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

pub enum ColorSortKind {
    // Vertical colors are in shared state so we can use it as a global background color
    Vertical,
    Wrapped { colors: Vec<Color> },
}

pub struct ColorSort {
    kind: ColorSortKind,
}

impl ColorSort {
    fn name(&self) -> &'static str {
        match self.kind {
            ColorSortKind::Vertical => "Color Sort",
            ColorSortKind::Wrapped { .. } => "Color Sort (wrapped)",
        }
    }

    pub fn wrapped() -> Self {
        Self {
            kind: ColorSortKind::Wrapped {
                colors: many_colors(),
            },
        }
    }

    pub fn vertical() -> Self {
        Self {
            kind: ColorSortKind::Vertical,
        }
    }

    fn dnd_ui(items: &mut [Color], ui: &mut Ui, many: bool) {
        let item_size = if many {
            Vec2::splat(32.0)
        } else {
            Vec2::new(ui.available_width(), 32.0)
        };

        let response = dnd(ui, "fancy_dnd").show_custom(|ui, iter| {
            items.iter_mut().enumerate().for_each(|(index, item)| {
                iter.next(ui, Id::new(item.index), index, true, |ui, item_handle| {
                    item_handle.ui_sized(ui, item_size, |ui, handle, state| {
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
                                let rounding = (x * 16.0 + 1.0).round() as u8;

                                ui.painter().rect_filled(
                                    rect.shrink(x * 4.0 * size_factor)
                                        .shrink(rect.width() * (1.0 - size_factor)),
                                    CornerRadius::same(rounding),
                                    item.color,
                                );

                                if !many {
                                    ui.heading(item.name);
                                }
                            });
                        });
                    })
                });
            });
        });

        response.update_vec(items);
    }
}

impl ExampleTrait for ColorSort {
    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState) {
        demo_area(ui, self.name(), 286.0, |ui| {
            ui.spacing_mut().item_spacing.x = ui.spacing().item_spacing.y;
            match &mut self.kind {
                ColorSortKind::Vertical => {
                    Self::dnd_ui(&mut shared_state.background_colors, ui, false);
                    ui.add_space(5.0);
                    ui.small("* it's actually yellow");
                }
                ColorSortKind::Wrapped { colors } => {
                    ui.horizontal_wrapped(|ui| {
                        Self::dnd_ui(colors, ui, true);
                    });
                    ui.small("");
                }
            }

            crate_usage_ui(ui, COLOR_SORT_EXAMPLE.crates, shared_state);
        });
    }
}

fn many_colors() -> Vec<Color> {
    let colors = 21;

    (0..colors)
        .map(|i| {
            let hue = i as f32 / colors as f32;
            let color = Color32::from(Hsva::new(hue, 0.8, 0.8, 1.0));
            Color {
                name: "Rainbow Color",
                color,
                rounded: false,
                index: i,
            }
        })
        .collect()
}
