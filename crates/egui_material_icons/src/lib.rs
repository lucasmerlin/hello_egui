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

#[cfg(all(feature = "filled", not(feature = "compressed")))]
pub(crate) const FONT_DATA: &[u8] = include_bytes!("../MaterialSymbolsRounded_Filled-Regular.ttf");

#[cfg(all(feature = "outline", not(feature = "filled"), not(feature = "compressed")))]
pub(crate) const FONT_DATA: &[u8] = include_bytes!("../MaterialSymbolsRounded-Regular.ttf");

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

/// The font family name used for filled material icons.
pub const FONT_FAMILY: &str = "material-icons";

/// The font family name used for outlined material icons.
pub const FONT_FAMILY_OUTLINED: &str = "material-icons-outlined";

// =============================================================================
// IconStyle & MaterialIcon
// =============================================================================

/// The style of a material icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum IconStyle {
    Filled,
    Outlined,
}

/// A material icon that can be rendered as filled or outlined.
///
/// Use directly with widgets: `ui.button(ICON_ADD)` (filled by default),
/// or `ui.button(ICON_ADD.outlined())` for the outline variant.
#[derive(Clone, Copy, Debug)]
pub struct MaterialIcon {
    pub codepoint: &'static str,
    pub style: IconStyle,
}

impl MaterialIcon {
    /// Creates a new icon with the default style based on enabled features.
    ///
    /// - If `filled` is enabled, defaults to [`IconStyle::Filled`].
    /// - If only `outline` is enabled, defaults to [`IconStyle::Outlined`].
    pub const fn new(codepoint: &'static str) -> Self {
        #[cfg(feature = "filled")]
        {
            Self {
                codepoint,
                style: IconStyle::Filled,
            }
        }
        #[cfg(not(feature = "filled"))]
        {
            Self {
                codepoint,
                style: IconStyle::Outlined,
            }
        }
    }

    /// Returns this icon with the outlined style.
    #[cfg(feature = "outline")]
    pub const fn outlined(self) -> Self {
        Self {
            codepoint: self.codepoint,
            style: IconStyle::Outlined,
        }
    }

    /// Returns this icon with the filled style.
    #[cfg(feature = "filled")]
    pub const fn filled(self) -> Self {
        Self {
            codepoint: self.codepoint,
            style: IconStyle::Filled,
        }
    }

    /// Returns the [`FontFamily`] for this icon's style.
    pub fn font_family(&self) -> FontFamily {
        match self.style {
            IconStyle::Filled => FontFamily::Name(FONT_FAMILY.into()),
            IconStyle::Outlined => FontFamily::Name(FONT_FAMILY_OUTLINED.into()),
        }
    }

    /// Returns the icon as a [`RichText`] with the appropriate font family.
    pub fn rich_text(self) -> RichText {
        RichText::new(self.codepoint).family(self.font_family())
    }
}

impl From<MaterialIcon> for RichText {
    fn from(icon: MaterialIcon) -> Self {
        icon.rich_text()
    }
}

impl From<MaterialIcon> for egui::WidgetText {
    fn from(icon: MaterialIcon) -> Self {
        icon.rich_text().into()
    }
}

impl From<MaterialIcon> for &str {
    fn from(icon: MaterialIcon) -> Self {
        icon.codepoint
    }
}

impl From<MaterialIcon> for String {
    fn from(icon: MaterialIcon) -> Self {
        icon.codepoint.to_string()
    }
}

// =============================================================================
// Font registration
// =============================================================================

/// Creates a [`FontInsert`] for the material icons font.
pub fn font_insert() -> FontInsert {
    let mut data = FontData::from_static(&FONT_DATA);
    data.tweak.y_offset_factor = 0.05;

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
pub fn icon_button(ui: &mut egui::Ui, icon: MaterialIcon) -> Response {
    Frame::new()
        .show(ui, |ui| {
            Button::new(icon.rich_text().size(18.0)).frame(false).ui(ui)
        })
        .inner
}

/// Creates a [`RichText`] from an icon.
pub fn icon_text(icon: MaterialIcon) -> RichText {
    icon.rich_text()
}
