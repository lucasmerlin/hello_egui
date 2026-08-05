//! Every shader regui has, side by side.
//!
//! One button hides and shows all of them at once, so you can watch what each one does with
//! the same animation. The tiles hold ordinary widgets, and nothing inside a tile knows it
//! is being blurred, dissolved or smeared.
//!
//! Two different things are on show here:
//!
//! * An [`regui::Effect`] runs over the child ui's own image, after it has been rendered.
//!   The blur, the dissolve, the motion blur and the shadow are all effects, and they
//!   chain: the last tile runs three of them one after the other.
//! * [`BackdropBlur`] works the other way round. It blurs what is already _behind_ a rect,
//!   so a panel can sit on frosted glass.

use egui::{Color32, Frame, Id, Rect, RichText, Slider, Ui, UiBuilder, Vec2, emath::easing, vec2};
use regui::{
    BackdropBlur, Regui,
    effect::{Dissolve, MotionBlur, Shadow},
};

/// How big every tile's content is.
const CARD: Vec2 = vec2(215.0, 120.0);

/// Space between tiles.
///
/// Effects paint outside the child they run on, and the room they take is not part of the
/// layout, so a shadow or a smear will happily reach over its neighbour. This is how much
/// rope they get.
const GAP: f32 = 34.0;

/// The colour the dissolve front glows.
const BURN: Color32 = Color32::from_rgb(255, 140, 40);

fn main() {
    let mut open = true;
    let mut settings = Settings::default();

    // Where the sliding tile was last frame. The difference is its velocity, which is what
    // a motion blur wants; guessing at it from the animation would be a worse example.
    let mut previous_travel = Vec2::ZERO;

    let mut sized = false;

    hello_egui_utils_dev::run!(move |ui: &mut Ui, frame: &mut eframe::Frame| {
        // Without this, none of the shaders have a device to work with and nothing happens.
        if let Some(render_state) = frame.wgpu_render_state() {
            regui::install_wgpu(ui.ctx(), render_state.clone());
        }

        // Eight tiles need the room. Only once, so the window can still be resized.
        if !sized {
            sized = true;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(vec2(1320.0, 700.0)));
        }

        // The backdrop blur can only show what is already drawn, so give it something worth
        // blurring. It is behind the tiles as well, which is what makes the dissolve and the
        // motion blur easy to read.
        busy_background(ui, ui.max_rect());

        // A panel rather than a window, so it never covers a tile. It draws its own
        // background over the pattern, since the pattern is only there for the glass.
        egui::Panel::right("controls").show(ui, |ui| controls(ui, &mut open, &mut settings));

        // One number drives every tile: 0 is hidden, 1 is shown. `cubic_out` starts fast and
        // settles slowly, and egui runs it backwards when closing.
        let t = ui.ctx().animate_bool_with_time_and_easing(
            Id::new("effects_open"),
            open,
            settings.seconds,
            easing::cubic_out,
        );

        let travel = vec2(-settings.slide * (1.0 - t), 0.0);
        let velocity = (travel - previous_travel) * settings.shutter;
        previous_travel = travel;

        tiles(ui, t, travel, velocity, &settings);
    });
}

/// Every tile, laid out across the window.
fn tiles(ui: &mut Ui, t: f32, travel: Vec2, velocity: Vec2, settings: &Settings) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(GAP);
        ui.horizontal_wrapped(|ui| {
            tile(ui, "Blur", "The child's own image, out of focus.", |ui| {
                show(ui, "blur", t, Vec2::ZERO, |child| {
                    // A radius of zero adds no effect at all, so the finished tile costs
                    // nothing.
                    child.blur(settings.blur * (1.0 - t))
                });
            });

            tile(ui, "Dissolve", "Breaks up, with a burning front.", |ui| {
                show(ui, "dissolve", t, Vec2::ZERO, |child| {
                    child.effect(Dissolve::new(t).noise(settings.speckle).burn(BURN))
                });
            });

            tile(ui, "Wipe", "The same dissolve, along a direction.", |ui| {
                show(ui, "wipe", t, Vec2::ZERO, |child| {
                    child.effect(
                        Dissolve::new(t)
                            .wipe(vec2(1.0, 0.25))
                            .softness(0.25)
                            .burn(BURN),
                    )
                });
            });

            tile(ui, "Motion blur", "Smeared along the way it moves.", |ui| {
                // The offset moves what is painted without touching the layout, so the
                // tile slides without pushing its neighbours around.
                show(ui, "motion", t, travel, |child| {
                    child.effect(MotionBlur::new(velocity))
                });
            });

            tile(
                ui,
                "Drop shadow",
                "Thrown by the content, not a rect.",
                |ui| {
                    show(ui, "shadow", t, Vec2::ZERO, |child| {
                        // The card lifts as it arrives: the shadow starts tight underneath
                        // and grows as it rises.
                        child.effect(
                            Shadow::new()
                                .radius(settings.shadow * t)
                                .offset(vec2(0.0, 10.0 * t)),
                        )
                    });
                },
            );

            tile(
                ui,
                "Frosted glass",
                "Blurs what is behind it, and bends it.",
                |ui| {
                    glass(ui, "frosted", t, settings, false);
                },
            );

            tile(
                ui,
                "Liquid glass",
                "A squircle, and a lens at the rim.",
                |ui| {
                    glass(ui, "liquid", t, settings, true);
                },
            );

            tile(
                ui,
                "All three",
                "Effects chain, in the order given.",
                |ui| {
                    show(ui, "chained", t, travel, |child| {
                        child
                            .effect(Shadow::new().radius(settings.shadow * t))
                            .effect(MotionBlur::new(velocity))
                            .effect(Dissolve::new(t).noise(settings.speckle))
                    });
                },
            );
        });
    });
}

/// What the sliders in the controls window set.
struct Settings {
    seconds: f32,
    blur: f32,
    speckle: f32,
    slide: f32,

    /// How long the motion blur's shutter is open, as a multiple of one frame's movement.
    shutter: f32,

    shadow: f32,
    glass_blur: f32,
    refraction: f32,
    specular: f32,

    /// How hard the liquid glass squeezes what is behind its rim.
    lens: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            seconds: 0.9,
            blur: 20.0,
            speckle: 10.0,
            slide: 220.0,
            shutter: 3.0,
            shadow: 18.0,
            glass_blur: 12.0,
            refraction: 8.0,
            specular: 0.7,
            lens: 1.0,
        }
    }
}

fn controls(ui: &mut Ui, open: &mut bool, settings: &mut Settings) {
    if ui.button(if *open { "hide" } else { "show" }).clicked() {
        *open = !*open;
    }
    ui.add(Slider::new(&mut settings.seconds, 0.1..=4.0).text("seconds"));

    ui.separator();
    ui.add(Slider::new(&mut settings.blur, 0.0..=64.0).text("blur"));
    ui.add(Slider::new(&mut settings.speckle, 2.0..=40.0).text("speckle size"));
    ui.add(Slider::new(&mut settings.slide, 0.0..=600.0).text("slide"));
    ui.add(Slider::new(&mut settings.shutter, 0.0..=10.0).text("shutter"));
    ui.add(Slider::new(&mut settings.shadow, 0.0..=40.0).text("shadow"));

    ui.separator();
    ui.add(Slider::new(&mut settings.glass_blur, 0.0..=24.0).text("glass blur"));
    ui.add(Slider::new(&mut settings.refraction, 0.0..=24.0).text("refraction"));
    ui.add(Slider::new(&mut settings.specular, 0.0..=1.0).text("specular"));
    ui.add(Slider::new(&mut settings.lens, 0.0..=1.0).text("lens"));
}

/// One labelled tile in the grid.
fn tile(ui: &mut Ui, title: &str, note: &str, content: impl FnOnce(&mut Ui)) {
    // The header is a fixed height, so every card in a row starts at the same line however
    // long its note wraps.
    const HEADER: f32 = 40.0;

    ui.allocate_ui(vec2(CARD.x, CARD.y + HEADER), |ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
        ui.vertical(|ui| {
            ui.allocate_ui(vec2(CARD.x, HEADER), |ui| {
                // The pattern behind is bright and busy, so lay the label on something
                // dark. Nothing else here would be readable over it.
                let rect = ui.max_rect().expand2(vec2(6.0, 2.0));
                ui.painter()
                    .rect_filled(rect, 6.0, Color32::from_black_alpha(150));
                // Set the colour on the text itself: `strong` would otherwise pick the
                // theme's dark strong colour and vanish into the chip.
                ui.label(RichText::new(title).strong().color(Color32::from_gray(245)));
                ui.label(RichText::new(note).small().color(Color32::from_gray(190)));
            });
            content(ui);
        });
    });
}

/// A card, run as a child ui with whatever effects are put on it.
fn show(ui: &mut Ui, id: &str, t: f32, offset: Vec2, effects: impl FnOnce(Regui) -> Regui) {
    let child = Regui::new(id)
        // The child always lays itself out at this size, whatever it is drawn at, so its
        // text does not reflow while it animates.
        .size(CARD)
        .offset(offset)
        // A card that is half gone should not answer clicks.
        .interactive(t >= 1.0);

    effects(child).show(ui, card);
}

/// Whatever the panel is for. Ordinary widgets; nothing here knows about any of this.
fn card(ui: &mut Ui) {
    Frame::window(ui.style()).show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        ui.label("A panel with things on it.");
        let _ = ui.button("A button");
        ui.small("Ordinary widgets, laid out as usual.");
    });
}

/// The two glass tiles.
///
/// Neither is an effect: nothing runs over the card's image. [`BackdropBlur`] blurs what
/// egui has already drawn under the rect, and the card is then drawn on top of it.
///
/// `liquid` picks between the two looks. Frosted glass is a blurred rectangle with a ground
/// edge: the rim bends what lies beside it and catches the light. Liquid glass is a
/// squircle, and instead of bending its edge it gathers the whole background into the rim,
/// so the ring shows a squeezed copy of everything around the panel.
fn glass(ui: &mut Ui, id: &str, t: f32, settings: &Settings, liquid: bool) {
    // A blur takes its id from the `Ui` it is painted into, so two of them in one `Ui`
    // would share it and only the second would be drawn. Give each its own.
    ui.push_id(id, |ui| glass_inner(ui, t, settings, liquid));
}

fn glass_inner(ui: &mut Ui, t: f32, settings: &Settings, liquid: bool) {
    let (rect, _) = ui.allocate_exact_size(CARD, egui::Sense::hover());

    let glass = BackdropBlur::new(settings.glass_blur * t)
        .tint(ui.visuals().window_fill.gamma_multiply(0.5 * t))
        // While the tile animates its edge is soft, so the glass has no crisp outline
        // around content that has not arrived yet.
        .feather(20.0 * (1.0 - t));

    let glass = if liquid {
        // One call sets the squircle, the lens, the sheen, the bevel, the grain and the
        // colour fringe. The lens is what does most of the work.
        glass.liquid_glass().lens(settings.lens * t)
    } else {
        glass
            .corner_radius(12.0 * t)
            .refraction(settings.refraction * t)
            .specular(settings.specular * t)
    };

    glass.paint_at(ui, rect);

    ui.scope_builder(UiBuilder::new().max_rect(rect.shrink(14.0)), |ui| {
        ui.multiply_opacity(t);
        ui.label("A panel with things on it.");
        let _ = ui.button("A button");
    });
}

/// Something with enough detail that a blur is obvious.
fn busy_background(ui: &Ui, rect: Rect) {
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
