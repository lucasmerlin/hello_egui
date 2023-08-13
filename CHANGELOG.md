# Changelog

## v0.5.0
 - Added animations
 - Dragging in a ScrollArea will now scroll if we are near the edge
 - Added support for dragging in a ScrollArea on touch devices (by pressing and holding until the scroll is canceled and the drag starts)
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
