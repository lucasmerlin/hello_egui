// This example shows how to sort items in a horizontal_wrapped layout where each item has a different size.
use eframe::{egui, NativeOptions};

use egui::{CentralPanel, Frame, Id, ScrollArea, Ui, Vec2};
use egui_dnd::dnd;
use hello_egui_utils::measure_text;

pub fn dnd_ui(ui: &mut Ui, items: &mut [(usize, String)]) {
    ui.horizontal_wrapped(|ui| {
        dnd(ui, "dnd_example").show_custom_vec(items, |ui, items, item_iter| {
            items.iter().enumerate().for_each(|(idx, item)| {
                let size = measure_text(ui, &item.1);

                let frame_padding = 4.0;
                let size = size + Vec2::splat(frame_padding) * 2.0;

                item_iter.next(ui, Id::new(item.0), idx, true, |ui, item_handle| {
                    item_handle.ui_sized(ui, size, |ui, handle, _state| {
                        Frame::NONE
                            .inner_margin(frame_padding)
                            .fill(ui.visuals().extreme_bg_color)
                            .corner_radius(4.0)
                            .show(ui, |ui| {
                                handle.ui_sized(ui, size, |ui| {
                                    ui.label(&item.1);
                                });
                            });
                    })
                });
            });
        });
    });
}

const TEXT: &str = r"
Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.

Duis autem vel eum iriure dolor in hendrerit in vulputate velit esse molestie consequat, vel illum dolore eu feugiat nulla facilisis at vero eros et accumsan et iusto odio dignissim qui blandit praesent luptatum zzril delenit augue duis dolore te feugait nulla facilisi. Lorem ipsum dolor sit amet, consectetuer adipiscing elit, sed diam nonummy nibh euismod tincidunt ut laoreet dolore magna aliquam erat volutpat.

Ut wisi enim ad minim veniam, quis nostrud exerci tation ullamcorper suscipit lobortis nisl ut aliquip ex ea commodo consequat. Duis autem vel eum iriure dolor in hendrerit in vulputate velit esse molestie consequat, vel illum dolore eu feugiat nulla facilisis at vero eros et accumsan et iusto odio dignissim qui blandit praesent luptatum zzril delenit augue duis dolore te feugait nulla facilisi.

Nam liber tempor cum soluta nobis eleifend option congue nihil imperdiet doming id quod mazim placerat facer possim assum. Lorem ipsum dolor sit amet, consectetuer adipiscing elit, sed diam nonummy nibh euismod tincidunt ut laoreet dolore magna aliquam erat volutpat. Ut wisi enim ad minim veniam, quis nostrud exerci tation ullamcorper suscipit lobortis nisl ut aliquip ex ea commodo consequat.

Duis autem vel eum iriure dolor in hendrerit in vulputate velit esse molestie consequat, vel illum dolore eu feugiat nulla facilisis.

At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, At accusam aliquyam diam diam dolore dolores duo eirmod eos erat, et nonumy sed tempor et et invidunt justo labore Stet clita ea et gubergren, kasd magna no rebum. sanctus sea sed takimata ut vero voluptua. est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat.

Consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus.
";

pub fn main() -> eframe::Result<()> {
    let mut items = TEXT
        .replace('\n', "")
        .split(' ')
        .map(|i| i.trim().to_string())
        // Store the index so we can use it as id, since there are duplicate words
        .enumerate()
        .collect::<Vec<_>>();

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default()
                .frame(Frame::NONE.inner_margin(8.0).fill(
                    ctx.style().visuals.panel_fill.gamma_multiply(
                        if ctx.style().visuals.dark_mode {
                            1.5
                        } else {
                            0.8
                        },
                    ),
                ))
                .show(ctx, |ui| {
                    ui.style_mut().animation_time = 0.5;
                    ui.spacing_mut().item_spacing.x = ui.spacing().item_spacing.y;

                    ScrollArea::vertical().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        dnd_ui(ui, &mut items);
                    });
                });
        },
    )
}
