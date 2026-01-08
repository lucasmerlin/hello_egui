# egui_material_icons

[![egui_ver](https://img.shields.io/badge/egui-0.33.0-blue)](https://github.com/emilk/egui)
[![Latest version](https://img.shields.io/crates/v/egui_material_icons.svg)](https://crates.io/crates/egui_material_icons)
[![Documentation](https://docs.rs/egui_material_icons/badge.svg)](https://docs.rs/egui_material_icons)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/egui_material_icons.svg)](https://crates.io/crates/egui_material_icons)

[content]:<>

Provides material icons (now material symbols) for egui.

example:

```no_build
// register the fonts:
egui_material_icons::initialize(&cc.egui_ctx);

// later in some ui:
ui.button(egui_material_icons::icons::ICON_ADD);
```

Currently, this provides the rounded icons. By default, the filled variant is used.

## Features

- **`compressed`** (default) - Compress embedded fonts with DEFLATE, reducing binary size significantly.

- **`outline`** - Include both filled and outline fonts. Adds `ICON_OUTLINE_*` constants:

  ```rust
  use egui_material_icons::icons::*;

  egui_material_icons::initialize(&cc.egui_ctx); // registers both fonts

  // Use filled (default)
  ui.button(ICON_ADD);

  // Use outlined
  ui.button(ICON_OUTLINE_ADD);
  ```

- **`outline-only`** - Use the outline font instead of the filled font. The default `ICON_*` constants will render as outline icons. Saves binary size if you only need outline icons.
