//! Tests for [`regui::effect::MotionBlur`], the directional blur.

#![cfg(feature = "wgpu")]

use egui::{Color32, Rect, Slider, Ui, Vec2, vec2};
use egui_kittest::{
    Harness,
    wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup},
};
use regui::{Regui, effect::MotionBlur};

const SIZE: [f32; 2] = [320.0, 220.0];
const BLOCK: egui::Vec2 = vec2(120.0, 80.0);

/// How far the block sits from the panel's top left, so a smear has room on every side.
const MARGIN: f32 = 60.0;

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

/// Where two images differ, as a pixel rect, and `None` if they are the same.
///
/// A channel has to move by `threshold` to count, so the faintest tail of a smear does not
/// decide the answer.
fn changed_bounds(
    a: &image::RgbaImage,
    b: &image::RgbaImage,
    threshold: u8,
) -> Option<(u32, u32, u32, u32)> {
    let mut min = (u32::MAX, u32::MAX);
    let mut max = (0_u32, 0_u32);
    for (x, y, pixel) in a.enumerate_pixels() {
        let other = b.get_pixel(x, y);
        let moved = pixel
            .0
            .iter()
            .zip(other.0.iter())
            .any(|(l, r)| l.abs_diff(*r) >= threshold);
        if moved {
            min.0 = min.0.min(x);
            min.1 = min.1.min(y);
            max.0 = max.0.max(x);
            max.1 = max.1.max(y);
        }
    }
    (min.0 != u32::MAX).then_some((min.0, min.1, max.0, max.1))
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

/// Render a solid white block with an effect built by `effect`, and hand back the image and
/// the rect the block landed in.
///
/// The block fills the child completely, so its content touches every edge and the smear
/// has to reach past them. Its hard edges also make the extent of the smear easy to
/// measure. `effect` is called again every frame, since an [`regui::Effect`] is not
/// `Clone`.
fn scene(effect: impl Fn() -> Option<MotionBlur> + 'static) -> (image::RgbaImage, Rect) {
    let rect = std::rc::Rc::new(std::cell::Cell::new(Rect::ZERO));
    let out = rect.clone();
    let image = render(move |ui| {
        let rect = out.clone();
        ui.add_space(MARGIN);
        ui.horizontal(|ui| {
            ui.add_space(MARGIN);
            let mut child = Regui::new("child")
                .size(BLOCK)
                .offscreen(true)
                .interactive(false);
            if let Some(effect) = effect() {
                child = child.effect(effect);
            }
            let output = child.show(ui, |ui| {
                let viewport = ui.ctx().viewport_rect();
                ui.painter().rect_filled(viewport, 0.0, Color32::WHITE);
            });
            rect.set(output.response.rect);
        });
    });
    (image, rect.get())
}

/// The point of the effect: it smears along the velocity and nowhere else.
///
/// A symmetric smear along x has to reach past the block's left and right edges, and leave
/// its top and bottom exactly where they were.
#[test]
fn a_horizontal_velocity_spreads_sideways_only() {
    const REACH: f32 = 24.0;

    let (sharp, rect) = scene(|| None);
    let (smeared, _) = scene(|| Some(MotionBlur::new(vec2(REACH, 0.0)).symmetric()));

    let (left, top, right, bottom) =
        changed_bounds(&sharp, &smeared, 4).expect("the smear changed nothing at all");

    let slack = 3.0;
    assert!(
        (left as f32) < rect.left() - slack && (right as f32) > rect.right() + slack,
        "the smear did not reach past the block sideways: it spans x {left}..{right}, but the \
         block spans {}..{}",
        rect.left(),
        rect.right()
    );
    assert!(
        (top as f32) > rect.top() - slack && (bottom as f32) < rect.bottom() + slack,
        "the smear blurred vertically as well: it spans y {top}..{bottom}, but the block spans \
         {}..{}",
        rect.top(),
        rect.bottom()
    );
}

/// A zero velocity has to be a clean no-op, and must not divide by zero.
#[test]
fn zero_velocity_changes_nothing() {
    let (plain, _) = scene(|| None);
    let (still, _) = scene(|| Some(MotionBlur::new(Vec2::ZERO)));

    let delta = difference(&plain, &still);
    assert!(
        delta < 0.001,
        "a motion blur of zero velocity changed the child by {delta:.5} per channel"
    );
}

/// A longer velocity reaches further.
#[test]
fn a_longer_velocity_reaches_further() {
    let (sharp, rect) = scene(|| None);
    let (short, _) = scene(|| Some(MotionBlur::new(vec2(12.0, 0.0)).symmetric()));
    let (long, _) = scene(|| Some(MotionBlur::new(vec2(36.0, 0.0)).symmetric()));

    let reach = |image: &image::RgbaImage| {
        let (left, _, right, _) =
            changed_bounds(&sharp, image, 4).expect("the smear changed nothing at all");
        (rect.left() - left as f32, right as f32 - rect.right())
    };
    let (short_left, short_right) = reach(&short);
    let (long_left, long_right) = reach(&long);

    assert!(
        long_left > short_left + 8.0 && long_right > short_right + 8.0,
        "the longer velocity did not reach further: 12 points reached {short_left}/{short_right} \
         and 36 points reached {long_left}/{long_right}"
    );
}

/// A trailing smear lands behind the block, not in front of it.
///
/// The block moves right, so the streak has to be on its left. Nothing may appear past its
/// right edge: that is where the block is now, not where it has been.
#[test]
fn a_trailing_smear_lands_behind() {
    const REACH: f32 = 30.0;

    let (sharp, rect) = scene(|| None);
    let (smeared, _) = scene(|| Some(MotionBlur::new(vec2(REACH, 0.0)).trailing()));

    let (left, _, right, _) =
        changed_bounds(&sharp, &smeared, 4).expect("the smear changed nothing at all");

    assert!(
        (left as f32) < rect.left() - 8.0,
        "the trail did not land behind the block: it starts at x {left}, but the block starts \
         at {}",
        rect.left()
    );
    assert!(
        (right as f32) <= rect.right() + 3.0,
        "the trail ran ahead of the block: it ends at x {right}, but the block ends at {}",
        rect.right()
    );
}

/// The child ui the snapshots use. Text and a frame, since a smear over flat colour tells
/// you nothing about banding.
fn panel(ui: &mut Ui, value: &mut f32) {
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

/// Snapshot the panel with `effect` on it.
fn snapshot(name: &str, effect: impl Fn() -> MotionBlur + 'static) {
    let render_state = create_render_state(
        default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );
    let installed = render_state.clone();

    let mut harness = Harness::builder()
        .with_size([360.0, 260.0])
        .renderer(WgpuTestRenderer::from_render_state(render_state))
        .build_ui(move |ui| {
            regui::install_wgpu(ui.ctx(), installed.clone());
            ui.add_space(40.0);
            ui.horizontal(|ui| {
                ui.add_space(50.0);
                let mut value = 0.5;
                Regui::new("child")
                    .size(vec2(220.0, 140.0))
                    .effect(effect())
                    .interactive(false)
                    .show(ui, |ui| panel(ui, &mut value));
            });
        });
    harness.run();
    harness.run();
    harness.snapshot(name);
}

#[test]
fn snapshot_motion_blur_horizontal() {
    snapshot("motion_blur_horizontal", || {
        MotionBlur::new(vec2(24.0, 0.0))
    });
}

#[test]
fn snapshot_motion_blur_diagonal() {
    snapshot("motion_blur_diagonal", || MotionBlur::new(vec2(18.0, 12.0)));
}

#[test]
fn snapshot_motion_blur_symmetric() {
    snapshot("motion_blur_symmetric", || {
        MotionBlur::new(vec2(0.0, 20.0)).symmetric()
    });
}
