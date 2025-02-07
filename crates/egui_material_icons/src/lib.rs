#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
use egui::{Button, FontData, FontFamily, Frame, Response, RichText, Widget};

pub mod icons;

pub const FONT_DATA: &[u8] = include_bytes!("../MaterialSymbolsRounded_Filled-Regular.ttf");

pub fn font_insert() -> FontInsert {
    let mut data = FontData::from_static(FONT_DATA);
    data.tweak.y_offset_factor = 0.05;

    FontInsert::new(
        "material-icons",
        data,
        vec![InsertFontFamily {
            family: FontFamily::Proportional,
            priority: FontPriority::Lowest,
        }],
    )
}

pub fn initialize(ctx: &egui::Context) {
    ctx.add_font(font_insert());
}

pub fn icon_button(ui: &mut egui::Ui, icon: &str) -> Response {
    Frame::new()
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
