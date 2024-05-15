use crate::TransitionConfig;
use egui::emath::ease_in_ease_out;
use egui::{vec2, Ui};

pub trait TransitionTrait {
    fn show(&self, ui: &mut Ui, t: f32, content: impl FnOnce(&mut Ui));
}

#[derive(Debug, Clone)]
pub enum Transition {
    Fade(FadeTransition),
    NoTransition(NoTransition),
    Slide(SlideTransition),
}

impl TransitionTrait for Transition {
    fn show(&self, ui: &mut Ui, t: f32, content: impl FnOnce(&mut Ui)) {
        match self {
            Transition::Fade(fade) => fade.show(ui, t, content),
            Transition::NoTransition(no_transition) => no_transition.show(ui, t, content),
            Transition::Slide(slide) => slide.show(ui, t, content),
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
    fn show(&self, ui: &mut Ui, t: f32, content: impl FnOnce(&mut Ui)) {
        let _ = ui.scope(|ui| {
            ui.set_opacity(t);
            content(ui);
        });
    }
}

impl TransitionTrait for NoTransition {
    fn show(&self, ui: &mut Ui, _t: f32, content: impl FnOnce(&mut Ui)) {
        content(ui);
    }
}

impl TransitionTrait for SlideTransition {
    fn show(&self, ui: &mut Ui, t: f32, content: impl FnOnce(&mut Ui)) {
        let width = ui.available_width();
        let offset = width * (1.0 - t) * self.amount;
        let child_rect = ui.max_rect().translate(vec2(offset, 0.0));

        let mut child_ui = ui.child_ui(child_rect, *ui.layout());
        content(&mut child_ui);
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
    transition: TransitionType,
}

pub enum ActiveTransitionResult {
    Done,
    DonePop,
    Continue,
}

impl ActiveTransition {
    pub fn new(duration: Option<f32>, transition: TransitionType) -> Self {
        Self {
            duration,
            easing: ease_in_ease_out,
            progress: 0.0,
            transition,
        }
    }

    pub fn forward(config: TransitionConfig) -> Self {
        Self {
            duration: config.duration,
            easing: config.easing,
            progress: 0.0,
            transition: TransitionType::Forward {
                _in: config._in,
                out: config.out,
            },
        }
    }

    pub fn backward(config: TransitionConfig) -> Self {
        Self {
            duration: config.duration,
            easing: config.easing,
            progress: 0.0,
            transition: TransitionType::Backward {
                _in: config._in,
                out: config.out,
            },
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
        content_in: impl FnOnce(&mut Ui, &mut State),
        content_out: Option<impl FnOnce(&mut Ui, &mut State)>,
    ) -> ActiveTransitionResult {
        let dt = ui.input(|i| i.stable_dt);

        self.progress += dt / self.duration.unwrap_or_else(|| ui.style().animation_time);

        let t = (self.easing)(self.progress);
        ui.ctx().request_repaint();

        match &self.transition {
            TransitionType::Forward { _in, out, .. } => {
                if let Some(content_out) = content_out {
                    let mut child_b = ui.child_ui(ui.max_rect(), *ui.layout());
                    out.show(&mut child_b, 1.0 - t, |ui| content_out(ui, state));
                }

                let mut child_a = ui.child_ui(ui.max_rect(), *ui.layout());
                _in.show(&mut child_a, t, |ui| content_in(ui, state));
            }
            TransitionType::Backward { _in, out, .. } => {
                if let Some(content_out) = content_out {
                    let mut child_b = ui.child_ui(ui.max_rect(), *ui.layout());
                    out.show(&mut child_b, t, |ui| content_out(ui, state));
                }

                let mut child_a = ui.child_ui(ui.max_rect(), *ui.layout());
                _in.show(&mut child_a, 1.0 - t, |ui| content_in(ui, state));
            }
        }

        if self.progress >= 1.0 {
            match &self.transition {
                TransitionType::Forward { .. } => ActiveTransitionResult::Done,
                TransitionType::Backward { .. } => ActiveTransitionResult::DonePop,
            }
        } else {
            ActiveTransitionResult::Continue
        }
    }
}
