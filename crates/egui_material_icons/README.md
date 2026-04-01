# egui_material_icons

[![egui_ver](https://img.shields.io/badge/egui-0.34.0-blue)](https://github.com/emilk/egui)
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

| Features                                            | Fonts Included   |
| --------------------------------------------------- | ---------------- |
| default (`filled`)                                  | Filled only      |
| `--features outline`                                | Filled + Outline |
| `--no-default-features --features outline`          | Outline only     |
| `--no-default-features --features "filled outline"` | Both             |

- **`filled`** (default) - Include the filled font variant.

- **`outline`** - Include the outline font variant.

  ```rust
  use egui_material_icons::icons::*;

  fn init(ctx: &egui::Context) {
    egui_material_icons::initialize(ctx);
  }

  fn my_ui(ui: &mut egui::Ui) {
    ui.button(ICON_ADD);          // filled
    ui.button(ICON_ADD.outlined());  // outlined
  }
  ```

- **`compressed`** - Compress embedded fonts with DEFLATE at compile time, decompressing at runtime.
  This can reduce binary size for native targets, but is **not recommended for wasm**: transport
  compression (gzip/brotli) is more effective and the decompression runtime (zstd + libflate) adds
  ~250 KB of code to the wasm binary, resulting in a larger final download.
