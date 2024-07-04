use crate::TransitionConfig;
use egui::{Id, Ui, Vec2};

/// Trait for declaring a transition.
/// Prefer [ComposableTransitionTrait] unless you need to create a new ui to apply the transition.
pub trait TransitionTrait {
    /// Create a child ui with the transition applied
    fn create_child_ui(&self, ui: &mut Ui, t: f32, with_id: Id) -> Ui;
}

/// Trait for declaring a composable transition.
pub trait ComposableTransitionTrait {
    /// Apply the transition to the ui
    fn apply(&self, ui: &mut Ui, t: f32);
}

impl<T: ComposableTransitionTrait> TransitionTrait for T {
    fn create_child_ui(&self, ui: &mut Ui, t: f32, with_id: Id) -> Ui {
        let mut child = ui.child_ui_with_id_source(ui.max_rect(), *ui.layout(), with_id, None);
        self.apply(&mut child, t);
        child
    }
}

/// Enum containing all possible transitions
#[derive(Debug, Clone)]
pub enum Transition {
    /// Simple fade transition
    Fade(FadeTransition),
    /// No transition
    NoTransition(NoTransition),
    /// Slide transition
    Slide(SlideTransition),
    /// Combined slide and fade transitions
    SlideFade(SlideFadeTransition),
}

impl TransitionTrait for Transition {
    fn create_child_ui(&self, ui: &mut Ui, t: f32, with_id: Id) -> Ui {
        match self {
            Transition::Fade(fade) => fade.create_child_ui(ui, t, with_id),
            Transition::NoTransition(no_transition) => {
                no_transition.create_child_ui(ui, t, with_id)
            }
            Transition::Slide(slide) => slide.create_child_ui(ui, t, with_id),
            Transition::SlideFade(slide_fade) => slide_fade.create_child_ui(ui, t, with_id),
        }
    }
}

/// Simple fade transition
#[derive(Debug, Clone)]
pub struct FadeTransition;

/// No transition
#[derive(Debug, Clone)]
pub struct NoTransition;

/// Slide transition
#[derive(Debug, Clone)]
pub struct SlideTransition {
    /// Amount and direction to slide. Default is [Vec2::X] (so it will slide in from the right)
    pub amount: Vec2,
}

/// Combining slide and fade transitions
#[derive(Debug, Clone)]
pub struct SlideFadeTransition(pub SlideTransition, pub FadeTransition);

impl Default for SlideTransition {
    fn default() -> Self {
        Self { amount: Vec2::X }
    }
}

impl SlideTransition {
    /// Create a new slide transition. Default is [Vec2::X] (so it will slide in from the right)
    pub fn new(amount: Vec2) -> Self {
        Self { amount }
    }
}

impl ComposableTransitionTrait for FadeTransition {
    fn apply(&self, ui: &mut Ui, t: f32) {
        ui.set_opacity(t);
    }
}

impl ComposableTransitionTrait for NoTransition {
    fn apply(&self, _: &mut Ui, _: f32) {}
}

impl TransitionTrait for SlideTransition {
    fn create_child_ui(&self, ui: &mut Ui, t: f32, with_id: Id) -> Ui {
        let available_size = ui.available_size();
        let offset = available_size * (1.0 - t) * self.amount;
        let child_rect = ui.max_rect().translate(offset);

        ui.child_ui_with_id_source(child_rect, *ui.layout(), with_id, None)
    }
}

impl TransitionTrait for SlideFadeTransition {
    fn create_child_ui(&self, ui: &mut Ui, t: f32, with_id: Id) -> Ui {
        let mut child = self.0.create_child_ui(ui, t, with_id);
        self.1.apply(&mut child, t);
        child
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

impl From<SlideFadeTransition> for Transition {
    fn from(slide_fade: SlideFadeTransition) -> Self {
        Transition::SlideFade(slide_fade)
    }
}

/// Configuration for a transition, containing the in and out transitions
/// The in transition is the transition that will be applied to the page that is being navigated to
/// The out transition is the transition that will be applied to the page that is being navigated from
pub enum TransitionType {
    /// Forward transition
    Forward {
        /// Will be applied to the page that is being navigated to
        _in: Transition,
        /// Will be applied to the page that is being navigated from
        out: Transition,
    },
    /// Backward transition (will play the out transition in reverse)
    Backward {
        /// Will be applied to the page that is being navigated to
        _in: Transition,
        /// Will be applied to the page that is being navigated from
        out: Transition,
    },
}

pub(crate) struct ActiveTransition {
    duration: Option<f32>,
    progress: f32,
    easing: fn(f32) -> f32,
    _in: Transition,
    out: Transition,
    backward: bool,
}

pub(crate) enum ActiveTransitionResult {
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
        (in_id, content_in): (usize, impl FnOnce(&mut Ui, &mut State)),
        content_out: Option<(usize, impl FnOnce(&mut Ui, &mut State))>,
    ) -> ActiveTransitionResult {
        let dt = ui.input(|i| i.stable_dt);

        self.progress += dt / self.duration.unwrap_or_else(|| ui.style().animation_time);

        let t = self.progress.min(1.0);
        ui.ctx().request_repaint();

        if !self.backward {
            if let Some((out_id, content_out)) = content_out {
                with_temp_auto_id(ui, out_id, |ui| {
                    let mut out_ui = self.out.create_child_ui(
                        ui,
                        (self.easing)(1.0 - t),
                        Id::new("router_child").with(out_id),
                    );
                    content_out(&mut out_ui, state);
                });
            }

            with_temp_auto_id(ui, in_id, |ui| {
                let mut in_ui = self._in.create_child_ui(
                    ui,
                    (self.easing)(t),
                    Id::new("router_child").with(in_id),
                );
                content_in(&mut in_ui, state);
            });
        } else {
            with_temp_auto_id(ui, in_id, |ui| {
                let mut out_ui = self.out.create_child_ui(
                    ui,
                    (self.easing)(t),
                    Id::new("router_child").with(in_id),
                );
                content_in(&mut out_ui, state);
            });

            if let Some((out_id, content_out)) = content_out {
                with_temp_auto_id(ui, out_id, |ui| {
                    let mut in_ui = self._in.create_child_ui(
                        ui,
                        (self.easing)(1.0 - t),
                        Id::new("router_child").with(out_id),
                    );
                    content_out(&mut in_ui, state);
                });
            }
        }

        if self.progress >= 1.0 {
            ActiveTransitionResult::Done
        } else {
            ActiveTransitionResult::Continue
        }
    }

    pub fn show_default(ui: &mut Ui, with_id: usize, content: impl FnOnce(&mut Ui)) {
        with_temp_auto_id(ui, with_id, |ui| {
            let mut ui = ui.child_ui_with_id_source(
                ui.max_rect(),
                *ui.layout(),
                Id::new("router_child").with(with_id),
                None,
            );
            content(&mut ui);
        });
    }
}

fn with_temp_auto_id(ui: &mut Ui, id: usize, content: impl FnOnce(&mut Ui)) {
    ui.skip_ahead_auto_ids(id);
    content(ui);
    ui.skip_ahead_auto_ids(usize::MAX - (id));
}
