use crate::shared_state::SharedState;
use egui::{Align, Layout, Ui};

pub trait Example {
    fn name(&self) -> &'static str;

    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState);
}

pub struct Category {
    pub name: String,
    pub examples: Vec<Box<dyn Example>>,
}

pub struct SideBar {
    categories: Vec<Category>,
    active_category: usize,
    active_example: usize,
}

impl SideBar {
    pub fn new(categories: Vec<Category>) -> Self {
        Self {
            categories,
            active_category: 0,
            active_example: 0,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> bool {
        let mut clicked = false;
        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
            ui.add_space(4.0);
            ui.heading("hello_egui");
            ui.add_space(4.0);

            for (category_idx, category) in &mut self.categories.iter_mut().enumerate() {
                ui.small(&category.name);
                for (example_idx, example) in category.examples.iter_mut().enumerate() {
                    if ui.button(example.name()).clicked() {
                        self.active_category = category_idx;
                        self.active_example = example_idx;
                        clicked = true;
                    }
                }
            }
        });

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add_space(8.0);
            ui.hyperlink_to(
                "GitHub: hello_egui",
                "https://github.com/lucasmerlin/hello_egui",
            );
        });

        clicked
    }

    pub fn active_example_mut(&mut self) -> &mut dyn Example {
        &mut *self.categories[self.active_category].examples[self.active_example]
    }
}
