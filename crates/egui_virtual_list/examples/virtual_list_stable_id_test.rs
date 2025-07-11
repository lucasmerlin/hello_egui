use egui::ScrollArea;
use egui_virtual_list::VirtualList;
use hello_egui_utils_dev::run;

fn main() {
    let mut vl = VirtualList::default();

    run!(move |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            vl.ui_custom_layout(ui, 100, |ui, first_index| {
                let response = ui.button("Hello");
                ui.label(format!("id: {:?}, index: {}", response.id, first_index));

                1
            });
        });
    });
}
