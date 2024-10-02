#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use egui::{
    Button, FontData, FontDefinitions, FontFamily, Frame, Margin, Response, RichText, Widget,
};

pub mod icons;

pub const FONT_DATA: &[u8] = include_bytes!("../MaterialSymbolsRounded_Filled-Regular.ttf");

pub fn initialize(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut data = FontData::from_static(FONT_DATA);
    data.tweak.y_offset_factor = 0.05;
    fonts.font_data.insert("material-icons".to_string(), data);
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .push("material-icons".to_string());

    ctx.set_fonts(fonts);
}

pub fn icon_button(ui: &mut egui::Ui, icon: &str) -> Response {
    Frame::none()
        .inner_margin(Margin {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        })
        .show(ui, |ui| {
            Button::new(RichText::new(icon).size(18.0))
                .frame(false)
                .ui(ui)
        })
        .inner
}

pub fn icon_text(icon: &str) -> RichText {
    RichText::new(icon)
}
