# Changelog

## 0.12.0

- Update egui to 0.31

## 0.11.0

- update egui to 0.30

## 0.10.0

- update egui to 0.29

## 0.9.1

- make ItemIterator public
- change area order of dragged item to be `Order::Tooltip`, so it will be shown above any other areas

## 0.9.0

- update egui to 0.28

## 0.8.0

- update egui to 0.27

## v0.7.0

- Updated to egui 0.26.0
- By default, egui_dnd will disable egui's new text selection within the drag handle, so it doesn't interfere
  with dragging.
- There is a new function on the handle, `enable_selectable_labels`, that will make egui_dnd not disable text selection.

## v0.6.0

- Updated to egui 0.25.0, dropping support for older versions.

## v0.5.1

- Added setting to configure animation duration for swap and return animations
- Add support for egui 0.23

## v0.5.0

- Added animations
- Dragging in a ScrollArea will now scroll if we are near the edge
- Added support for dragging in a ScrollArea on touch devices (by pressing and holding until the scroll is canceled and
  the drag starts)
- Fixed bug where offset was wrong when the handle isn't the first element
- Allow the handle or buttons in the handle to be clicked
- Complete refactor, with much better state and detection handling
- Added `dnd` function that stores and reads the state from egui data, for simpler usage:

```rust
pub fn main() -> eframe::Result<()> {
  let mut items = vec!["alfred", "bernhard", "christian"];
  eframe::run_simple_native("DnD Simple Example", Default::default(), move |ctx, _frame| {
    CentralPanel::default().show(ctx, |ui| {
      dnd(ui, "dnd_example")
        .show_vec(&mut items, |ui, item, handle, state| {
          handle.ui(ui, |ui| {
            ui.label("drag");
          });
          ui.label(**item);
        });
    });
  })
}
```

- **Breaking**: Removed DragDropUi in favor of dnd function
- Made the drag cursor when hovering over a handle configurable
- Fixed ui having unexpected margins
- Added support for sorting in horizontal and horizontal wrapping layouts
- Improved fancy example
- Improved sorting snappiness
