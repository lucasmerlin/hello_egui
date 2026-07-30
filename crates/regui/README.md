# regui

[![egui_ver](https://img.shields.io/badge/egui-0.35.0-blue)](https://github.com/emilk/egui)
[![Latest version](https://img.shields.io/crates/v/regui.svg)](https://crates.io/crates/regui)
[![Documentation](https://docs.rs/regui/badge.svg)](https://docs.rs/regui)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/regui.svg)](https://crates.io/crates/regui)



[content]:<>

**re**tained **egui** — render an egui ui inside another egui ui.

`regui` runs a part of your ui in its own egui viewport and paints the result into the
parent ui. Because the child is a real viewport, you can move, scale and rotate it, and
with the `wgpu` feature you can run shaders over it — blur a dialog's backdrop, fade a
panel out, or shrink a whole screen into a preview thumbnail.

The child shares the parent's `Context`, so it shares memory, style and fonts. It looks
and behaves like the rest of your app, but it gets its own input, its own hit-testing and
its own focus.

```rust
# egui::__run_test_ui(|ui| {
use regui::Regui;

Regui::new("preview")
    .size(egui::vec2(320.0, 240.0))
    .scale(0.5)
    .show(ui, |ui| {
        ui.heading("I am half the size");
        ui.button("...and still clickable").clicked();
    });
# });
```

## Backdrop blur

With the `wgpu` feature, `BackdropBlur` blurs whatever is already behind it — the frosted
glass under a dialog or a sidebar:

```rust
# egui::__run_test_ui(|ui| {
# #[cfg(feature = "wgpu")]
regui::BackdropBlur::new(12.0).show(ui, |ui| {
    ui.heading("On frosted glass");
});
# });
```

Call `regui::install_wgpu(&cc.egui_ctx, cc.wgpu_render_state.clone().unwrap())` once at
startup so it can reach the device.

egui draws in order, so by the time it reaches the widget the background is already drawn.
Nothing can sample the texture it is drawing into, so egui-wgpu interrupts its render pass,
copies the half-drawn frame aside, blurs it, and draws the result back. The blur therefore
shows exactly what is behind it, with no guessing and no one-frame lag — but it costs a pass
split and a full-screen copy each, so use a handful rather than dozens.

This needs `CallbackTrait::needs_backdrop`, which is not in a released egui-wgpu yet.

## How it renders

`Regui` tessellates the child itself and hands the triangles to the parent's painter, so it
works with any egui backend — no wgpu needed. An untransformed child comes out pixel for
pixel identical to the same ui drawn straight into the parent.

## Caveats

- Clipping is axis-aligned, because egui's clip rectangles are. Rotate a child and its
  clip rectangles grow to their bounding boxes, so content that should be cut off at the
  edge of a scroll area can spill a little.
- Child popups and tooltips cannot leave the child's rect: all of the child's shapes go
  into a single parent layer.
- The child is not in the parent's accessibility tree.
- A paint callback inside a rotated child is dropped, since a paint callback draws into an
  axis-aligned rectangle of the screen.
- Needs `egui::Context::run_hosted_viewport`, which is not in a released egui yet.
