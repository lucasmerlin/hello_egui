#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

#[cfg(not(any(feature = "filled", feature = "outline")))]
compile_error!("At least one of `filled` or `outline` features must be enabled.");

#[cfg(feature = "compressed")]
use include_flate::flate;

use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
use egui::{Button, FontData, FontFamily, Frame, Response, RichText, Widget};

pub mod icons;

// =============================================================================
// Font data (uncompressed)
// =============================================================================

// When both filled and outline are enabled, or only filled
#[cfg(all(feature = "filled", not(feature = "compressed")))]
pub(crate) const FONT_DATA: &[u8] = include_bytes!("../MaterialSymbolsRounded_Filled-Regular.ttf");

// When only outline is enabled (no filled), use outline as the default font
#[cfg(all(
    feature = "outline",
    not(feature = "filled"),
    not(feature = "compressed")
))]
pub(crate) const FONT_DATA: &[u8] = include_bytes!("../MaterialSymbolsRounded-Regular.ttf");

// Separate outline font data when both are enabled
#[cfg(all(feature = "filled", feature = "outline", not(feature = "compressed")))]
pub(crate) const FONT_DATA_OUTLINED: &[u8] =
    include_bytes!("../MaterialSymbolsRounded-Regular.ttf");

// =============================================================================
// Font data (compressed)
// =============================================================================

#[cfg(all(feature = "filled", feature = "compressed"))]
flate!(pub(crate) static FONT_DATA: [u8] from "MaterialSymbolsRounded_Filled-Regular.ttf");

#[cfg(all(feature = "outline", not(feature = "filled"), feature = "compressed"))]
flate!(pub(crate) static FONT_DATA: [u8] from "MaterialSymbolsRounded-Regular.ttf");

#[cfg(all(feature = "filled", feature = "outline", feature = "compressed"))]
flate!(pub(crate) static FONT_DATA_OUTLINED: [u8] from "MaterialSymbolsRounded-Regular.ttf");

// =============================================================================
// Font family names
// =============================================================================

/// The font family name used for material icons.
pub const FONT_FAMILY: &str = "material-icons";

/// The font family name used for outlined material icons (requires `outline` feature).
#[cfg(feature = "outline")]
pub const FONT_FAMILY_OUTLINED: &str = "material-icons-outlined";

// =============================================================================
// OutlinedIcon type
// =============================================================================

/// An outlined icon that renders with the outline font family.
///
/// Use directly with widgets: `ui.button(ICON_OUTLINE_ADD)`
///
/// This type is only available when the `outline` feature is enabled.
#[cfg(feature = "outline")]
#[derive(Clone, Copy, Debug)]
pub struct OutlinedIcon(pub &'static str);

#[cfg(feature = "outline")]
impl From<OutlinedIcon> for egui::WidgetText {
    fn from(icon: OutlinedIcon) -> Self {
        RichText::new(icon.0)
            .family(FontFamily::Name(FONT_FAMILY_OUTLINED.into()))
            .into()
    }
}

#[cfg(feature = "outline")]
impl OutlinedIcon {
    /// Returns the icon as a [`RichText`] with the outline font family.
    pub fn rich_text(self) -> RichText {
        RichText::new(self.0).family(FontFamily::Name(FONT_FAMILY_OUTLINED.into()))
    }
}

// =============================================================================
// Font registration
// =============================================================================

/// Creates a [`FontInsert`] for the material icons font.
pub fn font_insert() -> FontInsert {
    let mut data = FontData::from_static(&FONT_DATA);
    data.tweak.y_offset_factor = 0.05;

    // When only outline is enabled, also register under the outlined family name
    #[cfg(all(feature = "outline", not(feature = "filled")))]
    let families = vec![
        InsertFontFamily {
            family: FontFamily::Proportional,
            priority: FontPriority::Lowest,
        },
        InsertFontFamily {
            family: FontFamily::Name(FONT_FAMILY.into()),
            priority: FontPriority::Highest,
        },
        // Also register as outlined family so ICON_OUTLINE_* works
        InsertFontFamily {
            family: FontFamily::Name(FONT_FAMILY_OUTLINED.into()),
            priority: FontPriority::Highest,
        },
    ];

    #[cfg(feature = "filled")]
    let families = vec![
        InsertFontFamily {
            family: FontFamily::Proportional,
            priority: FontPriority::Lowest,
        },
        InsertFontFamily {
            family: FontFamily::Name(FONT_FAMILY.into()),
            priority: FontPriority::Highest,
        },
    ];

    FontInsert::new(FONT_FAMILY, data, families)
}

/// Creates a [`FontInsert`] for the outlined material icons font.
///
/// This is only available when both `filled` and `outline` features are enabled.
#[cfg(all(feature = "filled", feature = "outline"))]
pub fn font_insert_outlined() -> FontInsert {
    let mut data = FontData::from_static(&FONT_DATA_OUTLINED);
    data.tweak.y_offset_factor = 0.05;

    FontInsert::new(
        FONT_FAMILY_OUTLINED,
        data,
        vec![
            InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: FontFamily::Name(FONT_FAMILY_OUTLINED.into()),
                priority: FontPriority::Highest,
            },
        ],
    )
}

/// Initializes the material icons font(s).
///
/// - With `filled` feature (default), registers the filled font.
/// - With `outline` feature, registers the outline font.
/// - With both, registers both fonts.
pub fn initialize(ctx: &egui::Context) {
    ctx.add_font(font_insert());
    #[cfg(all(feature = "filled", feature = "outline"))]
    ctx.add_font(font_insert_outlined());
}

// =============================================================================
// Helper functions
// =============================================================================

/// Creates a frameless icon button.
pub fn icon_button(ui: &mut egui::Ui, icon: &str) -> Response {
    Frame::new()
        .show(ui, |ui| {
            Button::new(RichText::new(icon).size(18.0))
                .frame(false)
                .ui(ui)
        })
        .inner
}

/// Creates a [`RichText`] from an icon.
pub fn icon_text(icon: &str) -> RichText {
    RichText::new(icon)
}
