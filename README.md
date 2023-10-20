# Hello Egui!
This projects contains a collection of egui tools I've created during
development of the native app for https://hellopaint.io (still unreleased, stay tuned!).

The crates have varying levels of maturity, some are ready for use in production
while others are highly experimental.
If you're interested in using one of the experimental crates, open an issue, and I'll try to
release it on crates.io.

## Example app
An example using most of the crates is available [here](https://lucasmerlin.github.io/hello_egui/).
Source code in [fancy-example](fancy-example).

## **Mature** Crates
- [egui_dnd](crates/egui_dnd)
  - Drag & drop sorting library for egui
  - released on [crates.io](https://crates.io/crates/egui_dnd)

- [egui_inbox](crates/egui_inbox)
  - Simple utility for sending messages to egui uis from other threads / async functions
  - released on [crates.io](https://crates.io/crates/egui_inbox)

- [egui_pull_to_refresh](crates/egui_pull_to_refresh)
  - Adds pull to refresh functionality to egui.
  - released on [crates.io](https://crates.io/crates/egui_pull_to_refresh)

## **Experimental** Crates

- [egui_virtual_list](crates/egui_virtual_list)
  - Flexible virtual scroll widget for egui with support for dynamic heights and complex layouts
  - Compatible with [egui_dnd](crates/egui_dnd) (let me know if you need an example)
  - Experimental, unreleased

- [egui_infinite_scroll](crates/egui_infinite_scroll)
  - Infinite scroll based on [egui_virtual_list](crates/egui_virtual_list)
  - Experimental, unreleased

- [egui_animation](crates/egui_animation)
  - Animation utilities for egui
  - Experimental, released on [crates.io](https://crates.io/crates/egui_animation), used internally by [egui_dnd](crates/egui_dnd)

- [egui_taffy](crates/egui_taffy)
  - Adds flexbox layout to egui using [taffy](https://github.com/DioxusLabs/taffy)
  - Highly experimental, unreleased

- [hello_egui_utils](crates/hello_egui_utils)
  - Collection of utilities used by the other crates
