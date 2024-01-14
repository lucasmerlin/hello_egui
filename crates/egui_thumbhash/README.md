# egui_thumbhash

This crate adds an [image loader](https://docs.rs/egui/latest/egui/load/index.html) 
to easily use [thumbhashes](https://evanw.github.io/thumbhash/) in egui.
It also includes a ThumbhashImage widget to easily display the thumbhash while the image is loading.

Internally we use [the thumbhash crate](https://crates.io/crates/thumbhash) to load the images.

For a showcase, check the [gallery example](https://lucasmerlin.github.io/hello_egui/).