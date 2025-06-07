#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "async")]
mod async_route;
mod handler;
/// History types
pub mod history;
mod route_kind;
mod router;
mod router_builder;
/// Transition types
pub mod transition;

use crate::history::HistoryError;
use crate::transition::{ActiveTransition, SlideFadeTransition, SlideTransition, Transition};
use egui::emath::ease_in_ease_out;
use egui::{Ui, Vec2};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;

pub use handler::{HandlerError, HandlerResult};
pub use router::EguiRouter;
pub use router_builder::RouterBuilder;

/// A route instance created by a [`handler::Handler`]
pub trait Route<State = ()> {
    /// Render the route ui
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State);
}

impl<F: FnMut(&mut Ui, &mut State), State> Route<State> for F {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        self(ui, state);
    }
}

static ID: AtomicUsize = AtomicUsize::new(0);

struct RouteState<State> {
    path: String,
    route: HandlerResult<Box<dyn Route<State>>>,
    id: usize,
    state: u32,
}

/// Router Result type
pub type RouterResult<T = ()> = Result<T, RouterError>;

/// Router error
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    /// Error when updating history
    #[error("History error: {0}")]
    HistoryError(HistoryError),
    /// Not found error
    #[error("Route not found")]
    NotFound,
}

impl From<HistoryError> for RouterError {
    fn from(err: HistoryError) -> Self {
        Self::HistoryError(err)
    }
}

/// Page transition configuration
#[derive(Debug, Clone)]
pub struct TransitionConfig {
    duration: Option<f32>,
    easing: fn(f32) -> f32,
    in_: Transition,
    out: Transition,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            duration: None,
            easing: ease_in_ease_out,
            in_: transition::SlideTransition::new(Vec2::X).into(),
            out: transition::SlideTransition::new(Vec2::X * -0.1).into(),
        }
    }
}

impl TransitionConfig {
    /// Create a new transition
    pub fn new(in_: impl Into<Transition>, out: impl Into<Transition>) -> Self {
        Self {
            in_: in_.into(),
            out: out.into(),
            ..Self::default()
        }
    }

    /// A iOS-like slide transition (Same as [`TransitionConfig::default`])
    #[must_use]
    pub fn slide() -> Self {
        Self::default()
    }

    /// A android-like fade up transition
    #[must_use]
    pub fn fade_up() -> Self {
        Self::new(
            SlideFadeTransition(
                SlideTransition::new(Vec2::Y * 0.3),
                transition::FadeTransition,
            ),
            transition::NoTransition,
        )
    }

    /// A basic fade transition
    #[must_use]
    pub fn fade() -> Self {
        Self::new(transition::FadeTransition, transition::FadeTransition)
    }

    /// No transition
    #[must_use]
    pub fn none() -> Self {
        Self::new(transition::NoTransition, transition::NoTransition)
    }

    /// Customise the easing function
    #[must_use]
    pub fn with_easing(mut self, easing: fn(f32) -> f32) -> Self {
        self.easing = easing;
        self
    }

    /// Customise the duration
    #[must_use]
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }
}

struct CurrentTransition<State> {
    active_transition: ActiveTransition,
    leaving_route: Option<RouteState<State>>,
}

/// Request passed to a [`handler::MakeHandler`]
pub struct Request<'a, State = ()> {
    /// The parsed path params
    pub params: matchit::Params<'a, 'a>,
    /// The parsed query params
    pub query: BTreeMap<Cow<'a, str>, Cow<'a, str>>,
    /// The custom state
    pub state: &'a mut State,
}

#[cfg(feature = "async")]
/// Owned request, passed to [`handler::AsyncMakeHandler`]
pub struct OwnedRequest<State = ()> {
    /// The parsed path params
    pub params: BTreeMap<String, String>,
    /// The parsed query params
    pub query: BTreeMap<String, String>,
    /// The custom state
    pub state: State,
}
