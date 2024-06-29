#[cfg(feature = "async")]
mod async_route;
mod handler;
pub mod history;
mod route_kind;
mod router;
mod router_builder;
pub mod transition;

use crate::history::HistoryError;
use crate::transition::{ActiveTransition, SlideFadeTransition, SlideTransition, Transition};
use egui::emath::ease_in_ease_out;
use egui::{Ui, Vec2};
use std::sync::atomic::AtomicUsize;

pub use handler::{HandlerError, HandlerResult};
pub use router::EguiRouter;

#[cfg(feature = "async")]
pub use handler::async_impl::OwnedRequest;

pub trait Route<State> {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State);
}

static ID: AtomicUsize = AtomicUsize::new(0);

struct RouteState<State> {
    path: String,
    route: HandlerResult<Box<dyn Route<State>>>,
    id: usize,
    state: u32,
}

pub type RouterResult<T = ()> = Result<T, RouterError>;

#[derive(Debug)]
pub enum RouterError {
    HistoryError(HistoryError),
    NotFound,
}

impl From<HistoryError> for RouterError {
    fn from(err: HistoryError) -> Self {
        Self::HistoryError(err)
    }
}

#[derive(Debug, Clone)]
pub struct TransitionConfig {
    duration: Option<f32>,
    easing: fn(f32) -> f32,
    _in: Transition,
    out: Transition,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            duration: None,
            easing: ease_in_ease_out,
            _in: transition::SlideTransition::new(Vec2::X).into(),
            out: transition::SlideTransition::new(Vec2::X * -0.1).into(),
        }
    }
}

impl TransitionConfig {
    pub fn new(_in: impl Into<Transition>, out: impl Into<Transition>) -> Self {
        Self {
            _in: _in.into(),
            out: out.into(),
            ..Self::default()
        }
    }

    pub fn slide() -> Self {
        Self::default()
    }

    pub fn fade_up() -> Self {
        Self::new(
            SlideFadeTransition(
                SlideTransition::new(Vec2::Y * 0.3),
                transition::FadeTransition,
            ),
            transition::NoTransition,
        )
    }

    pub fn fade() -> Self {
        Self::new(transition::FadeTransition, transition::FadeTransition)
    }

    pub fn none() -> Self {
        Self::new(transition::NoTransition, transition::NoTransition)
    }

    pub fn with_easing(mut self, easing: fn(f32) -> f32) -> Self {
        self.easing = easing;
        self
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }
}

struct CurrentTransition<State> {
    active_transition: ActiveTransition,
    leaving_route: Option<RouteState<State>>,
}

pub struct Request<'a, State = ()> {
    pub params: matchit::Params<'a, 'a>,
    pub state: &'a mut State,
}

// impl<F, Fut, State, R: 'static> Handler<State> for F
// where
//     F: Fn(&mut State) -> Fut,
//     Fut: std::future::Future<Output = R>,
// {
//     async fn handle(&mut self, state: &mut State) -> Box<dyn Route<State>> {
//         Box::new((self(state)).await)
//     }
// }

impl<F: FnMut(&mut Ui, &mut State), State> Route<State> for F {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        self(ui, state)
    }
}
