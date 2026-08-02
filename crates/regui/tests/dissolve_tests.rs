//! Tests for the `Dissolve` effect, which breaks a child ui up instead of fading it.

#![cfg(feature = "wgpu")]

use egui::{Color32, Rect, Ui, Vec2, vec2};
use egui_kittest::{
    Harness,
    wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup},
};
use regui::{Regui, effect::Dissolve};

const SIZE: [f32; 2] = [320.0, 220.0];
const CHILD: Vec2 = vec2(240.0, 150.0);

/// The child ui under test.
///
/// It fills itself completely, so every pixel of the child either survives the dissolve or
/// does not, and there is no transparent background to confuse the counting.
fn child(ui: &mut Ui) {
    let rect = ui.ctx().viewport_rect();
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_rgb(230, 235, 245));
    ui.heading("Heading");
    ui.label("Some text that has to survive the trip.");
    let _ = ui.button("A button");
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

/// Render the child with an effect built by `effect`, and report where the child landed.
fn dissolved(effect: impl Fn() -> Option<Dissolve> + Send + Sync + 'static) -> Scene {
    use std::cell::Cell;
    use std::rc::Rc;

    let rect = Rc::new(Cell::new(Rect::ZERO));
    let out = Rc::clone(&rect);

    let image = render(move |ui| {
        let mut regui = Regui::new("child").size(CHILD).interactive(false);
        if let Some(effect) = effect() {
            regui = regui.effect(effect);
        } else {
            regui = regui.offscreen(true);
        }
        out.set(regui.show(ui, child).response.rect);
    });

    Scene {
        image,
        rect: rect.get(),
    }
}

/// One rendered frame, and where the child sat in it.
struct Scene {
    image: image::RgbaImage,
    rect: Rect,
}

impl Scene {
    /// Walk every pixel of the child's rect.
    fn child_pixels(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        let (width, height) = self.image.dimensions();
        let left = self.rect.left().ceil().max(0.0) as u32;
        let top = self.rect.top().ceil().max(0.0) as u32;
        let right = (self.rect.right().floor() as u32).min(width);
        let bottom = (self.rect.bottom().floor() as u32).min(height);
        (top..bottom).flat_map(move |y| (left..right).map(move |x| (x, y)))
    }
}

/// How many channel values two images differ by, as a fraction.
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

/// Full progress has to leave the child exactly as it was.
///
/// The soft band makes this easy to get wrong by one band's worth, and a panel that is a
/// little bit eaten before the animation has started is very easy to see.
#[test]
fn full_progress_changes_nothing() {
    let plain = dissolved(|| None);
    for softness in [0.0, 0.15, 1.0] {
        let whole = dissolved(move || Some(Dissolve::new(1.0).softness(softness)));
        let delta = difference(&plain.image, &whole.image);
        assert!(
            delta < 1e-9,
            "a dissolve at full progress with softness {softness} changed the child by \
             {delta:.6} per channel; it must be untouched"
        );
    }
}

/// No progress has to leave nothing at all, however wide the band is.
#[test]
fn no_progress_leaves_nothing() {
    let empty = render(|ui| {
        let _ = ui.allocate_space(CHILD);
    });

    for softness in [0.0, 0.15, 1.0] {
        let gone = dissolved(move || Some(Dissolve::new(0.0).softness(softness)));
        let delta = difference(&empty, &gone.image);
        assert!(
            delta < 1e-9,
            "a dissolve at no progress with softness {softness} left {delta:.6} per channel \
             behind; the child must be gone"
        );
    }
}

/// Half way through, some of the child is gone and some of it is not.
///
/// This is what tells a dissolve from a fade: whole pixels and missing pixels side by side,
/// rather than every pixel at half strength.
#[test]
fn a_middle_progress_removes_some_pixels() {
    let whole = dissolved(|| None);
    let empty = dissolved(|| Some(Dissolve::new(0.0)));
    let middle = dissolved(|| Some(Dissolve::new(0.5)));

    let mut kept = 0_u32;
    let mut removed = 0_u32;
    let mut total = 0_u32;
    for (x, y) in middle.child_pixels() {
        let pixel = middle.image.get_pixel(x, y);
        total += 1;
        if pixel == whole.image.get_pixel(x, y) {
            kept += 1;
        } else if pixel == empty.image.get_pixel(x, y) {
            removed += 1;
        }
    }

    assert!(total > 1000, "the child covered almost no pixels");
    let tenth = total / 10;
    assert!(
        kept > tenth,
        "only {kept} of {total} pixels came through untouched; the dissolve took too much"
    );
    assert!(
        removed > tenth,
        "only {removed} of {total} pixels were removed; this reads as a fade, not a dissolve"
    );
}

/// A wipe eats the child from the side it is pointed at, and leaves the other side alone.
#[test]
fn a_wipe_eats_one_side() {
    let whole = dissolved(|| None);
    let empty = dissolved(|| Some(Dissolve::new(0.0)));
    let wiped = dissolved(|| Some(Dissolve::new(0.5).wipe(Vec2::RIGHT).softness(0.0)));

    let middle = wiped.rect.center().x;
    let mut left_kept = 0_u32;
    let mut left_removed = 0_u32;
    let mut right_kept = 0_u32;
    let mut right_removed = 0_u32;

    for (x, y) in wiped.child_pixels() {
        let pixel = wiped.image.get_pixel(x, y);
        // Skip the couple of pixels either side of the front, where the two are equal.
        let (kept, removed) = if (x as f32) < middle - 3.0 {
            (&mut left_kept, &mut left_removed)
        } else if (x as f32) > middle + 3.0 {
            (&mut right_kept, &mut right_removed)
        } else {
            continue;
        };
        if pixel == whole.image.get_pixel(x, y) {
            *kept += 1;
        } else if pixel == empty.image.get_pixel(x, y) {
            *removed += 1;
        }
    }

    assert_eq!(
        left_removed, 0,
        "a wipe to the right removed {left_removed} pixels from the left half"
    );
    assert_eq!(
        right_kept, 0,
        "a wipe to the right left {right_kept} pixels of the right half standing"
    );
    assert!(left_kept > 1000, "the left half was not drawn at all");
    assert!(right_removed > 1000, "the right half was not removed");
}

/// Build a harness for a snapshot, with one dissolving child.
fn snapshot(name: &str, effect: impl Fn() -> Dissolve + Send + Sync + 'static) {
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
            Regui::new("child")
                .size(CHILD)
                .interactive(false)
                .effect(effect())
                .show(ui, |ui| {
                    let rect = ui.ctx().viewport_rect();
                    ui.painter()
                        .rect_filled(rect, 8.0, Color32::from_rgb(40, 90, 170));
                    ui.heading("Dissolving");
                    ui.label("Some text that has to break up with the rest.");
                    let _ = ui.button("A button");
                });
        });
    harness.run();
    harness.run();
    harness.snapshot(name);
}

#[test]
fn snapshot_dissolve_noise() {
    snapshot("dissolve_noise", || Dissolve::new(0.55));
}

#[test]
fn snapshot_dissolve_burn() {
    snapshot("dissolve_burn", || {
        Dissolve::new(0.55)
            .noise(14.0)
            .softness(0.25)
            .burn(Color32::from_rgb(255, 140, 40))
    });
}

#[test]
fn snapshot_dissolve_wipe() {
    snapshot("dissolve_wipe", || {
        Dissolve::new(0.5)
            .wipe(Vec2::RIGHT)
            .softness(0.12)
            .burn(Color32::from_rgb(255, 200, 90))
    });
}
