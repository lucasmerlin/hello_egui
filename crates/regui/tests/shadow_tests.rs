//! Tests for [`regui::effect::Shadow`], which throws a shadow from the child's own shape.

#![cfg(feature = "wgpu")]

use egui::{Color32, Rect, Ui, Vec2, pos2, vec2};
use egui_kittest::{
    Harness,
    wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup},
};
use regui::{Regui, effect::Shadow};
use std::cell::Cell;
use std::rc::Rc;

const SIZE: [f32; 2] = [320.0, 260.0];
const CHILD: Vec2 = vec2(140.0, 90.0);

/// Where the child sits, measured from the top left of the panel.
const MARGIN: f32 = 70.0;

/// Render a scene, sharing one render state between the harness and `install_wgpu`.
fn render(build: impl Fn(&mut Ui) + 'static) -> image::RgbaImage {
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
            // A pale background, so a dark shadow is easy to see and easy to measure.
            ui.painter()
                .rect_filled(ui.ctx().viewport_rect(), 0.0, Color32::from_gray(230));
            build(ui);
        });
    harness.run();
    harness.run();
    harness.render().expect("failed to render")
}

/// A scene holding one child at a fixed place, with whatever effects the caller adds.
///
/// The child is a solid block, so its shadow has a straight edge to throw and every pixel
/// outside it started out as background.
fn scene(
    build: impl Fn(Regui) -> Regui + 'static,
    rect: Rc<Cell<Rect>>,
) -> impl Fn(&mut Ui) + 'static {
    move |ui: &mut Ui| {
        ui.add_space(MARGIN);
        ui.horizontal(|ui| {
            ui.add_space(MARGIN);
            let child = Regui::new("child").size(CHILD).interactive(false);
            let output = build(child).show(ui, |ui| {
                let viewport = ui.ctx().viewport_rect();
                ui.painter()
                    .rect_filled(viewport, 0.0, Color32::from_rgb(40, 90, 160));
            });
            rect.set(output.response.rect);
        });
    }
}

/// How bright a pixel is, from 0 to 255.
fn luma(image: &image::RgbaImage, x: f32, y: f32) -> f64 {
    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let pixel = image.get_pixel(x.round() as u32, y.round() as u32);
    0.299 * f64::from(pixel[0]) + 0.587 * f64::from(pixel[1]) + 0.114 * f64::from(pixel[2])
}

/// Render the same child twice, with and without a shadow, and hand back both and the
/// child's rect.
fn with_and_without(
    shadow: impl Fn() -> Shadow + Copy + 'static,
) -> (image::RgbaImage, image::RgbaImage, Rect) {
    let rect = Rc::new(Cell::new(Rect::ZERO));
    let plain = render(scene(|child| child.offscreen(true), Rc::clone(&rect)));
    let shaded = render(scene(move |child| child.effect(shadow()), Rc::clone(&rect)));
    (plain, shaded, rect.get())
}

/// The background below the child goes dark, because that is where the shadow falls.
#[test]
fn the_shadow_darkens_the_background() {
    let (plain, shaded, child) = with_and_without(|| Shadow::new().radius(10.0).offset([0.0, 8.0]));

    // The shadow fades out as it goes, so each step down asks for less.
    let x = child.center().x;
    for (below, least) in [(3.0, 12.0), (8.0, 8.0), (12.0, 4.0)] {
        let y = child.bottom() + below;
        let (before, after) = (luma(&plain, x, y), luma(&shaded, x, y));
        assert!(
            after < before - least,
            "{below} points below the child the background is {after:.1} with a shadow and \
             {before:.1} without: the shadow is not there"
        );
    }
}

/// The shadow falls the way the offset points, and hardly at all the other way.
#[test]
fn the_offset_points_somewhere() {
    let (plain, shaded, child) = with_and_without(|| Shadow::new().radius(8.0).offset([0.0, 12.0]));

    let x = child.center().x;
    let below = luma(&plain, x, child.bottom() + 6.0) - luma(&shaded, x, child.bottom() + 6.0);
    let above = luma(&plain, x, child.top() - 6.0) - luma(&shaded, x, child.top() - 6.0);

    assert!(
        below > 4.0 * above.max(1.0),
        "the shadow darkened the pixels below the child by {below:.1} and the ones above by \
         {above:.1}: the offset is not pointing anywhere"
    );
}

/// Where the child is opaque, the child is all you see.
///
/// A shadow that shows through is a sign the composite is wrong: with premultiplied alpha
/// the child covers its own shadow completely.
#[test]
fn the_child_itself_is_unchanged() {
    let (plain, shaded, child) = with_and_without(|| Shadow::new().radius(12.0).offset([6.0, 6.0]));

    // Inside the child's edge, so antialiasing on the edge itself is not being measured.
    let inset = child.shrink(4.0);
    for point in [
        inset.center(),
        inset.left_top(),
        inset.right_bottom(),
        pos2(inset.center().x, inset.bottom()),
    ] {
        let (before, after) = (
            luma(&plain, point.x, point.y),
            luma(&shaded, point.x, point.y),
        );
        assert!(
            (after - before).abs() < 3.0,
            "the child's own pixels changed at {point:?}: {before:.1} became {after:.1}, so \
             the shadow is bleeding through where the child is opaque"
        );
    }
}

/// A bigger radius throws the shadow further out.
#[test]
fn a_bigger_radius_reaches_further() {
    /// How far below the child a pixel has to darken by to count as shaded.
    const THRESHOLD: f64 = 3.0;

    let reach = |radius: f32| {
        let (plain, shaded, child) =
            with_and_without(move || Shadow::new().radius(radius).offset([0.0, 0.0]));
        let x = child.center().x;
        #[expect(clippy::cast_precision_loss)]
        let bottom = plain.height() as f32 - 1.0;
        let mut reach = 0.0_f32;
        let mut below = 1.0_f32;
        while child.bottom() + below < bottom {
            let y = child.bottom() + below;
            if luma(&plain, x, y) - luma(&shaded, x, y) > THRESHOLD {
                reach = below;
            }
            below += 1.0;
        }
        reach
    };

    let (small, big) = (reach(6.0), reach(20.0));
    assert!(
        big > small + 4.0,
        "a radius of 20 reached {big} points past the child and a radius of 6 reached {small}: \
         the radius is not doing much"
    );
}

/// Spread grows the shadow, so the same blur covers more.
#[test]
fn spread_grows_the_shadow() {
    let darkening = |spread: f32| {
        let (plain, shaded, child) =
            with_and_without(move || Shadow::new().radius(12.0).offset([0.0, 0.0]).spread(spread));
        let (x, y) = (child.center().x, child.bottom() + 4.0);
        luma(&plain, x, y) - luma(&shaded, x, y)
    };

    let (tucked_in, grown) = (darkening(-4.0), darkening(6.0));
    assert!(
        grown > tucked_in + 5.0,
        "a spread of 6 darkened the background by {grown:.1} and a spread of -4 by \
         {tucked_in:.1}: spread is not doing much"
    );
}

/// A shadow under a circle: the shape a rectangle shadow cannot do.
#[test]
fn snapshot_shadow_circle() {
    snapshot("shadow_circle", |ui| {
        let painter = ui.painter();
        let rect = ui.ctx().viewport_rect();
        painter.circle_filled(
            rect.center(),
            rect.height() * 0.4,
            Color32::from_rgb(230, 120, 60),
        );
    });
}

/// A shadow under two separated widgets, each of which throws its own.
#[test]
fn snapshot_shadow_widgets() {
    snapshot("shadow_widgets", |ui| {
        ui.horizontal(|ui| {
            let _ = ui.button("left");
            ui.add_space(40.0);
            let _ = ui.button("right");
        });
        ui.add_space(24.0);
        ui.label("and a line of text");
    });
}

/// Render one child with a shadow over a pale background and compare it with the snapshot.
fn snapshot(name: &str, child: impl Fn(&mut Ui) + 'static) {
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
            ui.painter()
                .rect_filled(ui.ctx().viewport_rect(), 0.0, Color32::from_gray(230));
            ui.add_space(MARGIN);
            ui.horizontal(|ui| {
                ui.add_space(MARGIN);
                Regui::new("child")
                    .size(CHILD)
                    .interactive(false)
                    .effect(
                        Shadow::new()
                            .radius(14.0)
                            .offset([4.0, 8.0])
                            .color(Color32::from_black_alpha(140)),
                    )
                    .show(ui, &child);
            });
        });
    harness.run();
    harness.run();
    harness.snapshot(name);
}
