use crate::color_sort::Color;
use crate::FancyMessage;
use egui_inbox::UiInboxSender;

pub struct SharedState {
    pub background_colors: Vec<Color>,
    pub tx: UiInboxSender<FancyMessage>,
    pub active_route: String,
}

impl SharedState {
    pub fn new(tx: UiInboxSender<FancyMessage>) -> Self {
        Self {
            background_colors: colors(),
            tx,
            active_route: "/example/color_sort".to_owned(),
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
