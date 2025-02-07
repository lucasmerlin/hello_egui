# Hello Egui!

This project contains a collection of egui tools I've created during
development of the native app for <https://hellopaint.io> (still unreleased, stay tuned!).

The crates have varying levels of maturity, some are ready for use in production
while others are highly experimental.
If you're interested in using one of the experimental crates, open an issue, and I'll try to
release it on crates.io.

Join the [hello_egui channel on the egui](https://discord.gg/MSftBbKSYm) discord for questions and discussions.

## Example app

An example using most of the crates is available [here](https://lucasmerlin.github.io/hello_egui/).
Source code in [fancy-example](fancy-example).

## [**hello_egui**](https://crates.io/crates/hello_egui), this crate

A collection of reexports for the other crates, if you want to use all or most of them.
You can toggle individual features to only include the crates you need. By default, all crates are included.
Only includes crates that have been released on [crates.io](https://crates.io/).

## **Mature** Crates

- [egui_dnd](crates/egui_dnd)
    - Drag & drop sorting library for egui
    - released on [crates.io](https://crates.io/crates/egui_dnd)

- [egui_flex](crates/egui_flex)
    - Flexbox layout for egui
    - released on [crates.io](https://crates.io/crates/egui_flex)

- [egui_inbox](crates/egui_inbox)
    - Simple utility for sending messages to egui uis from other threads / async functions
    - released on [crates.io](https://crates.io/crates/egui_inbox)

- [egui_virtual_list](crates/egui_virtual_list)
    - Flexible virtual scroll widget for egui with support for dynamic heights and complex layouts
    - Compatible with [egui_dnd](crates/egui_dnd) (let me know if you need an example)
    - released on [crates.io](https://crates.io/crates/egui_virtual_list)

- [egui_infinite_scroll](crates/egui_infinite_scroll)
    - Infinite scroll based on [egui_virtual_list](crates/egui_virtual_list)
    - released on [crates.io](https://crates.io/crates/egui_infinite_scroll)

- [egui_router](crates/egui_router)
    - A SPA router for egui with support for transitions
    - released on [crates.io](https://crates.io/crates/egui_router)

- [egui_form](crates/egui_form)
    - Form validation for egui
    - released on [crates.io](https://crates.io/crates/egui_form)

- [egui_pull_to_refresh](crates/egui_pull_to_refresh)
    - Adds pull to refresh functionality to egui.
    - released on [crates.io](https://crates.io/crates/egui_pull_to_refresh)

- [egui_suspense](crates/egui_suspense)
    - A helper to display loading, error and retry uis when waiting for asynchronous data.
    - released on [crates.io](https://crates.io/crates/egui_suspense)

- [egui_thumbhash](crates/egui_thumbhash)
    - Easily use [thumbhashes](https://evanw.github.io/thumbhash/) in egui.
    - For a showcase, check the [gallery example](https://lucasmerlin.github.io/hello_egui/#/example/gallery).
    - released on [crates.io](https://crates.io/crates/egui_thumbhash)

- [egui_material_icons](crates/egui_material_icons)
    - Material icons for egui
    - released on [crates.io](https://crates.io/crates/egui_material_icons)

## **Experimental** Crates

- [egui_animation](crates/egui_animation)
    - Animation utilities for egui
    - Experimental, released on [crates.io](https://crates.io/crates/egui_animation), used internally
      by [egui_dnd](crates/egui_dnd)

- [egui_taffy](crates/egui_taffy)
    - Adds flexbox layout to egui using [taffy](https://github.com/DioxusLabs/taffy)
    - Highly experimental, unreleased

- [egui_webview](crates/egui_webview)
    - WebView widget for egui, based on wry
    - Experimental, unreleased
    - Warning: Currently uses some unsafe to get around Send / Sync limitations,
      so it probably has some safety issues.

- [perfect_cursors](crates/perfect_cursors)
    - A port of steve ruiz's [perfect cursors](https://github.com/steveruizok/perfect-cursors) to rust
    - independent of egui, but there is a egui example

- [hello_egui_utils](crates/hello_egui_utils)
    - Collection of utilities used by the other crates
