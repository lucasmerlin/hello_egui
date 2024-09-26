use std::time::{Duration, Instant};

use eframe::egui::{CentralPanel, Pos2, Vec2};
use eframe::{egui, NativeOptions};

use perfect_cursors::PerfectCursor;

struct CursorSim {
    delay: Duration,
    last_time: Instant,
    cursor: PerfectCursor,
    offset: Vec2,
    next_point: Option<Pos2>,
    received_point: Option<Pos2>,
    last_point: Option<Pos2>,
}

pub fn main() -> eframe::Result<()> {
    let mut cursors = vec![
        CursorSim {
            delay: Duration::from_millis(80),
            last_time: Instant::now(),
            cursor: PerfectCursor::new(),
            offset: Vec2::new(60.0, 0.0),
            next_point: None,
            received_point: None,
            last_point: None,
        },
        CursorSim {
            delay: Duration::from_millis(160),
            last_time: Instant::now(),
            cursor: PerfectCursor::new(),
            offset: Vec2::new(120.0, 0.0),
            next_point: None,
            received_point: None,
            last_point: None,
        },
        CursorSim {
            delay: Duration::from_millis(250),
            last_time: Instant::now(),
            cursor: PerfectCursor::new(),
            offset: Vec2::new(180.0, 0.0),
            next_point: None,
            received_point: None,
            last_point: None,
        },
    ];

    let mut show_update_events = false;
    let mut use_perfect_cursors = true;

    eframe::run_simple_native(
        "Perfect Cursors Egui Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut show_update_events, "Show update events");
                    ui.checkbox(&mut use_perfect_cursors, "Use Perfect Cursors");
                });

                for cursor in &mut cursors {
                    if cursor.last_time.elapsed() > cursor.delay {
                        cursor.last_time = Instant::now();

                        if let Some(next) = cursor.next_point.take() {
                            if use_perfect_cursors {
                                cursor.cursor.add_point(next.into());
                            } else {
                                cursor.received_point = Some(next);
                            }
                            if show_update_events {
                                ui.painter().circle(
                                    next + cursor.offset,
                                    15.0,
                                    egui::Color32::BLUE,
                                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                                );
                            }
                        }

                        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                            if cursor.last_point != Some(pos) {
                                cursor.next_point = Some(pos);
                            }
                            cursor.last_point = Some(pos);
                        }
                    }
                }

                for cursor in &mut cursors {
                    let current_pos = if use_perfect_cursors {
                        cursor.cursor.tick().map(std::convert::Into::into)
                    } else {
                        cursor.received_point
                    };

                    if let Some(pos) = current_pos {
                        ui.painter().circle(
                            pos + cursor.offset,
                            10.0,
                            egui::Color32::RED,
                            egui::Stroke::new(1.0, egui::Color32::WHITE),
                        );
                    }

                    ui.ctx().request_repaint();
                }
            });
        },
    )
}
