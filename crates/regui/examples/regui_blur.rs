//! A panel sitting on a blurred copy of whatever is behind it.
//!
//! The blur reads what egui has already drawn, so it always shows what is actually behind
//! the panel, with no lag. Drag the window over the pattern to see it keep up.

use egui::{Color32, Frame, Id, Pos2, Shadow, Slider, Ui, Vec2, Window};
use regui::BackdropBlur;

fn main() {
    let mut radius = 12.0_f32;
    let mut corner_radius = 12_u8;
    // `None` leaves `BackdropBlur` to pick the theme's window fill.
    let mut tint: Option<Color32> = None;
    let mut show_demo = false;
    let mut demo = egui_demo_lib::DemoWindows::default();

    hello_egui_utils_dev::run!(move |ui: &mut Ui, frame: &mut eframe::Frame| {
        // Without this, `BackdropBlur` has no device to work with and draws nothing.
        if let Some(render_state) = frame.wgpu_render_state() {
            regui::install_wgpu(ui.ctx(), render_state.clone());
        }

        // Whatever is drawn here is what the blur will pick up. The demo is the more
        // interesting test: it has panels, windows, text and images, so you can see how the
        // blur behaves over each. Its own windows are ordinary areas, so whether one ends up
        // above or below a blurred window depends on which you clicked last.
        if show_demo {
            demo.ui(ui);
        } else {
            busy_background(ui, ui.max_rect());
        }

        ui.ctx().all_styles_mut(|style| {
            style.visuals.window_shadow = Shadow::NONE;
        });

        for i in 0..2 {
            // A see-through fill, so the blur shows through. The frame's stroke, shadow and
            // rounded corners are still drawn, on top of the blur.
            let frame = Frame::window(ui.style()).fill(Color32::TRANSPARENT);

            let mut blur = BackdropBlur::new(radius).corner_radius(frame.corner_radius);
            if let Some(tint) = tint {
                blur = blur.tint(tint);
            }

            // `show_window` claims the first slot in the window's layer, so the blur sits under
            // everything the window draws, its own frame included. Drag the window around: the
            // blur always shows what is really behind it, because it reads the frame egui has
            // already drawn rather than guessing.
            blur.show_window(
                ui,
                Id::new(("blurred", i)),
                Window::new("blurred").frame(frame),
                |ui| {
                    // A blurred background has less contrast than a flat one, so lean on the
                    // theme's strong colour for everything on the glass.
                    let strong = ui.visuals().strong_text_color();
                    ui.visuals_mut().override_text_color = Some(strong);
                    ui.visuals_mut().widgets.inactive.fg_stroke.color = strong;
                    ui.visuals_mut().widgets.hovered.fg_stroke.color = strong;
                    ui.visuals_mut().widgets.active.fg_stroke.color = strong;

                    ui.heading("Frosted glass");
                    ui.label("Whatever is behind this window is blurred.");
                    ui.add_space(8.0);
                    ui.checkbox(&mut show_demo, "show the egui demo behind");
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
                },
            );
        }
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
