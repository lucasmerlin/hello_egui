//! Rendering tests for [`regui::BackdropBlur`].
//!
//! These need the `wgpu` feature and a GPU, like the rest of the snapshot tests.

#![cfg(feature = "wgpu")]

use egui::{Color32, Pos2, Rect, Ui, UiBuilder, vec2};
use egui_kittest::{
    Harness,
    wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup},
};
use regui::BackdropBlur;
use std::cell::Cell;
use std::rc::Rc;

const SIZE: [f32; 2] = [240.0, 180.0];

/// Hard edges in both directions, so that both halves of the separable blur have something
/// to do and a mistake in either one shows up.
fn checkerboard(ui: &Ui, rect: Rect) {
    const CELL: f32 = 8.0;

    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, Color32::BLACK);

    let mut row = 0;
    let mut y = rect.top();
    while y < rect.bottom() {
        let mut column = 0;
        let mut x = rect.left();
        while x < rect.right() {
            if (row + column) % 2 == 0 {
                painter.rect_filled(
                    Rect::from_min_size(Pos2::new(x, y), vec2(CELL, CELL)),
                    0.0,
                    Color32::WHITE,
                );
            }
            column += 1;
            x += CELL;
        }
        row += 1;
        y += CELL;
    }
}

/// A harness that draws a checkerboard with a blurred panel on top of it.
///
/// The harness renders with the same [`egui_wgpu::RenderState`] that `install_wgpu` is
/// given, so that the blur's textures and egui's live on the same device.
///
/// A `radius` of zero switches the blur off, which is how the test below gets an otherwise
/// identical image to compare against.
fn harness(radius: f32, panel_rect: Rc<Cell<Rect>>) -> Harness<'static> {
    harness_with(radius, panel_rect, 0, Color32::TRANSPARENT)
}

/// As above, but with rounded corners and a tint.
fn harness_with(
    radius: f32,
    panel_rect: Rc<Cell<Rect>>,
    corner_radius: u8,
    tint: Color32,
) -> Harness<'static> {
    let render_state = create_render_state(
        default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );
    let installed = render_state.clone();

    Harness::builder()
        .with_size(SIZE)
        .renderer(WgpuTestRenderer::from_render_state(render_state))
        .build_ui(move |ui| {
            regui::install_wgpu(ui.ctx(), installed.clone());

            let rect = ui.max_rect();
            checkerboard(ui, rect);

            let panel = Rect::from_min_size(rect.min + vec2(40.0, 40.0), vec2(160.0, 100.0));
            panel_rect.set(panel);
            let mut panel_ui = ui.new_child(UiBuilder::new().max_rect(panel));
            BackdropBlur::new(radius)
                .corner_radius(corner_radius)
                .tint(tint)
                .paint_at(&panel_ui, panel);
            panel_ui.label("on glass");
        })
}

/// Render the scene, and report where the panel ended up.
fn render(radius: f32) -> (image::RgbaImage, Rect) {
    let panel_rect = Rc::new(Cell::new(Rect::ZERO));
    let mut harness = harness(radius, Rc::clone(&panel_rect));
    harness.run();
    let image = harness.render().expect("failed to render");
    (image, panel_rect.get())
}

/// A blur radius above zero has to change the pixels behind the panel, and only those.
#[test]
fn the_blur_changes_the_background_behind_the_panel() {
    let (sharp, _) = render(0.0);
    let (blurred, panel) = render(14.0);

    assert_eq!(sharp.dimensions(), blurred.dimensions());
    assert!(
        sharp != blurred,
        "the blur did not change anything, so it never ran"
    );

    // The stripes are pure black and white, so any pixel that is neither is one the blur
    // mixed. Counting them tells us we are seeing an actual blur, rather than the region
    // being replaced by a single flat colour.
    let mixed = blurred
        .pixels()
        .filter(|pixel| {
            let [r, g, b, _] = pixel.0;
            let grey = r.max(g).max(b);
            (8..248).contains(&grey)
        })
        .count();
    assert!(
        mixed > 500,
        "expected many blended pixels from the blur, found {mixed}"
    );

    // Outside the panel the image must be untouched. The panel starts at (40, 40), so the
    // top left corner is well clear of it.
    for position in [(2, 2), (2, 20), (20, 2)] {
        assert_eq!(
            sharp.get_pixel(position.0, position.1),
            blurred.get_pixel(position.0, position.1),
            "the blur leaked outside the rect it was given, at {position:?}"
        );
    }

    // The whole rect must be blurred, corners included, so check two pixels in from each
    // corner of wherever the panel actually landed.
    let inset = 2.0;
    let corners = [
        panel.left_top() + vec2(inset, inset),
        panel.right_top() + vec2(-inset, inset),
        panel.left_bottom() + vec2(inset, -inset),
        panel.right_bottom() + vec2(-inset, -inset),
    ];
    for corner in corners {
        let position = (corner.x as u32, corner.y as u32);
        assert_ne!(
            sharp.get_pixel(position.0, position.1),
            blurred.get_pixel(position.0, position.1),
            "the corner at {position:?} was left sharp, so the blur did not cover the whole rect"
        );
    }
}

/// Rounded corners let the sharp background show through at the corners, and the tint
/// fades the blur towards white.
#[test]
fn snapshot_backdrop_blur_rounded() {
    let mut harness = harness_with(
        14.0,
        Rc::new(Cell::new(Rect::ZERO)),
        24,
        Color32::from_white_alpha(70),
    );
    harness.run();
    harness.snapshot("backdrop_blur_rounded");
}

#[test]
fn snapshot_backdrop_blur() {
    let mut harness = harness(14.0, Rc::new(Cell::new(Rect::ZERO)));
    harness.run();
    harness.snapshot("backdrop_blur");
}
