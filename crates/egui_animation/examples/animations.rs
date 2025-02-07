use eframe::emath::Align;
use eframe::{egui, NativeOptions};
use egui::{CentralPanel, ComboBox, Layout, ScrollArea, Vec2};
use egui_animation::{animate_ui_translation, Collapse};
use hello_egui_utils::measure_text;
use rand::seq::SliceRandom;

#[allow(clippy::type_complexity)]
const EASINGS: [(fn(f32) -> f32, &str); 31] = [
    (simple_easing::cubic_in_out, "Cubic in-out"),
    (simple_easing::cubic_in, "Cubic in"),
    (simple_easing::cubic_out, "Cubic out"),
    (simple_easing::linear, "Linear"),
    (simple_easing::quad_in_out, "Quadratic in-out"),
    (simple_easing::quad_in, "Quadratic in"),
    (simple_easing::quad_out, "Quadratic out"),
    (simple_easing::quart_in_out, "Quartic in-out"),
    (simple_easing::quart_in, "Quartic in"),
    (simple_easing::quart_out, "Quartic out"),
    (simple_easing::quint_in_out, "Quintic in-out"),
    (simple_easing::quint_in, "Quintic in"),
    (simple_easing::quint_out, "Quintic out"),
    (simple_easing::sine_in_out, "Sine in-out"),
    (simple_easing::sine_in, "Sine in"),
    (simple_easing::sine_out, "Sine out"),
    (simple_easing::bounce_in_out, "Bounce in-out"),
    (simple_easing::bounce_in, "Bounce in"),
    (simple_easing::bounce_out, "Bounce out"),
    (simple_easing::elastic_in_out, "Elastic in-out"),
    (simple_easing::elastic_in, "Elastic in"),
    (simple_easing::elastic_out, "Elastic out"),
    (simple_easing::back_in_out, "Back in-out"),
    (simple_easing::back_in, "Back in"),
    (simple_easing::back_out, "Back out"),
    (simple_easing::circ_in_out, "Circular in-out"),
    (simple_easing::circ_in, "Circular in"),
    (simple_easing::circ_out, "Circular out"),
    (simple_easing::expo_in_out, "Exponential in-out"),
    (simple_easing::expo_in, "Exponential in"),
    (simple_easing::expo_out, "Exponential out"),
];

#[allow(clippy::too_many_lines)] // It's ok for an example.
pub fn main() -> eframe::Result<()> {
    let mut target = 0.0;

    let mut easing: fn(f32) -> f32 = simple_easing::cubic_in_out;

    let text_de = "“Die Bild-Zeitung ist ein Organ der Niedertracht. Es ist falsch, sie zu lesen. Jemand, der zu dieser Zeitung beiträgt, ist gesellschaftlich absolut inakzeptabel. Es wäre verfehlt, zu einem ihrer Redakteure freundlich oder auch nur höflich zu sein. Man muss so unfreundlich zu ihnen sein, wie es das Gesetz gerade noch zuläßt. Es sind schlechte Menschen, die Falsches tun.” (Max Goldt)";
    let text_en = "“The Bild newspaper is an organ of malice. It is wrong to read it. Anyone who contributes to this newspaper is socially absolutely unacceptable. It would be wrong to be friendly or even polite to one of their editors. You have to be as unfriendly to them as the law allows. They are bad people who do wrong.” (Max Goldt)";

    let mut text = text_de;

    let mut words: Vec<_> = text.split("").collect();
    let mut ids: Vec<_> = (0..words.len()).collect();

    let mut visible = true;

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ComboBox::new("easing", "Select Easing")
                        .selected_text(
                            EASINGS
                                .iter()
                                .find(|(val, _name)| *val == easing)
                                .unwrap()
                                .1,
                        )
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            for (easing_fn, name) in &EASINGS {
                                ui.selectable_value(&mut easing, *easing_fn, *name);
                            }
                        });
                });

                if ui.button("Animate").clicked() {
                    if target == 0.0 {
                        target = 200.0;
                    } else {
                        target = 0.0;
                    }
                }

                let x = egui_animation::animate_eased(ui.ctx(), "test", target, 1.0, easing);

                ui.horizontal(|ui| {
                    ui.add_space(x);

                    ui.label("Ayyy");
                });

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Shuffle Letters").clicked() {
                            let mut rng = rand::rng();
                            ids.shuffle(&mut rng);
                            text = if text == text_de { text_en } else { text_de };
                            words = text.split("").collect();
                        }

                        if ui.button("Shuffle Words").clicked() {
                            let mut rng = rand::rng();
                            ids.shuffle(&mut rng);
                            text = if text == text_de { text_en } else { text_de };
                            words = text.split_inclusive(' ').collect();
                        }
                    });

                    ui.with_layout(
                        Layout::left_to_right(Align::Min).with_main_wrap(true),
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;

                            let mut iter = words.iter().zip(ids.iter()).peekable();

                            let mut words = vec![];

                            while let Some((text, id)) = iter.next() {
                                if text.chars().count() > 1 {
                                    let size = measure_text(ui, *text);
                                    animate_ui_translation(
                                        ui,
                                        *id,
                                        simple_easing::cubic_out,
                                        size,
                                        false,
                                        |ui| {
                                            ui.label(*text);
                                        },
                                    );
                                } else if text == &" " || iter.peek().is_none() {
                                    words.push((*id, text));

                                    let text = words.iter().map(|(_, text)| text).fold(
                                        String::new(),
                                        |mut acc, text| {
                                            acc.push_str(text);
                                            acc
                                        },
                                    );

                                    let size = measure_text(ui, text) + Vec2::new(4.0, 0.0);

                                    ui.allocate_ui(size, |ui| {
                                        for (id, text) in &words {
                                            let size = measure_text(ui, **text);
                                            animate_ui_translation(
                                                ui,
                                                id,
                                                simple_easing::cubic_out,
                                                size,
                                                false,
                                                |ui| {
                                                    ui.label(**text);
                                                },
                                            );
                                        }
                                    });
                                    words = vec![];
                                } else {
                                    words.push((*id, text));
                                }
                            }
                        },
                    );
                });

                if ui.button("Toggle Collapse").clicked() {
                    visible = !visible;
                }

                Collapse::vertical("collapse", visible).ui(ui, |ui| {
                    ui.group(|ui| {
                        ScrollArea::vertical()
                            .max_height(100.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add_space(50.0);

                                animate_ui_translation(
                                    ui,
                                    "haaa",
                                    simple_easing::cubic_in_out,
                                    Vec2::new(200.0, 10.0),
                                    true,
                                    |ui| {
                                        ui.label(text_de);
                                    },
                                );

                                ui.add_space(1000.0);
                            });
                    });
                });
            });
        },
    )
}
