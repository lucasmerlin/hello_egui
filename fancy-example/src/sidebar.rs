use crate::crate_ui::{crate_button_ui, Crate, CrateUsage, ALL_CRATES};
use crate::shared_state::SharedState;
use egui::{Align, Direction, Layout, RichText, Ui};
use egui_dnd::DragDropItem;
use egui_taffy::taffy;
use egui_taffy::taffy::{Display, FlexDirection, FlexWrap, Style};
use std::cell::RefCell;

pub trait Example {
    fn name(&self) -> &'static str;

    fn crates(&self) -> &'static [CrateUsage];

    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState);
}

pub struct Category {
    pub name: String,
    pub examples: Vec<Box<dyn Example>>,
}

pub struct SideBar {
    pub categories: Vec<Category>,
    pub active: ActiveElement,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveElement {
    Crate(usize),
    Example(usize, usize),
}

impl ActiveElement {
    pub fn select_crate(c: &Crate) -> Self {
        Self::Crate(ALL_CRATES.iter().position(|x| x == c).unwrap())
    }
}

impl SideBar {
    pub fn new(categories: Vec<Category>) -> Self {
        Self {
            categories,
            active: ActiveElement::Example(0, 0),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> bool {
        let mut clicked = false;
        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
            ui.add_space(4.0);
            ui.heading("hello_egui");
            ui.add_space(4.0);

            ui.label("Examples");
            ui.add_space(4.0);

            ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);

            for (category_idx, category) in &mut self.categories.iter_mut().enumerate() {
                ui.small(&category.name);
                for (example_idx, example) in category.examples.iter_mut().enumerate() {
                    // if ui
                    //     .button(RichText::new(example.name()).size(14.0))
                    //     .clicked()
                    // {
                    //     self.active = ActiveElement::Example(category_idx, example_idx);
                    //     clicked = true;
                    // }

                    clicked |= ui
                        .selectable_value(
                            &mut self.active,
                            ActiveElement::Example(category_idx, example_idx),
                            RichText::new(example.name()).size(14.0),
                        )
                        .clicked();
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();

        ui.label("Crates in hello_egui");

        let taffy_response = RefCell::new(None);
        let response_ref = &taffy_response;

        let mut taffy = egui_taffy::TaffyPass::new(
            ui,
            "crate_list".into(),
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                gap: taffy::Size::length(8.0),
                ..Default::default()
            },
        );

        for (idx, item) in ALL_CRATES.iter().enumerate() {
            let selected = matches!(self.active, ActiveElement::Crate(i) if i == idx);
            taffy.add(
                item.short_name().id(),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
                Layout::centered_and_justified(Direction::LeftToRight),
                move |ui| {
                    if crate_button_ui(ui, item.short_name(), selected).clicked() {
                        response_ref.borrow_mut().replace(idx);
                        //self.active = ActiveElement::Crate(idx);
                        //clicked = true;
                    }
                },
            );
        }

        taffy.show();

        if let Some(idx) = taffy_response.borrow_mut().take() {
            self.active = ActiveElement::Crate(idx);
            clicked = true;
        }

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add_space(8.0);
            ui.hyperlink_to(
                "GitHub: hello_egui",
                "https://github.com/lucasmerlin/hello_egui",
            );
        });

        clicked
    }

    pub fn active_example_mut(&mut self) -> Option<&mut dyn Example> {
        match self.active {
            ActiveElement::Crate(_) => None,
            ActiveElement::Example(category_idx, example_idx) => {
                Some(&mut *self.categories[category_idx].examples[example_idx])
            }
        }
    }

    pub fn active_crate(&self) -> Option<&Crate> {
        match self.active {
            ActiveElement::Crate(idx) => Some(&ALL_CRATES[idx]),
            ActiveElement::Example(_, _) => None,
        }
    }
}
