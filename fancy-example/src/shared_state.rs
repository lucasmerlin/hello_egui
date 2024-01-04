use crate::color_sort::Color;

pub struct SharedState {
    pub background_colors: Vec<Color>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            background_colors: colors(),
        }
    }
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
