//! A panel sitting on a blurred copy of whatever is behind it.
//!
//! The blur reads what egui has already drawn, so it always shows what is actually behind
//! the panel, with no lag. Drag the window over the pattern to see it keep up.

use egui::{Color32, Frame, Pos2, Slider, Ui, Vec2, Window};
use regui::BackdropBlur;

fn main() {
    let mut radius = 12.0_f32;
    let mut corner_radius = 12_u8;
    // `None` leaves `BackdropBlur` to pick the theme's window fill.
    let mut tint: Option<Color32> = None;

    hello_egui_utils_dev::run!(move |ui: &mut Ui, frame: &mut eframe::Frame| {
        // Without this, `BackdropBlur` has no device to work with and draws nothing.
        if let Some(render_state) = frame.wgpu_render_state() {
            regui::install_wgpu(ui.ctx(), render_state.clone());
        }

        let rect = ui.max_rect();
        busy_background(ui, rect);

        // Drag the window around: the blur always shows what is really behind it, because
        // it reads the frame egui has already drawn rather than guessing.
        Window::new("blurred")
            .title_bar(false)
            .frame(Frame::new().fill(Color32::TRANSPARENT))
            .show(ui, |ui| {
                let mut blur = BackdropBlur::new(radius)
                    .corner_radius(corner_radius)
                    .inner_margin(16);
                if let Some(tint) = tint {
                    blur = blur.tint(tint);
                }

                blur.show(ui, |ui| {
                    // A blurred background has less contrast than a flat one, so lean on
                    // the theme's strong colour for everything on the glass.
                    let strong = ui.visuals().strong_text_color();
                    ui.visuals_mut().override_text_color = Some(strong);
                    ui.visuals_mut().widgets.inactive.fg_stroke.color = strong;
                    ui.visuals_mut().widgets.hovered.fg_stroke.color = strong;
                    ui.visuals_mut().widgets.active.fg_stroke.color = strong;

                    ui.heading("Frosted glass");
                    ui.label("The pattern behind this panel is blurred.");
                    ui.add_space(8.0);
                    ui.add(Slider::new(&mut radius, 0.0..=40.0).text("blur radius"));
                    ui.add(Slider::new(&mut corner_radius, 0..=60).text("corner radius"));

                    ui.add_space(8.0);
                    let mut custom = tint.is_some();
                    if ui.checkbox(&mut custom, "override the tint").changed() {
                        tint = custom.then(|| ui.visuals().window_fill);
                    }
                    if let Some(tint) = &mut tint {
                        ui.horizontal(|ui| {
                            ui.color_edit_button_srgba(tint);
                            ui.label("its alpha fades the blur towards it");
                        });
                    }
                });
            });
    });
}

/// Something with enough detail that a blur is obvious.
fn busy_background(ui: &Ui, rect: egui::Rect) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 24, 40));

    let colors = [
        Color32::from_rgb(255, 92, 92),
        Color32::from_rgb(92, 255, 140),
        Color32::from_rgb(92, 160, 255),
        Color32::from_rgb(255, 216, 92),
    ];

    let spacing = 44.0;
    let mut index = 0;
    let mut y = rect.top() + spacing / 2.0;
    while y < rect.bottom() {
        let mut x = rect.left() + spacing / 2.0;
        while x < rect.right() {
            painter.circle_filled(Pos2::new(x, y), spacing / 3.0, colors[index % colors.len()]);
            index += 1;
            x += spacing;
        }
        y += spacing;
    }

    for i in 0..12 {
        let t = i as f32 / 12.0;
        painter.line_segment(
            [
                rect.lerp_inside(Vec2::new(t, 0.0)),
                rect.lerp_inside(Vec2::new(1.0 - t, 1.0)),
            ],
            egui::Stroke::new(1.5, Color32::from_white_alpha(40)),
        );
    }
}
