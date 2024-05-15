use crate::TransitionConfig;
use egui::emath::ease_in_ease_out;
use egui::{vec2, Id, Ui};

pub trait TransitionTrait {
    fn create_child_ui(&self, ui: &mut Ui, t: f32, with_id: Id) -> Ui;
}

#[derive(Debug, Clone)]
pub enum Transition {
    Fade(FadeTransition),
    NoTransition(NoTransition),
    Slide(SlideTransition),
}

impl TransitionTrait for Transition {
    fn create_child_ui(&self, ui: &mut Ui, t: f32, with_id: Id) -> Ui {
        match self {
            Transition::Fade(fade) => fade.create_child_ui(ui, t, with_id),
            Transition::NoTransition(no_transition) => {
                no_transition.create_child_ui(ui, t, with_id)
            }
            Transition::Slide(slide) => slide.create_child_ui(ui, t, with_id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FadeTransition;

#[derive(Debug, Clone)]
pub struct NoTransition;

#[derive(Debug, Clone)]
pub struct SlideTransition {
    pub amount: f32,
}

impl Default for SlideTransition {
    fn default() -> Self {
        Self { amount: 1.0 }
    }
}

impl SlideTransition {
    pub fn new(amount: f32) -> Self {
        Self { amount }
    }
}

impl TransitionTrait for FadeTransition {
    fn create_child_ui(&self, ui: &mut Ui, t: f32, with_id: Id) -> Ui {
        let mut ui = ui.child_ui_with_id_source(ui.max_rect(), *ui.layout(), with_id);
        ui.set_opacity(t);
        ui
    }
}

impl TransitionTrait for NoTransition {
    fn create_child_ui(&self, ui: &mut Ui, _t: f32, with_id: Id) -> Ui {
        ui.child_ui_with_id_source(ui.max_rect(), *ui.layout(), with_id)
    }
}

impl TransitionTrait for SlideTransition {
    fn create_child_ui(&self, ui: &mut Ui, t: f32, with_id: Id) -> Ui {
        let width = ui.available_width();
        let offset = width * (1.0 - t) * self.amount;
        let child_rect = ui.max_rect().translate(vec2(offset, 0.0));

        ui.child_ui_with_id_source(child_rect, *ui.layout(), with_id)
    }
}

impl From<FadeTransition> for Transition {
    fn from(fade: FadeTransition) -> Self {
        Transition::Fade(fade)
    }
}

impl From<NoTransition> for Transition {
    fn from(no_transition: NoTransition) -> Self {
        Transition::NoTransition(no_transition)
    }
}

impl From<SlideTransition> for Transition {
    fn from(slide: SlideTransition) -> Self {
        Transition::Slide(slide)
    }
}

pub enum TransitionType {
    Forward { _in: Transition, out: Transition },
    Backward { _in: Transition, out: Transition },
}

pub struct ActiveTransition {
    duration: Option<f32>,
    progress: f32,
    easing: fn(f32) -> f32,
    _in: Transition,
    out: Transition,
    backward: bool,
}

pub enum ActiveTransitionResult {
    Done,
    Continue,
}

impl ActiveTransition {
    pub fn forward(config: TransitionConfig) -> Self {
        Self {
            duration: config.duration,
            easing: config.easing,
            progress: 0.0,
            _in: config._in,
            out: config.out,
            backward: false,
        }
    }

    pub fn backward(config: TransitionConfig) -> Self {
        Self {
            duration: config.duration,
            easing: config.easing,
            progress: 0.0,
            _in: config._in,
            out: config.out,
            backward: true,
        }
    }

    pub fn with_default_duration(mut self, duration: Option<f32>) -> Self {
        if self.duration.is_none() {
            self.duration = duration;
        }
        self
    }

    pub fn show<State>(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        (in_id, content_in): (Id, impl FnOnce(&mut Ui, &mut State)),
        content_out: Option<(Id, impl FnOnce(&mut Ui, &mut State))>,
    ) -> ActiveTransitionResult {
        let dt = ui.input(|i| i.stable_dt);

        self.progress += dt / self.duration.unwrap_or_else(|| ui.style().animation_time);

        let t = (self.easing)(self.progress);
        ui.ctx().request_repaint();

        if !self.backward {
            if let Some((out_id, content_out)) = content_out {
                let mut out_ui = self.out.create_child_ui(ui, 1.0 - t, out_id);
                content_out(&mut out_ui, state);
            }

            let mut in_ui = self._in.create_child_ui(ui, t, in_id);
            content_in(&mut in_ui, state);
        } else {
            let mut out_ui = self.out.create_child_ui(ui, t, in_id);
            content_in(&mut out_ui, state);

            if let Some((out_id, content_out)) = content_out {
                let mut in_ui = self._in.create_child_ui(ui, 1.0 - t, out_id);
                content_out(&mut in_ui, state);
            }
        }

        if self.progress >= 1.0 {
            ActiveTransitionResult::Done
        } else {
            ActiveTransitionResult::Continue
        }
    }

    pub fn show_default(ui: &mut Ui, with_id: Id, content: impl FnOnce(&mut Ui)) {
        let mut ui = ui.child_ui_with_id_source(ui.max_rect(), *ui.layout(), with_id);
        content(&mut ui);
    }
}
