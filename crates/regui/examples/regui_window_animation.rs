//! Show and hide a window by blurring it in and out.
//!
//! This uses both of regui's blurs at once:
//!
//! * [`Regui::blur`] blurs the window's own content, so the panel arrives out of focus and
//!   sharpens as it opens, and smears away as it closes.
//! * [`BackdropBlur`] blurs what is behind the window, and its radius grows with the
//!   animation, so the frosted glass builds up under the panel instead of appearing all at
//!   once. Its edge is feathered while the panel animates, so the glass has no crisp
//!   outline until it has arrived.
//!
//! On top of that the panel grows, turns and slides into place, with [`Regui::scale`],
//! [`Regui::rotation`] and [`Regui::offset`]. Those only change how the child is painted, so
//! the window's layout stays where it is and the panel's text never reflows.
//!
//! The window's frame is drawn inside the child ui rather than by the window, so the frame
//! blurs, fades and scales with everything else on it.

use egui::{
    Color32, Frame, Id, LayerId, Order, Shadow, Slider, Ui, Vec2, Window, emath::easing, vec2,
};
use regui::{BackdropBlur, Regui};

/// How big the panel's content is. Fixed, so the window does not resize while it animates.
const CONTENT_SIZE: Vec2 = vec2(320.0, 190.0);

/// Empty space between the window's edge and its content.
///
/// The content blur spreads outside the child, the slide moves it, and turning it needs
/// more room than it takes up upright. The window clips all of that to its own rect, so this
/// is the room they get to work in: keep it at least as big as the largest blur radius, plus
/// the slide, plus what the rotation costs.
const BLEED: f32 = 56.0;

fn main() {
    let mut open = true;
    let mut animation_time = 0.7_f32;
    let mut content_blur = 24.0_f32;
    let mut backdrop_blur = 16.0_f32;
    let mut slide = 16.0_f32;
    let mut from_scale = 0.9_f32;
    let mut feather = 24.0_f32;
    let mut from_degrees = 6.0_f32;

    hello_egui_utils_dev::run!(move |ui: &mut Ui, frame: &mut eframe::Frame| {
        // Without this, neither blur has a device to work with and both draw nothing.
        if let Some(render_state) = frame.wgpu_render_state() {
            regui::install_wgpu(ui.ctx(), render_state.clone());
        }

        // The backdrop blur can only show what is already drawn, so give it something worth
        // blurring.
        busy_background(ui, ui.max_rect());

        Window::new("animation").show(ui.ctx(), |ui| {
            if ui.button(if open { "hide" } else { "show" }).clicked() {
                open = !open;
            }
            ui.add(Slider::new(&mut animation_time, 0.1..=3.0).text("seconds"));
            ui.add(Slider::new(&mut content_blur, 0.0..=128.0).text("blur the panel"));
            ui.add(Slider::new(&mut backdrop_blur, 0.0..=24.0).text("blur behind it"));
            ui.add(Slider::new(&mut slide, 0.0..=16.0).text("slide"));
            ui.add(Slider::new(&mut from_scale, 0.5..=2.0).text("scale from"));
            ui.add(Slider::new(&mut feather, 0.0..=64.0).text("soften the glass edge"));
            ui.add(Slider::new(&mut from_degrees, -30.0..=30.0).text("rotate from"));
        });

        // One number drives the whole animation: 0 is hidden, 1 is open. `cubic_out` starts
        // fast and settles slowly, and egui flips it when closing, so the panel leaves the
        // way it came.
        let t = ui.ctx().animate_bool_with_time_and_easing(
            Id::new("panel_open"),
            open,
            animation_time,
            easing::cubic_out,
        );

        if t > 0.0 {
            let animation = Animation {
                t,
                content_blur,
                backdrop_blur,
                slide,
                from_scale,
                feather,
                from_rotation: from_degrees.to_radians(),
            };
            panel(ui, animation);
        }
    });
}

/// How the panel is animated, all of it driven by `t`.
#[derive(Clone, Copy)]
struct Animation {
    /// 0 is hidden, 1 is open.
    t: f32,
    content_blur: f32,
    backdrop_blur: f32,
    slide: f32,

    /// How big the panel starts out, as a fraction of its open size.
    from_scale: f32,

    /// How far the glass edge fades out while the panel animates.
    feather: f32,

    /// How far the panel is turned when it is hidden, in radians. It is straight once open.
    from_rotation: f32,
}

/// The panel, somewhere between hidden (`t == 0`) and open (`t == 1`).
fn panel(ui: &mut Ui, animation: Animation) {
    let Animation {
        t,
        content_blur,
        backdrop_blur,
        slide,
        from_scale,
        feather,
        from_rotation,
    } = animation;

    let window_id = Id::new("frosted_panel");
    let corner_radius = ui.visuals().window_corner_radius;

    // The panel grows and straightens up as it opens. The rotation ends at zero, which is
    // what lets the glass under it stay an ordinary upright rect: see below.
    let scale = egui::lerp(from_scale..=1.0, t);
    let rotation = from_rotation * (1.0 - t);

    // Claim the bottom slot of the window's layer before the window runs, so the blur ends
    // up under everything the window draws. The radius and the tint both grow with `t`: at
    // the start there is barely any glass, at the end there is all of it. The corners are
    // scaled along with the panel, so the glass keeps following the frame drawn on it.
    //
    // The edge is feathered while the panel animates and sharp once it is open. Without
    // this the glass keeps a crisp outline around content that has been blurred to nothing,
    // which is the one part of the panel that does not fade.
    let pending = BackdropBlur::new(backdrop_blur * t)
        .corner_radius(corner_radius * scale)
        .feather(feather * (1.0 - t))
        .tint(ui.visuals().window_fill.gamma_multiply(0.6 * t))
        .behind_layer(ui.ctx(), LayerId::new(Order::Middle, window_id));

    // Scaling and rotating change how much room the child needs, and that room is measured
    // from its top left corner, so the panel would drift as it animates. Give back half the
    // difference to hold its centre still. The panel also comes up as it opens.
    // `Regui::offset` moves what is painted without changing what was laid out, so none of
    // this disturbs the window's own layout.
    let centring = (CONTENT_SIZE - painted_size(scale, rotation)) / 2.0;
    let offset = centring + vec2(0.0, (1.0 - t) * slide);

    // No frame and no title bar: the child draws the frame, so it fades with the content.
    // That leaves the `BLEED` margin as the only place to grab the window by, since the
    // child takes the drags over everything else.
    //
    // The size is fixed rather than fitted to the content, so the window keeps its rect
    // while the scaled child inside it grows and shrinks. Otherwise the window would
    // shrink with the child, dragging the panel towards its top left corner.
    let window = Window::new("frosted_panel")
        .id(window_id)
        .title_bar(false)
        .fixed_size(CONTENT_SIZE)
        .frame(Frame::NONE.inner_margin(BLEED));

    let response = window.show(ui.ctx(), |ui| {
        let child = Regui::new("panel_content")
            // The child always lays itself out at this size, whatever it is drawn at, so
            // its text does not reflow while it animates.
            .size(CONTENT_SIZE)
            .scale(scale)
            .rotation(rotation)
            .offset(offset)
            // Fully blurred while hidden, sharp once open. This is the fade: the panel
            // dissolves rather than simply turning see-through.
            .blur(content_blur * (1.0 - t))
            // A half-faded panel should not answer clicks.
            .interactive(t >= 1.0);

        let child = child.show(ui, |ui| {
            // A little opacity on top of the blur, so the panel does not pop in at full
            // strength. `sqrt` gets it most of the way early and leaves the blur to finish
            // the job, which reads as one movement rather than two.
            ui.multiply_opacity(t.sqrt());

            let frame = Frame::window(ui.style())
                // See-through, so the backdrop blur shows through the glass.
                .fill(Color32::TRANSPARENT)
                .corner_radius(corner_radius)
                // A shadow would only be clipped by the child's own edge.
                .shadow(Shadow::NONE);

            frame.show(ui, |ui| {
                // Fill the child, so the frame lines up with the blur behind it.
                ui.set_min_size(ui.available_size());
                content(ui);
            });
        });

        // The rect the child was laid out in grows when it turns, so it is no use as the
        // glass rect. Its centre is what we want: the child is painted centred in it.
        child.response.rect.center()
    });

    match response.and_then(|response| response.inner) {
        // The glass keeps the panel's own scaled size, and follows it as it slides. It
        // cannot follow the rotation, since the blur only masks upright rects, so the
        // corners of a turned panel hang over the edge of their glass. Nothing there is
        // sharp while the panel is turning, so it does not show.
        Some(centre) => pending.set_rect(
            ui.ctx(),
            egui::Rect::from_center_size(centre + offset, CONTENT_SIZE * scale),
        ),
        // Collapsed, so there is nothing to blur behind.
        None => pending.discard(),
    }
}

/// How much room the panel needs once it is scaled and turned.
///
/// This is what [`Regui`] lays out: the bounding box of the transformed child, which is
/// wider and taller than the child itself as soon as it is turned at all.
fn painted_size(scale: f32, rotation: f32) -> Vec2 {
    let (sin, cos) = rotation.sin_cos();
    let (sin, cos) = (sin.abs(), cos.abs());
    scale
        * vec2(
            cos * CONTENT_SIZE.x + sin * CONTENT_SIZE.y,
            sin * CONTENT_SIZE.x + cos * CONTENT_SIZE.y,
        )
}

/// Whatever the panel is for. Ordinary widgets; nothing here knows it is being animated.
fn content(ui: &mut Ui) {
    // A blurred background has less contrast than a flat one, so lean on the theme's
    // strong colour.
    let strong = ui.visuals().strong_text_color();
    ui.visuals_mut().override_text_color = Some(strong);

    ui.heading("Frosted panel");
    ui.label("Hide and show this from the controls behind it.");
    ui.add_space(8.0);
    ui.label("Both blurs run at once: this panel blurs itself away, and the glass under it thins out as it goes.");
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
            painter.circle_filled(
                egui::Pos2::new(x, y),
                spacing / 3.0,
                colors[index % colors.len()],
            );
            index += 1;
            x += spacing;
        }
        y += spacing;
    }
}
