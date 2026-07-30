//! Behaviour tests for [`regui::Regui`].
//!
//! These drive raw pointer and key events rather than using kittest's accessibility
//! queries, because the child viewport's widgets are not in the parent's AccessKit tree.
//!
//! Each test asks the child where its widget ended up, in the child's own coordinates,
//! and then uses the transform that `show` hands back to work out where to click. That
//! way the tests check the same mapping the pointer goes through, from the other side.

use egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, Sense, TextEdit, Vec2, vec2};
use egui_kittest::Harness;
use regui::{Regui, Transform};
use std::cell::{Cell, RefCell};

const CHILD_SIZE: Vec2 = vec2(200.0, 100.0);

/// What a test's ui function reports back about the child.
#[derive(Default)]
struct Probe {
    /// Maps the child's coordinates to the parent's.
    transform: Cell<Option<Transform>>,

    /// The rect of the widget under test, in the child's coordinates.
    rect: Cell<Option<Rect>>,

    clicks: Cell<u32>,
    hovered: Cell<bool>,
}

impl Probe {
    /// Where the probed widget's centre is on screen.
    fn widget_center(&self) -> Pos2 {
        let transform = self.transform.get().expect("the child ui should have run");
        let rect = self.rect.get().expect("the child ui should have run");
        transform.mul_pos(rect.center())
    }
}

fn click_at(harness: &mut Harness<'_>, pos: Pos2) {
    let button = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    for event in [Event::PointerMoved(pos), button(true), button(false)] {
        harness.input_mut().events.push(event);
    }
    harness.run();
}

fn move_pointer_to(harness: &mut Harness<'_>, pos: Pos2) {
    harness.input_mut().events.push(Event::PointerMoved(pos));
    // Twice: hovering needs the widget rects from the previous pass.
    harness.run();
    harness.run();
}

/// A button inside the child must be clickable, at the position it appears to be at.
///
/// This exercises the whole input path: the parent's response gates the events, they are
/// mapped into the child's coordinate space, and the child hit-tests them itself.
#[test]
fn a_button_in_the_child_can_be_clicked() {
    let probe = Probe::default();

    let mut harness = Harness::new_ui(|ui| {
        let output = Regui::new("child").size(CHILD_SIZE).show(ui, |ui| {
            let response = ui.button("click me");
            probe.rect.set(Some(response.rect));
            if response.clicked() {
                probe.clicks.set(probe.clicks.get() + 1);
            }
        });
        probe.transform.set(Some(output.transform));
    });

    harness.run();

    click_at(&mut harness, probe.widget_center());
    assert_eq!(
        probe.clicks.get(),
        1,
        "clicking the child's button did not register"
    );

    // Clicking well outside the child must not reach it.
    click_at(&mut harness, probe.widget_center() + vec2(500.0, 400.0));
    assert_eq!(
        probe.clicks.get(),
        1,
        "a click outside the child leaked into it"
    );
}

/// The same, but scaled and rotated, so the pointer has to go through a transform that is
/// not the identity to land on the button.
#[test]
fn a_transformed_child_maps_the_pointer_correctly() {
    const SCALE: f32 = 0.6;
    const ROTATION: f32 = 0.4;

    let probe = Probe::default();

    let mut harness = Harness::new_ui(|ui| {
        let output = Regui::new("child")
            .size(CHILD_SIZE)
            .scale(SCALE)
            .rotation(ROTATION)
            .show(ui, |ui| {
                let response = ui.button("click me");
                probe.rect.set(Some(response.rect));
                if response.clicked() {
                    probe.clicks.set(probe.clicks.get() + 1);
                }
            });
        probe.transform.set(Some(output.transform));
    });

    harness.run();

    click_at(&mut harness, probe.widget_center());
    assert_eq!(
        probe.clicks.get(),
        1,
        "the pointer was not mapped into the transformed child's space"
    );

    // The same point, but ignoring the transform: that is somewhere else entirely, so it
    // must not hit the button.
    let untransformed = probe.rect.get().unwrap().center();
    click_at(&mut harness, untransformed);
    assert_eq!(
        probe.clicks.get(),
        1,
        "the transform was not applied to the pointer at all"
    );
}

/// Text typed into the parent must not also go into the child, and vice versa.
///
/// Keyboard events only reach the child once something inside it has focus, which is what
/// keeps two text fields from both receiving the same keystrokes.
#[test]
fn keyboard_input_goes_to_whichever_text_field_has_focus() {
    let probe = Probe::default();
    let parent_text = RefCell::new(String::new());
    let child_text = RefCell::new(String::new());

    let mut harness = Harness::new_ui(|ui| {
        ui.add(TextEdit::singleline(&mut *parent_text.borrow_mut()).id_salt("parent_edit"));
        let output = Regui::new("child").size(CHILD_SIZE).show(ui, |ui| {
            let response =
                ui.add(TextEdit::singleline(&mut *child_text.borrow_mut()).id_salt("child_edit"));
            probe.rect.set(Some(response.rect));
        });
        probe.transform.set(Some(output.transform));
    });

    harness.run();

    // Nothing is focused yet, so nothing should receive the text.
    harness.input_mut().events.push(Event::Text("a".to_owned()));
    harness.run();
    assert_eq!(parent_text.borrow().as_str(), "");
    assert_eq!(child_text.borrow().as_str(), "");

    // Click the child's text field, then type.
    click_at(&mut harness, probe.widget_center());
    harness.input_mut().events.push(Event::Text("b".to_owned()));
    harness.run();
    assert_eq!(
        child_text.borrow().as_str(),
        "b",
        "the child's text field did not get the text"
    );
    assert_eq!(
        parent_text.borrow().as_str(),
        "",
        "the text leaked into the parent"
    );
}

/// The child's own repaint requests have to wake the parent up, or a child animation would
/// run for a single pass and then freeze.
#[test]
fn a_repaint_request_from_the_child_reaches_the_parent() {
    let harness = Harness::new_ui(|ui| {
        Regui::new("child").size(CHILD_SIZE).show(ui, |ui| {
            ui.ctx().request_repaint();
        });
    });

    let parent_id = harness.ctx.viewport_id();
    assert!(
        harness.ctx.has_requested_repaint_for(&parent_id),
        "the child's repaint request did not reach the parent"
    );
}

/// The child hovers its own widgets, and is told when the pointer leaves.
#[test]
fn the_child_tracks_the_pointer_entering_and_leaving() {
    let probe = Probe::default();

    let mut harness = Harness::new_ui(|ui| {
        let output = Regui::new("child").size(CHILD_SIZE).show(ui, |ui| {
            let response = ui.allocate_response(vec2(50.0, 20.0), Sense::click());
            probe.rect.set(Some(response.rect));
            probe.hovered.set(response.hovered());
        });
        probe.transform.set(Some(output.transform));
    });

    harness.run();

    move_pointer_to(&mut harness, probe.widget_center());
    assert!(probe.hovered.get(), "the child did not see the pointer");

    // Move the pointer away. The child cannot work this out on its own: it just stops
    // hearing about a pointer that, as far as it knows, is still there.
    move_pointer_to(&mut harness, Pos2::new(2000.0, 2000.0));
    assert!(
        !probe.hovered.get(),
        "the child was not told the pointer had left"
    );
}

/// A child that senses nothing should let clicks fall through to the parent.
#[test]
fn a_non_interactive_child_does_not_swallow_clicks() {
    let probe = Probe::default();
    let parent_clicks = Cell::new(0_u32);

    let mut harness = Harness::new_ui(|ui| {
        let output = Regui::new("child")
            .size(CHILD_SIZE)
            .interactive(false)
            .show(ui, |ui| {
                let response = ui.button("child button");
                probe.rect.set(Some(response.rect));
                if response.clicked() {
                    probe.clicks.set(probe.clicks.get() + 1);
                }
            });
        probe.transform.set(Some(output.transform));
        if output.response.interact(Sense::click()).clicked() {
            parent_clicks.set(parent_clicks.get() + 1);
        }
    });

    harness.run();
    click_at(&mut harness, probe.widget_center());

    assert_eq!(
        probe.clicks.get(),
        0,
        "a hover-only child should not be clickable"
    );
    assert_eq!(parent_clicks.get(), 1, "the click did not fall through");
}

/// Key events reach the child once it has focus.
#[test]
fn key_events_reach_a_focused_child() {
    let probe = Probe::default();
    let escapes = Cell::new(0_u32);

    let mut harness = Harness::new_ui(|ui| {
        let output = Regui::new("child").size(CHILD_SIZE).show(ui, |ui| {
            let response = ui.button("focus me");
            probe.rect.set(Some(response.rect));
            if response.clicked() {
                response.request_focus();
            }
            if ui.input(|input| input.key_pressed(Key::Escape)) {
                escapes.set(escapes.get() + 1);
            }
        });
        probe.transform.set(Some(output.transform));
    });

    harness.run();
    click_at(&mut harness, probe.widget_center());

    harness.input_mut().events.push(Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    harness.run();

    assert_eq!(
        escapes.get(),
        1,
        "the focused child did not get the key press"
    );
}

/// `show` passes the ui function's value back out, so it composes like other egui
/// containers.
#[test]
fn show_returns_the_inner_value() {
    let mut harness = Harness::new_ui(|ui| {
        let output = Regui::new("child").size(CHILD_SIZE).show(ui, |_ui| 42_u32);
        assert_eq!(output.inner, 42);
    });
    harness.run();
}

/// A menu inside the child must open and be clickable.
///
/// This is the most demanding thing to put in a child: the popup lives in a different layer
/// of the child's viewport, it takes an extra pass to appear, and it only works if the
/// child's areas and interaction state all survive between passes.
#[test]
fn a_menu_inside_the_child_works() {
    let probe = Probe::default();
    let item_rect = Cell::new(None::<Rect>);
    let chosen = RefCell::new(String::new());

    let mut harness = Harness::builder().with_size([300.0, 260.0]).build_ui(|ui| {
        let output = Regui::new("child").size(vec2(220.0, 200.0)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let response = ui.menu_button("Menu", |ui| {
                    let item = ui.button("Second");
                    item_rect.set(Some(item.rect));
                    if item.clicked() {
                        *chosen.borrow_mut() = "Second".to_owned();
                        ui.close();
                    }
                });
                probe.rect.set(Some(response.response.rect));
            });
        });
        probe.transform.set(Some(output.transform));
    });

    harness.run();

    // Open the menu.
    click_at(&mut harness, probe.widget_center());
    harness.run();

    let item = item_rect
        .get()
        .expect("the menu did not open, so its contents never ran");
    let transform = probe.transform.get().expect("the child ui should have run");
    click_at(&mut harness, transform.mul_pos(item.center()));

    assert_eq!(
        chosen.borrow().as_str(),
        "Second",
        "the menu item inside the child was not clickable"
    );
}
