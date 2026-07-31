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
    harness_with(radius, panel_rect, 0, Some(Color32::TRANSPARENT), 0.0)
}

/// As above, but with rounded corners, a tint, and a feathered edge.
fn harness_with(
    radius: f32,
    panel_rect: Rc<Cell<Rect>>,
    corner_radius: u8,
    // `None` leaves `BackdropBlur` to pick the theme's default.
    tint: Option<Color32>,
    feather: f32,
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
            let mut blur = BackdropBlur::new(radius)
                .corner_radius(corner_radius)
                .feather(feather);
            if let Some(tint) = tint {
                blur = blur.tint(tint);
            }
            blur.paint_at(&panel_ui, panel);
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
        Some(Color32::from_white_alpha(70)),
        0.0,
    );
    harness.run();
    harness.snapshot("backdrop_blur_rounded");
}

/// With no tint given, the blur is faded towards the theme's own window fill, so the glass
/// matches the rest of the app and content on it stays readable.
#[test]
fn snapshot_backdrop_blur_default_tint() {
    let mut harness = harness_with(14.0, Rc::new(Cell::new(Rect::ZERO)), 16, None, 0.0);
    harness.run();
    harness.snapshot("backdrop_blur_default_tint");
}

#[test]
fn snapshot_backdrop_blur() {
    let mut harness = harness(14.0, Rc::new(Cell::new(Rect::ZERO)));
    harness.run();
    harness.snapshot("backdrop_blur");
}

/// A feathered edge fades out across the rect's edge instead of stopping at it, so half of
/// the fade lands outside the rect and there is no step where the glass ends.
#[test]
fn the_feathered_edge_reaches_outside_the_rect() {
    const FEATHER: f32 = 16.0;

    let render = |feather: f32| {
        let panel_rect = Rc::new(Cell::new(Rect::ZERO));
        let mut harness = harness_with(
            14.0,
            Rc::clone(&panel_rect),
            0,
            Some(Color32::TRANSPARENT),
            feather,
        );
        harness.run();
        let image = harness.render().expect("failed to render");
        (image, panel_rect.get())
    };

    let (hard, panel) = render(0.0);
    let (soft, _) = render(FEATHER);
    let pixels_per_point = hard.width() as f32 / SIZE[0];

    // Walk out from the middle of the panel's left edge. Just outside it the hard edge has
    // left the checkerboard alone and the feather has not, and far enough out both are back
    // to the untouched background.
    let sample = |image: &image::RgbaImage, offset: f32| {
        let point = panel.left_center() + vec2(offset, 0.0);
        *image.get_pixel(
            (point.x * pixels_per_point) as u32,
            (point.y * pixels_per_point) as u32,
        )
    };

    let outside = -FEATHER / 4.0;
    assert_ne!(
        sample(&hard, outside),
        sample(&soft, outside),
        "the feather did not spread outside the rect, so its outer half was clipped away"
    );

    let well_outside = -FEATHER;
    assert_eq!(
        sample(&hard, well_outside),
        sample(&soft, well_outside),
        "the feather spread further than it was asked to"
    );

    // Inside the edge the feather is still fading in, so it cannot match the hard edge's
    // fully opaque glass either.
    let inside = FEATHER / 4.0;
    assert_ne!(
        sample(&hard, inside),
        sample(&soft, inside),
        "the feather did not soften the inside of the edge"
    );
}

/// What a feathered edge looks like: the glass has no outline of its own, it just thins out.
#[test]
fn snapshot_backdrop_blur_feathered() {
    let mut harness = harness_with(14.0, Rc::new(Cell::new(Rect::ZERO)), 16, None, 24.0);
    harness.run();
    harness.snapshot("backdrop_blur_feathered");
}

/// A window's frame reserves its shape slot before the body runs, so a blur added from
/// inside the body sits on top of the frame's fill. `show_window` claims the first slot in
/// the window's layer instead, so the frame lands on top of the blur.
///
/// The window frame has a visible stroke, so if the blur were painted over it the stroke
/// would be gone.
#[test]
fn show_window_puts_the_blur_under_the_window_frame() {
    let stroke_pixels = |blur_first: bool| {
        let render_state = create_render_state(
            default_wgpu_setup(),
            egui_wgpu::RendererOptions::PREDICTABLE,
        );
        let installed = render_state.clone();

        let mut harness = Harness::builder()
            .with_size(SIZE)
            .renderer(WgpuTestRenderer::from_render_state(render_state))
            .build_ui(move |ui| {
                regui::install_wgpu(ui.ctx(), installed.clone());
                checkerboard(ui, ui.max_rect());

                let frame = egui::Frame::window(ui.style())
                    .fill(Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(3.0, Color32::RED));
                let blur = BackdropBlur::new(10.0).corner_radius(0);
                let window = egui::Window::new("w")
                    .title_bar(false)
                    .frame(frame)
                    .default_pos(egui::pos2(40.0, 40.0))
                    .resizable(false);

                if blur_first {
                    blur.show_window(ui, egui::Id::new("w"), window, |ui| {
                        ui.label("on glass");
                    });
                } else {
                    // The old way: the blur goes over the frame and hides the stroke.
                    window.show(ui, |ui| {
                        blur.paint_at(ui, ui.max_rect().expand(8.0));
                        ui.label("on glass");
                    });
                }
            });

        harness.run();
        harness.run();
        let image = harness.render().expect("failed to render");
        image
            .pixels()
            .filter(|pixel| {
                let [r, g, b, _] = pixel.0;
                r > 150 && g < 90 && b < 90
            })
            .count()
    };

    let with_show_window = stroke_pixels(true);
    let with_blur_on_top = stroke_pixels(false);

    assert!(
        with_show_window > 100,
        "the window's red stroke should survive, found {with_show_window} red pixels"
    );
    assert!(
        with_show_window > with_blur_on_top * 2,
        "show_window should keep much more of the stroke than blurring over it \
         ({with_show_window} vs {with_blur_on_top} red pixels)"
    );
}

#[test]
fn snapshot_backdrop_blur_window() {
    let render_state = create_render_state(
        default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );
    let installed = render_state.clone();

    let mut harness = Harness::builder()
        .with_size([260.0, 200.0])
        .renderer(WgpuTestRenderer::from_render_state(render_state))
        .build_ui(move |ui| {
            regui::install_wgpu(ui.ctx(), installed.clone());
            checkerboard(ui, ui.max_rect());

            let frame = egui::Frame::window(ui.style()).fill(Color32::TRANSPARENT);
            BackdropBlur::new(12.0)
                .corner_radius(frame.corner_radius)
                .show_window(
                    ui,
                    egui::Id::new("w"),
                    egui::Window::new("w")
                        .title_bar(false)
                        .frame(frame)
                        .fixed_pos(egui::pos2(40.0, 50.0))
                        .resizable(false),
                    |ui| {
                        ui.label("on frosted glass");
                        ui.label("the frame is drawn on top");
                    },
                );
        });

    harness.run();
    harness.run();
    harness.snapshot("backdrop_blur_window");
}

/// Two blurred windows must both blur.
///
/// Every blur in a pass shares one `BlurResources`, since callback resources are keyed by
/// type. Sharing the shader uniform buffers meant the last window's settings overwrote the
/// first's, so the first drew with the wrong rect and vanished.
#[test]
fn two_windows_both_get_blurred() {
    let blurred_pixels = |count: usize| {
        let render_state = create_render_state(
            default_wgpu_setup(),
            egui_wgpu::RendererOptions::PREDICTABLE,
        );
        let installed = render_state.clone();

        let mut harness = Harness::builder()
            .with_size([320.0, 240.0])
            .renderer(WgpuTestRenderer::from_render_state(render_state))
            .build_ui(move |ui| {
                regui::install_wgpu(ui.ctx(), installed.clone());
                checkerboard(ui, ui.max_rect());

                for index in 0..count {
                    let frame = egui::Frame::window(ui.style()).fill(Color32::TRANSPARENT);
                    BackdropBlur::new(10.0).corner_radius(0).show_window(
                        ui,
                        egui::Id::new(("w", index)),
                        egui::Window::new(format!("w{index}"))
                            .title_bar(false)
                            .frame(frame)
                            .fixed_pos(egui::pos2(20.0, 20.0 + 100.0 * index as f32)),
                        |ui| {
                            ui.label("on glass");
                        },
                    );
                }
            });

        harness.run();
        harness.run();
        let image = harness.render().expect("failed to render");
        // The checkerboard is pure black and white, so a mid grey is a blurred pixel.
        image
            .pixels()
            .filter(|pixel| {
                let [r, g, b, _] = pixel.0;
                let grey = r.max(g).max(b);
                (40..220).contains(&grey)
            })
            .count()
    };

    let one = blurred_pixels(1);
    let two = blurred_pixels(2);

    assert!(one > 500, "one window should blur something, found {one}");
    assert!(
        two > one * 3 / 2,
        "the second window should blur roughly as much again, but {two} is not much more \
         than {one}: one of the two blurs is missing"
    );
}
