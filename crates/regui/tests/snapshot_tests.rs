//! Rendering tests for [`regui::Regui`].

use egui::{Slider, Ui, vec2};
use egui_kittest::{Harness, TestRenderer as _, wgpu::WgpuTestRenderer};
use regui::Regui;

/// The ui that gets painted, both directly and through `regui`, so the two can be
/// compared. Deliberately uses text, a rounded frame and a widget with a stroke, since
/// those are the things a transform is most likely to get wrong.
fn demo(ui: &mut Ui, value: &mut f32) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.heading("Heading");
        ui.label("Some text that should stay sharp.");
        ui.add(Slider::new(value, 0.0..=1.0).text("value"));
        let _ = ui.button("A button");
    });
}

fn render(harness: &mut Harness<'_>) -> image::RgbaImage {
    harness.run();
    WgpuTestRenderer::new()
        .render(&harness.ctx, harness.output())
        .expect("failed to render")
}

/// An untransformed child must come out pixel for pixel the same as the ui drawn straight
/// into the parent.
///
/// This is the strongest check there is that the child's shapes survive the trip through a
/// second viewport: same font atlas, same tessellation, same colour space, same position.
#[test]
fn an_untransformed_child_renders_the_same_as_a_plain_ui() {
    let mut direct = Harness::builder().with_size([320.0, 240.0]).build_ui(|ui| {
        let mut value = 0.5;
        ui.allocate_ui(vec2(280.0, 200.0), |ui| demo(ui, &mut value));
    });

    let mut through_regui = Harness::builder().with_size([320.0, 240.0]).build_ui(|ui| {
        let mut value = 0.5;
        Regui::new("child")
            .size(vec2(280.0, 200.0))
            .show(ui, |ui| demo(ui, &mut value));
    });

    let direct = render(&mut direct);
    let through_regui = render(&mut through_regui);

    assert_eq!(
        direct.dimensions(),
        through_regui.dimensions(),
        "the two renders should be the same size"
    );
    assert!(
        direct == through_regui,
        "an untransformed child did not render identically to a plain ui"
    );
}

#[test]
fn snapshot_scaled() {
    let mut harness = Harness::builder().with_size([400.0, 300.0]).build_ui(|ui| {
        let mut value = 0.5;
        Regui::new("child")
            .size(vec2(280.0, 200.0))
            .scale(0.75)
            .show(ui, |ui| demo(ui, &mut value));
    });
    harness.run();
    harness.snapshot("scaled");
}

#[test]
fn snapshot_scaled_crisp() {
    let mut harness = Harness::builder().with_size([400.0, 300.0]).build_ui(|ui| {
        let mut value = 0.5;
        Regui::new("child")
            .size(vec2(180.0, 130.0))
            .scale(1.5)
            .crisp(true)
            .show(ui, |ui| demo(ui, &mut value));
    });
    harness.run();
    harness.snapshot("scaled_crisp");
}

/// Everything the child paints is rotated, including the child's own background.
///
/// The child fills its whole `screen_rect`, the way a window would. Painting that fill in
/// a colour the parent does not use makes it obvious whether the fill is rotated with the
/// content or left behind as an axis-aligned block.
#[test]
fn snapshot_rotated_fill() {
    let mut harness = Harness::builder().with_size([300.0, 300.0]).build_ui(|ui| {
        Regui::new("child")
            .size(vec2(200.0, 120.0))
            .rotation(0.4)
            .show(ui, |ui| {
                let rect = ui.ctx().viewport_rect();
                ui.painter().rect_filled(rect, 0.0, egui::Color32::RED);
                ui.label("rotated");
            });
    });
    harness.run();
    harness.snapshot("rotated_fill");
}

#[test]
fn snapshot_rotated() {
    let mut harness = Harness::builder().with_size([420.0, 360.0]).build_ui(|ui| {
        let mut value = 0.5;
        Regui::new("child")
            .size(vec2(240.0, 160.0))
            .scale(0.8)
            .rotation(0.3)
            .show(ui, |ui| demo(ui, &mut value));
    });
    harness.run();
    harness.snapshot("rotated");
}
