//! Tests for `Regui::offscreen` and `Regui::blur`, which render the child through a texture.

#![cfg(feature = "wgpu")]

use egui::{Color32, Rect, Slider, Ui, vec2};
use egui_kittest::{
    Harness,
    wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup},
};
use regui::Regui;

const SIZE: [f32; 2] = [320.0, 220.0];
const CHILD: egui::Vec2 = vec2(240.0, 150.0);

/// The child ui under test. Text, a stroke and a rounded frame, since those are what a trip
/// through a texture is most likely to change.
fn child(ui: &mut Ui, value: &mut f32) {
    // Fill the child so the difference between the two paths is not just antialiasing on a
    // transparent background.
    let rect = ui.ctx().viewport_rect();
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_rgb(30, 40, 60));
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.heading("Heading");
        ui.label("Some text that has to survive the trip.");
        ui.add(Slider::new(value, 0.0..=1.0).text("value"));
        let _ = ui.button("A button");
    });
}

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
            build(ui);
        });
    harness.run();
    harness.run();
    harness.render().expect("failed to render")
}

/// How far apart two images are, as a fraction of channel values.
fn difference(a: &image::RgbaImage, b: &image::RgbaImage) -> f64 {
    assert_eq!(a.dimensions(), b.dimensions());
    let total: u64 = a
        .pixels()
        .zip(b.pixels())
        .map(|(left, right)| {
            left.0
                .iter()
                .zip(right.0.iter())
                .map(|(l, r)| u64::from(l.abs_diff(*r)))
                .sum::<u64>()
        })
        .sum();
    total as f64 / (f64::from(a.width()) * f64::from(a.height()) * 4.0 * 255.0)
}

/// Going through a texture must look the same as handing the parent triangles.
///
/// This is the check that the off-screen path gets the colour space right: an image
/// rendered into a texture and then sampled back has two chances to convert wrongly, and a
/// mistake shows up as everything being too dark or too bright.
#[test]
fn offscreen_looks_like_the_shape_backend() {
    let shapes = render(|ui| {
        let mut value = 0.5;
        Regui::new("child")
            .size(CHILD)
            .show(ui, |ui| child(ui, &mut value));
    });
    let offscreen = render(|ui| {
        let mut value = 0.5;
        Regui::new("child")
            .size(CHILD)
            .offscreen(true)
            .show(ui, |ui| child(ui, &mut value));
    });

    let delta = difference(&shapes, &offscreen);
    assert!(
        delta < 0.01,
        "the off-screen child differs from the shape backend by {delta:.4} per channel, \
         which is far more than antialiasing: the colour space is probably wrong"
    );

    // ...but it must actually have drawn something, or the comparison is meaningless.
    let blank = render(|_ui| {});
    assert!(
        difference(&shapes, &blank) > 0.02,
        "the child drew nothing, so there was nothing to compare"
    );
}

/// Blurring the child changes its content, and only inside the child's rect.
#[test]
fn blur_changes_the_child() {
    let sharp = render(|ui| {
        let mut value = 0.5;
        Regui::new("child")
            .size(CHILD)
            .offscreen(true)
            .show(ui, |ui| child(ui, &mut value));
    });
    let blurred = render(|ui| {
        let mut value = 0.5;
        Regui::new("child")
            .size(CHILD)
            .blur(6.0)
            .show(ui, |ui| child(ui, &mut value));
    });

    assert!(
        difference(&sharp, &blurred) > 0.001,
        "the blur did not change the child at all"
    );

    // Well outside the child, which starts at the panel's top left.
    let (width, height) = sharp.dimensions();
    for position in [(width - 3, height - 3), (width - 3, 3)] {
        assert_eq!(
            sharp.get_pixel(position.0, position.1),
            blurred.get_pixel(position.0, position.1),
            "the child's blur leaked outside its rect, at {position:?}"
        );
    }
}

#[test]
fn snapshot_offscreen_blur() {
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
            let mut value = 0.5;
            Regui::new("child")
                .size(CHILD)
                .blur(14.0)
                .interactive(false)
                .show(ui, |ui| child(ui, &mut value));
        });
    harness.run();
    harness.run();
    harness.snapshot("offscreen_blur");
}

/// A rotated child gets exact clipping through a texture, rather than the bounding box the
/// shape backend has to fall back to.
#[test]
fn snapshot_offscreen_rotated() {
    let render_state = create_render_state(
        default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );
    let installed = render_state.clone();

    let mut harness = Harness::builder()
        .with_size([360.0, 300.0])
        .renderer(WgpuTestRenderer::from_render_state(render_state))
        .build_ui(move |ui| {
            regui::install_wgpu(ui.ctx(), installed.clone());
            let mut value = 0.5;
            Regui::new("child")
                .size(vec2(220.0, 140.0))
                .rotation(0.3)
                .scale(0.9)
                .offscreen(true)
                .show(ui, |ui| child(ui, &mut value));
        });
    harness.run();
    harness.run();
    harness.snapshot("offscreen_rotated");
}

/// The child stays clickable when it is drawn through a texture: input does not go through
/// the renderer at all, but it is easy to break by mixing up the transform.
#[test]
fn an_offscreen_child_is_still_clickable() {
    use std::cell::Cell;
    use std::rc::Rc;

    let clicks = Rc::new(Cell::new(0_u32));
    let spot = Rc::new(Cell::new(Rect::ZERO));

    let render_state = create_render_state(
        default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );
    let installed = render_state.clone();

    let (clicks_in, spot_in) = (Rc::clone(&clicks), Rc::clone(&spot));
    let mut harness = Harness::builder()
        .with_size(SIZE)
        .renderer(WgpuTestRenderer::from_render_state(render_state))
        .build_ui(move |ui| {
            regui::install_wgpu(ui.ctx(), installed.clone());
            let output = Regui::new("child")
                .size(CHILD)
                .offscreen(true)
                .scale(0.75)
                .show(ui, |ui| {
                    let response = ui.button("click me");
                    spot_in.set(response.rect);
                    if response.clicked() {
                        clicks_in.set(clicks_in.get() + 1);
                    }
                });
            spot_in.set(Rect::from_center_size(
                output.transform.mul_pos(spot_in.get().center()),
                vec2(1.0, 1.0),
            ));
        });

    harness.run();
    let target = spot.get().center();
    let button = |pressed| egui::Event::PointerButton {
        pos: target,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    for event in [
        egui::Event::PointerMoved(target),
        button(true),
        button(false),
    ] {
        harness.input_mut().events.push(event);
    }
    harness.run();

    assert_eq!(clicks.get(), 1, "the off-screen child was not clickable");
}
