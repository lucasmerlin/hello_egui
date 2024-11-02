use crate::chat::CHAT_EXAMPLE;
use crate::color_sort::{COLOR_SORT_EXAMPLE, COLOR_SORT_WRAPPED_EXAMPLE};
use crate::crate_ui::CrateUsage;
use crate::flex::FLEX_EXAMPLE;
use crate::gallery::GALLERY_EXAMPLE;
use crate::shared_state::SharedState;
use crate::signup_form::SIGNUP_FORM_EXAMPLE;
use crate::stargazers::STARGAZERS_EXAMPLE;
use egui::Ui;

pub const EXAMPLES: &[Category] = &[
    Category {
        name: "Drag and Drop",
        examples: &[
            COLOR_SORT_EXAMPLE,
            COLOR_SORT_WRAPPED_EXAMPLE,
            STARGAZERS_EXAMPLE,
        ],
    },
    Category {
        name: "Infinite Scroll",
        examples: &[CHAT_EXAMPLE, GALLERY_EXAMPLE],
    },
    Category {
        name: "Form Validation",
        examples: &[SIGNUP_FORM_EXAMPLE],
    },
    Category {
        name: "Layout",
        examples: &[FLEX_EXAMPLE],
    },
];

pub trait ExampleTrait {
    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState);
}

pub struct Category {
    pub name: &'static str,
    pub examples: &'static [Example],
}

#[derive(Clone, Copy)]
pub struct Example {
    pub name: &'static str,
    pub slug: &'static str,
    pub crates: &'static [CrateUsage],
    pub get: fn() -> Box<dyn ExampleTrait>,
}

impl PartialEq for Example {
    fn eq(&self, other: &Self) -> bool {
        self.slug == other.slug
    }
}

impl Example {
    pub fn get(&self) -> Box<dyn ExampleTrait> {
        (self.get)()
    }
}
