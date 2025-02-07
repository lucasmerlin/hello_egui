# egui_thumbhash

[![egui_ver](https://img.shields.io/badge/egui-0.31.0-blue)](https://github.com/emilk/egui)
[![Latest version](https://img.shields.io/crates/v/egui_thumbhash.svg)](https://crates.io/crates/egui_thumbhash)
[![Documentation](https://docs.rs/egui_thumbhash/badge.svg)](https://docs.rs/egui_thumbhash)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/egui_thumbhash.svg)](https://crates.io/crates/egui_thumbhash)



[content]:<>


This crate adds an [image loader](https://docs.rs/egui/latest/egui/load/index.html)
to easily use [thumbhashes](https://evanw.github.io/thumbhash/) in egui.
It also includes a ThumbhashImage widget to easily display the thumbhash while the image is loading.

Internally we use [the thumbhash crate](https://crates.io/crates/thumbhash) to load the images.

For a showcase, check the [gallery example](https://lucasmerlin.github.io/hello_egui/#/example/gallery).