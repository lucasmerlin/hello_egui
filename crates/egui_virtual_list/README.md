# egui_virtual_list

[![egui_ver](https://img.shields.io/badge/egui-0.31.0-blue)](https://github.com/emilk/egui)
[![Latest version](https://img.shields.io/crates/v/egui_virtual_list.svg)](https://crates.io/crates/egui_virtual_list)
[![Documentation](https://docs.rs/egui_virtual_list/badge.svg)](https://docs.rs/egui_virtual_list)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/egui_virtual_list.svg)](https://crates.io/crates/egui_virtual_list)



[content]:<>


This crate adds a virtual list widget to [egui](https://github.com/emilk/egui).
Egui has a basic build in virtual list in the
[ScrollArea](https://docs.rs/egui/0.25.0/egui/containers/scroll_area/struct.ScrollArea.html#method.show_rows) widget.
This crate has some extra features though:

- Supports items with varying heights
    - Heights are calculated lazily and cached, as you scroll further down the list
- Supports custom layouts, so you could place multiple items in a single row
    - Check the [Gallery Example](https://lucasmerlin.github.io/hello_egui/#/example/gallery)
- Allows for adding items at the top without the scroll position changing
    - Check the [Chat Example](https://lucasmerlin.github.io/hello_egui/#/example/chat)

There are some limitations though:

- If you want to support a crazy amounts of items (1000000+ items), where you can instantly jump anywhere in the list,
  I recommend using egui's built in ScrollArea instead.
- Horizontal scrolling is not supported yet, but it should be easy to add if needed.

If you want to build a infinite scroll list, I recommend using
the [egui_infinite_scroll](https://crates.io/crates/egui_infinite_scroll) crate instead, which
is using this crate internally.
