use egui::style::StyleModifier;
use egui::{Color32, Response, Ui, Widget};
use std::ops::{Deref, DerefMut};

pub struct StyledWidget<Widget> {
    modifier: StyleModifier,
    widget: Widget,
}

impl<Widget: egui::Widget> egui::Widget for StyledWidget<Widget> {
    fn ui(self, ui: &mut Ui) -> Response {
        let previous_style = ui.style().clone();
        self.modifier.apply(ui.style_mut());
        let response = self.widget.ui(ui);
        ui.set_style(previous_style);
        response
    }
}

pub trait PrimaryStyleExt {
    fn primary(self) -> StyledWidget<Self>
    where
        Self: Sized,
    {
        StyledWidget {
            modifier: StyleModifier::new(|style| {
                style.visuals.widgets.inactive.bg_fill = Color32::RED;
                style.visuals.widgets.hovered.bg_fill = Color32::RED;
                style.visuals.widgets.hovered.weak_bg_fill = Color32::RED;
                style.visuals.widgets.inactive.weak_bg_fill = Color32::RED;
                style.visuals.widgets.inactive.fg_stroke.color = Color32::RED;
            }),
            widget: self,
        }
    }
}

impl<T: Widget> PrimaryStyleExt for T {}

impl<T> Deref for StyledWidget<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<T> DerefMut for StyledWidget<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

macro_rules! classes {
    ($name:ident, ($class:ident)) => {
        pub trait $name {
            fn $class(self) -> StyledWidget<Self>
            where
                Self: Sized,
            {
                StyledWidget {
                    modifier: StyleModifier::new(|style| {
                        style.visuals.widgets.inactive.$class = Color32::RED;
                        style.visuals.widgets.hovered.$class = Color32::RED;
                        style.visuals.widgets.hovered.weak_bg_fill = Color32::RED;
                        style.visuals.widgets.inactive.weak_bg_fill = Color32::RED;
                        style.visuals.widgets.inactive.fg_stroke.color = Color32::RED;
                    }),
                    widget: self,
                }
            }
        }
    };
}