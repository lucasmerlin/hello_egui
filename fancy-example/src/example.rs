use crate::chat::CHAT_EXAMPLE;
use crate::color_sort::{COLOR_SORT_EXAMPLE, COLOR_SORT_WRAPPED_EXAMPLE};
use crate::crate_ui::Crate::EguiDnd;
use crate::crate_ui::CrateUsage;
use crate::gallery::GALLERY_EXAMPLE;
use crate::shared_state::SharedState;
use crate::signup_form::SIGNUP_FORM_EXAMPLE;
use crate::stargazers::{Stargazers, STARGAZERS_EXAMPLE};
use egui::Ui;

pub const EXAMPLES: &[Category] = &[
    Category {
        name: "Drag and Drop",
        slug: "dnd",
        examples: &[
            COLOR_SORT_EXAMPLE,
            COLOR_SORT_WRAPPED_EXAMPLE,
            STARGAZERS_EXAMPLE,
        ],
    },
    Category {
        name: "Infinite Scroll",
        slug: "infinite_scroll",
        examples: &[CHAT_EXAMPLE, GALLERY_EXAMPLE],
    },
    Category {
        name: "Form Validation",
        slug: "form_validation",
        examples: &[SIGNUP_FORM_EXAMPLE],
    },
];

pub trait ExampleTrait {
    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState);
}

pub struct Category {
    pub name: &'static str,
    pub slug: &'static str,
    pub examples: &'static [Example],
}

pub struct Example {
    pub name: &'static str,
    pub slug: &'static str,
    pub crates: &'static [CrateUsage],
    pub get: fn() -> Box<dyn ExampleTrait>,
}

impl Example {
    pub fn get(&self) -> Box<dyn ExampleTrait> {
        (self.get)()
    }
}
